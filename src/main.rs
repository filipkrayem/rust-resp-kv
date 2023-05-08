mod handler;
mod resp;
use tokio::net::{TcpListener, TcpStream};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_stream(socket).await.unwrap();
        });
    }
}

async fn handle_stream(socket: TcpStream) -> Result<(), anyhow::Error> {
    let mut connection = resp::RespConnection::new(socket);

    loop {
        let value = connection.read_value().await?;
        match handler::CommandHandler::parse_command(value) {
            Ok(value) => connection.write_value(value).await?,
            Err(_) => break,
        }
    }
    Ok(())
}
