use crate::utils::random_sha1_hex;
use core::fmt;
use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Master,
    Slave(SocketAddr),
}

impl fmt::Display for StreamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Master => "master",
            Self::Slave(_) => "slave",
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub role: StreamType,
    pub connected_clients: usize,
    pub id: String,
    pub offset: u16,
}

impl StreamInfo {
    pub fn new(socket_addr: Option<SocketAddr>) -> Self {
        let role = if let Some(addr) = socket_addr {
            StreamType::Slave(addr)
        } else {
            StreamType::Master
        };

        Self {
            role,
            connected_clients: 0,
            id: random_sha1_hex(),
            offset: 0,
        }
    }
}
