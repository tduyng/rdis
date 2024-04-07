use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(_) => {
                println!("Accepted new connection")
            }
            Err(e) => {
                println!("Error: {}", e)
            }
        }
    }
}
