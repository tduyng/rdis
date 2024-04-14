use crate::message::Message;
use std::collections::VecDeque;
use tokio::net::TcpStream;

pub struct Connection {
    pub stream: TcpStream,
    pub read_cache: VecDeque<Message>,
}
