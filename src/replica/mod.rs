use crate::{
    connection::Connection,
    message::Message,
    stream::{StreamInfo, StreamType},
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
    time::timeout,
};

pub mod connection;
pub mod handler;
pub mod handshake;
pub mod message;

#[derive(Debug)]
pub struct ReplicaCommand {
    pub message: Message,
    pub timeout: Option<Duration>,
}

impl ReplicaCommand {
    pub fn new(message: Message, timeout: Option<Duration>) -> Self {
        Self { message, timeout }
    }
}

#[derive(Debug)]
pub struct ReplicaResponse {
    pub expired: bool,
}

impl ReplicaResponse {
    pub fn received() -> Self {
        Self { expired: false }
    }

    pub fn expired() -> Self {
        Self { expired: true }
    }
}
#[derive(Debug)]
pub struct ReplicaHandle {
    pub sender: Sender<ReplicaCommand>,
    pub receiver: Receiver<ReplicaResponse>,
}

pub fn replicate_channel(mut connection: Connection) -> (ReplicaHandle, JoinHandle<()>) {
    let (tx_res, mut rx) = mpsc::channel::<ReplicaCommand>(32);
    let (tx, rx_res) = mpsc::channel::<ReplicaResponse>(32);
    let handle = tokio::spawn(async move {
        loop {
            while let Some(replica_command) = rx.recv().await {
                _ = connection.write_message(replica_command.message).await;

                if let Some(duration) = replica_command.timeout {
                    let is_timeout = timeout(duration, connection.read_message()).await.is_err();
                    let response = if is_timeout {
                        ReplicaResponse::expired()
                    } else {
                        ReplicaResponse::received()
                    };
                    _ = tx.send(response).await;
                }
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

pub async fn should_replicate(stream_info: &Arc<StreamInfo>) -> bool {
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
