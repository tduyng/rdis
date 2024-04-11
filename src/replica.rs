pub mod handshake;

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Master,
    Slave,
}

#[derive(Debug, Clone)]
pub struct ReplInfo {
    pub role: StreamType,
    pub master_id: String,
    pub master_offset: u16,
}
