use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub enum ReplicaMode {
    Master,
    Slave(SocketAddr),
}
