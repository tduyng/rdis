pub mod handshake;

#[derive(Debug, Clone)]
pub enum StreamType {
    Master,
    Slave,
}

#[derive(Debug, Clone)]
pub struct ReplicaInfo {
    pub role: StreamType,
    pub repl_id: String,
    pub repl_offset: u16,
}
