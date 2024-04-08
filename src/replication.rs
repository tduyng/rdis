pub mod handshake;

#[derive(Debug, Clone)]
pub enum ReplicaMode {
    Master,
    Slave,
}
