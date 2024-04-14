use crate::{
    command::{Command, CommandInfo},
    connection::Connection,
    message::Message,
    protocol::rdb::Rdb,
    replica::{connection::ReplicaConnection, replicate_channel},
    store::Store,
    stream::StreamInfo,
};
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Handler {}

impl Handler {
    pub async fn parse_command(message: Message) -> Result<CommandInfo> {
        match message {
            Message::Array(a) => {
                if let Some(name) = a.first().and_then(|v| unpack_bulk_str(v.clone())) {
                    let args: Vec<String> =
                        a.into_iter().skip(1).filter_map(unpack_bulk_str).collect();
                    Ok(CommandInfo::new(name, args))
                } else {
                    Err(anyhow!("Invalid command format"))
                }
            }
            _ => Err(anyhow!("Unexpected command format: {:?}", message)),
        }
    }

    pub async fn handle_stream(
        mut connection: Connection,
        store: Arc<Mutex<Store>>,
        stream_info: Arc<Mutex<StreamInfo>>,
    ) {
        let mut full_resync = false;

        loop {
            if full_resync {
                {
                    let empty_rdb = Rdb::get_empty();
                    let _ = connection.write_bytes(&empty_rdb).await;
                }
                let (repl_handle, handle) = replicate_channel(connection);
                {
                    let stream_info = stream_info.lock().await;
                    stream_info.repl_handles.lock().await.push(repl_handle);
                }
                _ = handle.await;
                return;
            }
            if let Some(message) = connection.read_message().await {
                let cmd_info = match Self::parse_command(message).await {
                    Ok(cmd_info) => cmd_info,
                    Err(_) => {
                        continue;
                    }
                };

                match cmd_info.to_command() {
                    Some(command) => {
                        let command_clone = command.clone();
                        match command {
                            Command::Ping => {
                                _ = connection
                                    .write_message(Message::Simple("PONG".to_string()))
                                    .await;
                            }
                            Command::Echo(message) => {
                                _ = connection.write_message(Message::Simple(message)).await;
                            }
                            Command::Get(key) => {
                                let response = if let Some(entry) = store.lock().await.get(key) {
                                    format!("${}\r\n{}\r\n", entry.value.len(), entry.value)
                                } else {
                                    "$-1\r\n".to_string()
                                };
                                _ = connection.write_bytes(response.as_bytes()).await;
                            }
                            Command::Set(key, entry) => {
                                dbg!(
                                    "Set command with key {} entry {:?}",
                                    key.clone(),
                                    entry.clone()
                                );
                                store.lock().await.set(key, entry);
                                _ = connection
                                    .write_message(Message::Simple("OK".to_string()))
                                    .await;

                                for replication in stream_info
                                    .lock()
                                    .await
                                    .repl_handles
                                    .lock()
                                    .await
                                    .iter_mut()
                                {
                                    if let Some(replica_command) =
                                        command_clone.to_replica_command()
                                    {
                                        _ = replication.sender.send(replica_command).await;
                                    }
                                }
                            }
                            Command::Info => {
                                let info = stream_info.lock().await;
                                let response = format!(
                                    "# Replication\n\
                                    role:{}\n\
                                    connected_clients:{}\n\
                                    master_replid:{}\n\
                                    master_repl_offset:{}\n\
                                    ",
                                    info.role, info.connected_clients, info.id, info.offset
                                );
                                _ = connection.write_message(Message::Bulk(response)).await;
                            }
                            Command::Replconf(_args) => {
                                _ = connection
                                    .write_message(Message::Simple("OK".to_string()))
                                    .await;
                            }
                            Command::Psync => {
                                let message = Message::Simple(format!(
                                    "FULLRESYNC {} 0",
                                    stream_info.lock().await.id
                                ));
                                _ = connection.write_message(message).await;
                                full_resync = true;
                            }
                            Command::Wait => {
                                _ = connection.write_bytes(b":0\r\n").await;
                            }
                            _ => break,
                        }
                    }
                    None => {
                        _ = connection
                            .write_message(Message::Simple("Invalid command".to_string()))
                            .await;
                    }
                }
            }
        }
    }

    pub async fn handle_replica(
        mut replica_connection: ReplicaConnection,
        store: Arc<Mutex<Store>>,
    ) {
        let mut bytes_received = 0;

        loop {
            if let Some(message) = replica_connection.get_response().await {
                let cmd_info = match Self::parse_command(message.clone()).await {
                    Ok(cmd_info) => cmd_info,
                    Err(_) => {
                        continue;
                    }
                };

                match cmd_info.to_command() {
                    Some(command) => match command {
                        Command::Set(key, value) => {
                            store.lock().await.set(key, value);
                        }
                        Command::Replconf(args) => {
                            let command = args
                                .first()
                                .expect("Replconf args is required")
                                .to_lowercase();
                            if command == "getack" {
                                let message = Message::Array(vec![
                                    Message::Bulk("REPLCONF".to_string()),
                                    Message::Bulk("ACK".to_string()),
                                    Message::Bulk(bytes_received.to_string()),
                                ]);
                                _ = replica_connection.write_message(message).await;
                            }
                        }
                        _ => {}
                    },
                    None => {
                        _ = replica_connection
                            .write_message(Message::Simple("Invalid command".to_string()))
                            .await;
                    }
                }
                let message_len = message.encode().as_bytes().len();
                bytes_received += message_len;
            } else {
                println!("Unable to get a message from the stream");
                break;
            }
        }
    }
}

fn unpack_bulk_str(value: Message) -> Option<String> {
    match value {
        Message::Bulk(s) => Some(s),
        _ => None,
    }
}
