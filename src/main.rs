use std::str;
mod resp;
use resp::parse_resp_string;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_stream(socket).await;
        });
    }
}

async fn handle_stream(mut socket: TcpStream) {
    let mut buf = [0; 512];
    loop {
        let _n = match socket.read(&mut buf).await {
            // socket closed
            Ok(n) if n == 0 => return,
            Ok(n) => n,
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                return;
            }
        };

        let parsed = parse_resp_string(str::from_utf8(&buf).unwrap().trim_matches(char::from(0)));
        println!("buf: {:#?}", parsed);
        println!("command: {:#?}", parsed.to_command());

        let reply = "+Hello World\r\n";
        // Write the data back
        if let Err(e) = socket.write_all(reply.as_bytes()).await {
            eprintln!("failed to write to socket; err = {:?}", e);
            return;
        }
    }
}
