use std::collections::VecDeque;

use tokio::net::TcpStream;

use crate::protocol::parser::RespValue;

pub struct Connection {
    pub stream: TcpStream,
    pub read_cache: VecDeque<RespValue>
}