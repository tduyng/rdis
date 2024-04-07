use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Server listening on 127.0.0.1:6379");

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handle_connection(socket));
    }
}

async fn handle_connection(mut socket: TcpStream) -> Result<()> {
    println!("Accepted new connection");

    loop {
        let mut buffer = [0; 1024];
        let bytes_read = socket.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        let response = "+PONG\r\n";
        socket.write_all(response.as_bytes()).await?;
    }
    Ok(())
}
