use crate::{
    command::Command,
    message::Message,
    replica::connection::ReplicaConnection,
    store::{Entry, Store},
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ReplicaHandler {}

impl ReplicaHandler {
    pub async fn handle_replica(mut replica_connection: ReplicaConnection, store: Arc<Mutex<Store>>) -> Result<()> {
        let mut bytes_received = 0;

        loop {
            if let Some(message) = replica_connection.get_response().await {
                let cmd_info = match Message::parse_command(message.clone()).await {
                    Ok(cmd_info) => cmd_info,
                    Err(_) => {
                        continue;
                    }
                };

                match cmd_info.to_command() {
                    Some(command) => match command {
                        Command::Set(key, entry) => process_set(&store, key, entry).await?,
                        Command::Replconf(args) => {
                            process_replconf(&mut replica_connection, args, bytes_received).await?
                        }
                        _ => {}
                    },
                    None => process_invalid_command(&mut replica_connection).await?,
                }
                let message_len = message.encode().as_bytes().len();
                bytes_received += message_len;
            } else {
                println!("Unable to get a message from the stream");
                break;
            }
        }
        Ok(())
    }
}

async fn process_set(store: &Arc<Mutex<Store>>, key: String, entry: Entry) -> Result<()> {
    store.lock().await.set_kv(key, entry)
}

async fn process_replconf(
    replica_connection: &mut ReplicaConnection,
    args: Vec<String>,
    bytes_received: usize,
) -> Result<()> {
    let command = args.first().expect("Replconf args is required").to_lowercase();
    if command == "getack" {
        let message = Message::Array(vec![
            Message::Bulk("REPLCONF".to_string()),
            Message::Bulk("ACK".to_string()),
            Message::Bulk(bytes_received.to_string()),
        ]);
        replica_connection.write_message(message).await?;
    }
    Ok(())
}

async fn process_invalid_command(replica_connection: &mut ReplicaConnection) -> Result<()> {
    replica_connection
        .write_message(Message::Simple("Invalid command".to_string()))
        .await
}
