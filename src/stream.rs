use crate::{args::CliArgs, utils::random_sha1_hex};
use core::fmt;
use std::net::{SocketAddr, ToSocketAddrs};

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Master,
    Slave(SocketAddr),
}

impl fmt::Display for StreamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Master => "master",
            Self::Slave(_) => "slave",
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub role: StreamType,
    pub connected_clients: usize,
    pub id: String,
    pub offset: u16,
    pub socket_addr: SocketAddr,
}

impl StreamInfo {
    pub fn new(args: &CliArgs) -> Self {
        let role = if let Some(addr) = parse_replication_addr(args) {
            StreamType::Slave(addr)
        } else {
            StreamType::Master
        };
        let address = args.address;
        let port = args.port;
        let socket_addr = SocketAddr::new(address, port);

        Self {
            role,
            connected_clients: 0,
            id: random_sha1_hex(),
            offset: 0,
            socket_addr,
        }
    }
}

fn parse_replication_addr(args: &CliArgs) -> Option<SocketAddr> {
    let addrr = match &args.replica {
        Some(replica_args) => {
            let server = format!(
                "{}:{}",
                &replica_args[0],
                &replica_args[1].parse::<u16>().unwrap()
            );

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
