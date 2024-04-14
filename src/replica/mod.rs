use crate::{
    connection::Connection,
    message::Message,
    stream::{StreamInfo, StreamType},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    task::JoinHandle,
};

pub mod connection;
pub mod handshake;
pub mod message;

#[derive(Debug)]
pub struct ReplicaCommand {
    pub message: Message,
}
#[derive(Debug)]
pub struct ReplicaHandle {
    pub sender: Sender<ReplicaCommand>,
    pub receiver: Receiver<ReplicaCommand>,
}

pub fn replicate_channel(mut connection: Connection) -> (ReplicaHandle, JoinHandle<()>) {
    let (tx_res, mut rx) = mpsc::channel::<ReplicaCommand>(32);
    let (_tx, rx_res) = mpsc::channel::<ReplicaCommand>(32);
    let handle = tokio::spawn(async move {
        loop {
            dbg!("wait for response");
            while let Some(replica_command) = rx.recv().await {
                _ = connection.write_message(replica_command.message).await;
            }
        }
    });
    (
        ReplicaHandle {
            sender: tx_res,
            receiver: rx_res,
        },
        handle,
    )
}

pub async fn should_replicate(stream_info: &Arc<Mutex<StreamInfo>>) -> bool {
    let stream_info = stream_info.lock().await;
    match stream_info.role {
        StreamType::Master => false,
        StreamType::Replica(_) => true,
    }
}

pub fn get_master_socket_addr(stream_info: &StreamInfo) -> Option<SocketAddr> {
    match stream_info.role {
        StreamType::Master => None,
        StreamType::Replica(socket_addr) => Some(socket_addr),
    }
}
