use crate::{
    command::{Command, XAddArgs, XRangArgs, XReadArgs},
    connection::Connection,
    message::Message,
    protocol::rdb::Rdb,
    replica::{replicate_channel, ReplicaCommand},
    store::{Entry, EntryValue, Store},
    stream::{StreamId, StreamInfo},
};
use anyhow::{Ok, Result};
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::Mutex;

pub struct Handler {}

impl Handler {
    pub async fn handle_stream(
        mut connection: Connection,
        store: Arc<Mutex<Store>>,
        stream_info: Arc<StreamInfo>,
    ) -> Result<()> {
        let mut full_resync = false;

        loop {
            if full_resync {
                process_full_resync(connection, &stream_info).await?;
                return Ok(());
            }
            if let Some(message) = connection.read_message().await {
                let cmd_info = Message::parse_command(message).await?;

                match cmd_info.to_command() {
                    Some(command) => {
                        let command_clone = command.clone();
                        match command {
                            Command::Ping => process_ping(&mut connection).await?,
                            Command::Echo(message) => process_echo(&mut connection, message).await?,
                            Command::Get(key) => process_get(&mut connection, &store, key).await?,
                            Command::Set(key, entry) => {
                                process_set(&mut connection, &store, &stream_info, &command_clone, key, entry).await?
                            }
                            Command::Info => process_info(&mut connection, &stream_info).await?,
                            Command::Replconf(_) => process_replconf(&mut connection).await?,
                            Command::Psync => {
                                process_psync(&mut connection, &stream_info).await?;
                                full_resync = true;
                            }
                            Command::Wait(timeout) => {
                                process_wait(&mut connection, &store, &stream_info, timeout).await?
                            }
                            Command::Config(action, key) => {
                                process_config(&mut connection, &stream_info, action, key).await?
                            }
                            Command::Keys(pattern) => process_keys(&mut connection, &store, pattern).await?,
                            Command::Type(key) => process_type(&mut connection, &store, key).await?,
                            Command::XAdd(args) => process_xadd(&mut connection, &store, args).await?,
                            Command::XRange(args) => process_xrange(&mut connection, &store, args).await?,
                            Command::XRead(args) => process_xread(&mut connection, &store, args).await?,
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
        Ok(())
    }
}

async fn process_ping(connection: &mut Connection) -> Result<()> {
    connection.write_message(Message::Simple("PONG".to_string())).await
}

async fn process_echo(connection: &mut Connection, message: String) -> Result<()> {
    connection.write_message(Message::Simple(message)).await
}

async fn process_get(connection: &mut Connection, store: &Arc<Mutex<Store>>, key: String) -> Result<()> {
    let response = if let Some(entry) = store.lock().await.get_kv(&key) {
        format!("${}\r\n{}\r\n", entry.value.len(), entry.value)
    } else {
        "$-1\r\n".to_string()
    };
    connection.write_bytes(response.as_bytes()).await
}

async fn process_set(
    connection: &mut Connection,
    store: &Arc<Mutex<Store>>,
    stream_info: &Arc<StreamInfo>,
    command: &Command,
    key: String,
    entry: Entry,
) -> Result<()> {
    store.lock().await.set_kv(key, entry)?;
    connection.write_message(Message::Simple("OK".to_string())).await?;

    for replication in stream_info.repl_handles.lock().await.iter_mut() {
        if let Some(replica_command) = Command::to_replica_command(command) {
            replication.sender.send(replica_command).await?;
        }
    }
    Ok(())
}

async fn process_info(connection: &mut Connection, stream_info: &Arc<StreamInfo>) -> Result<()> {
    let response = format!(
        "# Replication\n\
        role:{}\n\
        connected_clients:{}\n\
        master_replid:{}\n\
        master_repl_offset:{}\n\
        ",
        stream_info.role,
        stream_info.count_replicas().await,
        stream_info.id,
        stream_info.offset
    );
    connection.write_message(Message::Bulk(response)).await
}

async fn process_replconf(connection: &mut Connection) -> Result<()> {
    connection.write_message(Message::Simple("OK".to_string())).await
}

async fn process_psync(connection: &mut Connection, stream_info: &Arc<StreamInfo>) -> Result<()> {
    let message = Message::Simple(format!("FULLRESYNC {} 0", stream_info.id));
    connection.write_message(message).await
}

async fn process_full_resync(mut connection: Connection, stream_info: &Arc<StreamInfo>) -> Result<()> {
    let stream_info = stream_info.clone();
    {
        let empty_rdb = Rdb::get_empty();
        let _ = connection.write_bytes(&empty_rdb).await;
    }
    let (repl_handle, handle) = replicate_channel(connection);
    {
        stream_info.repl_handles.lock().await.push(repl_handle);
    }
    handle.await?;
    Ok(())
}

async fn process_wait(
    connection: &mut Connection,
    store: &Arc<Mutex<Store>>,
    stream_info: &Arc<StreamInfo>,
    timeout: u64,
) -> Result<()> {
    let mut count = 0;

    if store.lock().await.is_empty() {
        let num_replicas = stream_info.repl_handles.lock().await.len();
        connection.write_message(Message::Int(num_replicas as isize)).await
    } else {
        for replica in stream_info.repl_handles.lock().await.iter_mut() {
            let message = Message::Array(vec![
                Message::Bulk("REPLCONF".to_string()),
                Message::Bulk("GETACK".to_string()),
                Message::Bulk("*".to_string()),
            ]);
            replica
                .sender
                .send(ReplicaCommand::new(message, Some(Duration::from_millis(timeout))))
                .await?;
        }

        for replica in stream_info.repl_handles.lock().await.iter_mut() {
            let response = replica.receiver.recv().await.unwrap();
            if !response.expired {
                count += 1;
            }
        }
        connection.write_message(Message::Int(count)).await
    }
}

async fn process_config(
    connection: &mut Connection,
    stream_info: &Arc<StreamInfo>,
    action: String,
    key: String,
) -> Result<()> {
    match action.to_lowercase().as_str() {
        "get" => {
            let config_value = stream_info.config.lock().await.get_value(&key);
            if let Some(value) = config_value {
                let message = Message::Array(vec![Message::Bulk(key), Message::Bulk(value)]);
                connection.write_message(message).await?
            } else {
                connection
                    .write_message(Message::Simple("Value not found".to_string()))
                    .await?
            }
        }
        _ => {
            connection
                .write_message(Message::Simple("Unsupported config action".to_string()))
                .await?
        }
    }
    Ok(())
}

async fn process_keys(connection: &mut Connection, store: &Arc<Mutex<Store>>, pattern: String) -> Result<()> {
    if pattern == "*" {
        let keys = store
            .lock()
            .await
            .keys()
            .into_iter()
            .map(Message::Bulk)
            .collect::<Vec<_>>();
        connection.write_message(Message::Array(keys)).await
    } else {
        connection
            .write_message(Message::Simple("Unsupported pattern".to_string()))
            .await
    }
}

async fn process_type(connection: &mut Connection, store: &Arc<Mutex<Store>>, key: String) -> Result<()> {
    let store = store.lock().await;
    let item = store.get_store_item(&key);

    let value_type = match item {
        Some(x) => x.value_type(),
        None => "none".to_string(),
    };

    connection.write_message(Message::Simple(value_type)).await
}

async fn process_xadd(connection: &mut Connection, store: &Arc<Mutex<Store>>, args: XAddArgs) -> Result<()> {
    let mut store = store.lock().await;
    let stream_id = store.generate_stream_id(&args.key, &args.id).unwrap();
    if let Err(err) = store.validate_stream(&args.key, &stream_id) {
        return connection.write_message(Message::Error(err.to_string())).await;
    }

    store.set_stream(args.key, stream_id.clone(), args.data)?;
    connection.write_message(Message::Bulk(stream_id)).await
}

async fn get_xrange_start(id: &str) -> Option<StreamId> {
    if id == "-" {
        return None;
    }
    let (ms, seq) = id.split_once('-').unwrap_or((id, "0"));
    let ms = ms.parse().expect("Unable to parse ms");
    let seq = seq.parse().expect("Unable to parse seq");
    Some(StreamId { ms, seq })
}

async fn get_xrange_end(store: &Arc<Mutex<Store>>, key: &str, id: &str) -> Option<StreamId> {
    if id == "+" {
        return None;
    }
    let mut store = store.lock().await;
    let template = if id.contains('-') {
        id.to_string()
    } else {
        format!("{}-*", id)
    };
    let id = store.generate_stream_id(key, &template)?;
    Some(StreamId::from(id.as_str()))
}

async fn process_xrange(connection: &mut Connection, store: &Arc<Mutex<Store>>, args: XRangArgs) -> Result<()> {
    let start = get_xrange_start(&args.start).await;
    let end = get_xrange_end(store, &args.key, &args.end).await;

    let stream = match store.lock().await.get_stream_range(&args.key, start, end) {
        Some(stream) => stream,
        None => {
            return connection
                .write_message(Message::Error("ERR Unable to parse start ID".to_string()))
                .await;
        }
    };

    let message_content: Vec<_> = stream
        .entries
        .iter()
        .map(|(id, data)| {
            Message::Array(vec![
                Message::Bulk(id.to_string()),
                Message::Array(data.flatten().iter().map(|x| Message::Bulk(x.clone())).collect()),
            ])
        })
        .collect();

    connection.write_message(Message::Array(message_content)).await?;

    Ok(())
}

async fn process_xread(connection: &mut Connection, store: &Arc<Mutex<Store>>, args: XReadArgs) -> Result<()> {
    let mut messages: Vec<Message> = Vec::new();

    let mut requests: Vec<(String, StreamId)> = Vec::new();
    for i in 0..args.requests.len() {
        let request = args.requests.get(i).unwrap();
        let (key, id) = request;
        let mut store = store.lock().await;
        let stream_id = if id == "$" {
            let result = store.get_lastest_stream(key);
            match result {
                Some(val) => val.clone(),
                None => StreamId::from("0-0"),
            }
        } else {
            StreamId::from(id.as_str())
        };

        requests.push((key.clone(), stream_id));
    }

    loop {
        messages.clear();

        for request in &requests {
            let (key, id) = request;
            let stream = store.lock().await.get_stream_after_id(key, id);
            if stream.is_none() {
                continue;
            }
            let stream = stream.unwrap();
            messages.push(Message::Array(vec![Message::Bulk(key.clone()), stream.to_message()]));
        }

        if messages.len() == args.requests.len() {
            break;
        } else if let Some(timeout) = args.block {
            let timed_out = SystemTime::now() > timeout;
            if !args.wait && timed_out {
                break;
            } else {
                tokio::time::sleep(Duration::from_micros(50)).await;
            }
        } else {
            break;
        }
    }

    if messages.is_empty() {
        connection.write_message(Message::Null).await
    } else {
        connection.write_message(Message::Array(messages)).await
    }
}
