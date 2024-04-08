pub mod handshake;

#[derive(Debug, Clone)]
pub enum ReplicaRole {
    Master,
    Slave,
}

#[derive(Debug, Clone)]
pub struct ReplicaInfo {
    pub role: ReplicaRole,
    pub master_replid: String,
    pub master_repl_offset: u16,
}
