use crate::{args::CliArgs, protocol::rdb::RdbConfig, replica::ReplicaHandle, utils::random_sha1_hex};
use core::fmt;
use std::{
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs},
};
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Master,
    Replica(SocketAddr),
}

impl fmt::Display for StreamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Master => "master",
            Self::Replica(_) => "slave",
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug)]
pub struct StreamInfo {
    pub role: StreamType,
    pub id: String,
    pub offset: u16,
    pub socket_addr: SocketAddr,
    pub repl_handles: Mutex<Vec<ReplicaHandle>>,
    pub config: Mutex<RdbConfig>,
}

#[derive(Debug, Clone)]
pub struct StreamData {
    pub data: HashMap<String, String>,
}

impl StreamData {
    pub fn flatten(&self) -> Vec<String> {
        let mut result = Vec::with_capacity(self.data.len() * 2);
        for (key, value) in self.data.iter() {
            result.push(key.clone());
            result.push(value.clone());
        }
        result
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct StreamId {
    pub ms: u64,
    pub seq: u64,
}

impl From<&str> for StreamId {
    fn from(value: &str) -> Self {
        let (ms_str, seq_str) = value.split_once('-').unwrap_or(("0", "0"));
        let ms = ms_str.parse::<u64>().expect("Unable to parse ms value from string");
        let seq = seq_str.parse::<u64>().expect("Unable to parse seq value from string");
        Self { ms, seq }
    }
}

impl std::fmt::Display for StreamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.ms, self.seq)
    }
}

#[derive(Debug)]
pub struct Stream {
    pub entries: Vec<(StreamId, StreamData)>,
}

impl Stream {
    pub fn new(entries: Vec<(StreamId, StreamData)>) -> Self {
        Self { entries }
    }

    pub fn empty() -> Self {
        Self { entries: Vec::new() }
    }
}

impl Default for Stream {
    fn default() -> Self {
        Self::empty()
    }
}

impl StreamInfo {
    pub fn new(args: &CliArgs) -> Self {
        let role = if let Some(addr) = parse_replication_addr(args) {
            StreamType::Replica(addr)
        } else {
            StreamType::Master
        };
        let address = args.address;
        let port = args.port;
        let socket_addr = SocketAddr::new(address, port);

        Self {
            role,
            id: random_sha1_hex(),
            offset: 0,
            socket_addr,
            repl_handles: Mutex::new(Vec::new()),
            config: Mutex::new(RdbConfig::new()),
        }
    }

    pub async fn count_replicas(&self) -> usize {
        self.repl_handles.lock().await.len()
    }
}

fn parse_replication_addr(args: &CliArgs) -> Option<SocketAddr> {
    let addrr = match &args.replica {
        Some(replica_args) => {
            let server = format!("{}:{}", &replica_args[0], &replica_args[1].parse::<u16>().unwrap());

            if let Ok(socket) = server.to_socket_addrs() {
                let server: Vec<_> = socket.collect();
                let addr = server.first().expect("No valid address found");
                Some(*addr)
            } else {
                None
            }
        }
        None => None,
    };

    addrr
}
