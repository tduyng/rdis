use crate::message::Message;

#[derive(Debug)]
pub enum ReplicaMessage {
    RdbFile(String),
    Response(Message),
}

impl ReplicaMessage {
    pub fn is_rdb_file(&self) -> bool {
        matches!(self, Self::RdbFile(_))
    }

    pub fn is_response(&self) -> bool {
        matches!(self, Self::Response(_))
    }
}
