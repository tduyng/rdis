use crate::{
    protocol::parser::RespValue,
    stream::{StreamInfo, StreamType},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

pub mod handshake;

#[derive(Debug)]
pub struct ReplicaCommand {
    pub message: RespValue,
}
#[derive(Debug)]
pub struct ReplicaHandle {
    pub sender: Sender<ReplicaCommand>,
    pub receiver: Receiver<ReplicaCommand>,
}

pub async fn should_replicate(stream_info: &Arc<Mutex<StreamInfo>>) -> bool {
    let stream_info = stream_info.lock().await;
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
