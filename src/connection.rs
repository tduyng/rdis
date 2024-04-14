use std::collections::VecDeque;

use tokio::net::TcpStream;

use crate::protocol::parser::Message;

pub struct Connection {
    pub stream: TcpStream,
    pub read_cache: VecDeque<Message>
}