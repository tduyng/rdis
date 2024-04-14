use crate::message::Message;
use bytes::BytesMut;
use std::collections::VecDeque;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use super::message::ReplicaMessage;
use anyhow::Result;

#[derive(Debug)]
pub struct ReplicaConnection {
    pub stream: TcpStream,
    pub cache: VecDeque<ReplicaMessage>,
}

impl ReplicaConnection {
    pub fn bind(stream: TcpStream) -> Self {
        Self {
            stream,
            cache: VecDeque::new(),
        }
    }

    pub async fn write_bytes(&mut self, data: &[u8]) -> Result<()> {
        self.stream.write_all(data).await?;
        self.stream.flush().await?;

        Ok(())
    }

    pub async fn write_message(&mut self, message: Message) -> Result<()> {
        self.write_bytes(message.encode().as_bytes()).await
    }

    pub async fn get_rdb(&mut self) -> Option<String> {
        if self.cache.is_empty() && !self.read_stream().await {
            return None;
        }

        let index = self.cache.iter().position(|x| x.is_rdb_file());

        if let Some(index) = index {
            if let ReplicaMessage::RdbFile(rdb) = self.cache.remove(index).unwrap() {
                return Some(rdb);
            }
        }
        None
    }

    pub async fn get_response(&mut self) -> Option<Message> {
        if self.cache.is_empty() && !self.read_stream().await {
            return None;
        }

        let index = self.cache.iter().position(|x| x.is_response());

        if let Some(index) = index {
            if let ReplicaMessage::Response(message) = self.cache.remove(index).unwrap() {
                return Some(message);
            }
        }
        None
    }

    async fn read_stream(&mut self) -> bool {
        let mut buffer = BytesMut::with_capacity(512);

        if let Ok(length) = self.stream.read_buf(&mut buffer).await {
            let mut index = 0;
            while index < length {
                let data = &buffer[index..];
                let message = match data[0] {
                    b'$' => {
                        index += 93; // empty rdb file length
                        ReplicaMessage::RdbFile("foobar".to_string())
                    }
                    b'+' | b'*' => {
                        if let Ok((message, offset)) = Message::decode(data.into()) {
                            index += offset;
                            ReplicaMessage::Response(message)
                        } else {
                            println!("Unsupported file format");
                            break;
                        }
                    }
                    _ => break,
                };

                self.cache.push_back(message)
            }
            true
        } else {
            false
        }
    }
}
