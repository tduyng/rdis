use crate::stream::{StreamInfo, StreamType};
use std::net::SocketAddr;

pub mod handshake;

pub fn should_replicate(stream_info: &StreamInfo) -> bool {
    match stream_info.role {
        StreamType::Master => false,
        StreamType::Slave(_) => true,
    }
}

pub fn get_master_socket_addr(stream_info: &StreamInfo) -> Option<SocketAddr> {
    match stream_info.role {
        StreamType::Master => None,
        StreamType::Slave(socket_addr) => Some(socket_addr),
    }
}
