use crate::message::Message;
use anyhow::Result;
use bytes::{Buf, BytesMut};
use std::collections::VecDeque;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Connection {
    pub stream: TcpStream,
    pub cache: VecDeque<Message>,
}

impl Connection {
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

    pub async fn read_message(&mut self) -> Option<Message> {
        if self.cache.is_empty() {
            self.read_stream().await;
        }
        self.cache.pop_front()
    }

    async fn read_stream(&mut self) {
        let mut buffer = BytesMut::with_capacity(512);
        if let Ok(length) = self.stream.read_buf(&mut buffer).await {
            let mut index = 0;
            while index < length {
                if let Ok((message, offset)) = Message::decode(buffer.clone()) {
                    self.cache.push_back(message);
                    buffer.advance(offset);
                    index += offset;
                } else {
                    println!("Invalid data structure");
                    break;
                }
            }
        }
    }
}
