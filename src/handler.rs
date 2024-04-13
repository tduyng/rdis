use crate::{
    command::{RedisCommand, RedisCommandInfo},
    protocol::{parser::RespValue, rdb::Rdb},
    replica::replicate_channel,
    store::Store,
    stream::StreamInfo,
};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

pub struct Handler {}

impl Handler {
    pub async fn parse_command(stream: &mut TcpStream) -> Result<RedisCommandInfo> {
        let mut buffer = BytesMut::with_capacity(512);
        let bytes_to_read = stream.read_buf(&mut buffer).await?;
        if bytes_to_read == 0 {
            return Err(anyhow!("Empty buffer!"));
        };
        let (value, _) = RespValue::decode(buffer)?;
        match value {
            RespValue::Array(a) => {
                if let Some(name) = a.first().and_then(|v| unpack_bulk_str(v.clone())) {
                    let args: Vec<String> =
                        a.into_iter().skip(1).filter_map(unpack_bulk_str).collect();
                    Ok(RedisCommandInfo::new(name, args))
                } else {
                    Err(anyhow!("Invalid command format"))
                }
            }
            _ => Err(anyhow!("Unexpected command format: {:?}", value)),
        }
    }

    pub async fn handle_stream(
        mut stream: TcpStream,
        store: Arc<Mutex<Store>>,
        stream_info: Arc<Mutex<StreamInfo>>,
    ) {
        let mut full_resync = false;

        loop {
            if full_resync {
                {
                    let empty_rdb = Rdb::get_empty();
                    let _ = stream.write_all(&empty_rdb).await;
                }
                let (repl_handle, handle) = replicate_channel(stream);
                {
                    let stream_info = stream_info.lock().await;
                    stream_info.repl_handles.lock().await.push(repl_handle);
                }
                _ = handle.await;
                return;
            }
            let cmd_info = match Self::parse_command(&mut stream).await {
                Ok(cmd_info) => cmd_info,
                Err(_) => {
                    continue;
                }
            };

            match cmd_info.to_command() {
                Some(command) => {
                    let command_clone = command.clone();
                    match command {
                        RedisCommand::Ping => {
                            write_response(
                                &mut stream,
                                RespValue::SimpleString("PONG".to_string()).encode(),
                            )
                            .await
                        }
                        RedisCommand::Echo(message) => {
                            write_response(&mut stream, RespValue::SimpleString(message).encode())
                                .await;
                        }
                        RedisCommand::Get(key) => {
                            let response = if let Some(entry) = store.lock().await.get(key) {
                                format!("${}\r\n{}\r\n", entry.value.len(), entry.value)
                            } else {
                                "$-1\r\n".to_string()
                            };
                            write_response(&mut stream, response).await;
                        }
                        RedisCommand::Set(key, entry) => {
                            dbg!(
                                "Set command with key {} entry {:?}",
                                key.clone(),
                                entry.clone()
                            );
                            store.lock().await.set(key, entry);
                            write_response(
                                &mut stream,
                                RespValue::SimpleString("OK".to_string()).encode(),
                            )
                            .await;

                            for replication in stream_info
                                .lock()
                                .await
                                .repl_handles
                                .lock()
                                .await
                                .iter_mut()
                            {
                                if let Some(replica_command) = command_clone.to_replica_command() {
                                    _ = replication.sender.send(replica_command).await;
                                }
                            }
                        }
                        RedisCommand::Info => {
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
                            write_response(&mut stream, RespValue::BulkString(response).encode())
                                .await;
                        }
                        RedisCommand::Replconf => {
                            write_response(
                                &mut stream,
                                RespValue::SimpleString("OK".to_string()).encode(),
                            )
                            .await;
                        }
                        RedisCommand::Psync => {
                            let response = RespValue::SimpleString(format!(
                                "FULLRESYNC {} 0",
                                stream_info.lock().await.id
                            ))
                            .encode();
                            write_response(&mut stream, response).await;

                            full_resync = true;
                        }
                        _ => break,
                    }
                }
                None => {
                    write_response(
                        &mut stream,
                        RespValue::SimpleString("Invalid command".to_string()).encode(),
                    )
                    .await
                }
            }
        }
    }

    pub async fn handle_replica(mut stream: TcpStream, store: Arc<Mutex<Store>>) {
        loop {
            let cmd_info = match Self::parse_command(&mut stream).await {
                Ok(cmd_info) => cmd_info,
                Err(_) => {
                    continue;
                }
            };

            match cmd_info.to_command() {
                Some(command) => match command {
                    RedisCommand::Set(key, value) => {
                        store.lock().await.set(key, value);
                    }
                    _ => {}
                },
                None => {
                    write_response(
                        &mut stream,
                        RespValue::SimpleString("Invalid command".to_string()).encode(),
                    )
                    .await
                }
            }
        }
    }
}

fn unpack_bulk_str(value: RespValue) -> Option<String> {
    match value {
        RespValue::BulkString(s) => Some(s),
        _ => None,
    }
}

pub async fn write_response(stream: &mut TcpStream, response: String) {
    let _ = stream.write_all(response.as_bytes()).await;
}
