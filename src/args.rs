use clap::Parser;
use std::net::IpAddr;

#[derive(Parser, Debug)]
pub struct CliArgs {
    #[arg(default_value = "127.0.0.1")]
    #[clap(short, long)]
    pub address: IpAddr,

    #[clap(short, long, default_value = "6379")]
    pub port: u16,

    #[clap(short, long = "replicaof", value_names = &["MASTER_HOST", "MASTER_PORT"], num_args = 2)]
    pub replica: Option<Vec<String>>,

    #[clap(long)]
    pub dir: Option<String>,

    #[clap(long)]
    pub dbfilename: Option<String>,
}
