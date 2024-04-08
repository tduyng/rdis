pub mod handshake;

#[derive(Debug, Clone)]
pub enum StreamType {
    Master,
    Slave,
}

#[derive(Debug, Clone)]
pub struct ReplicaInfo {
    pub stream_type: StreamType,
    pub master_replid: String,
    pub master_repl_offset: u16,
}
