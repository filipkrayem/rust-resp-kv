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

        if let Some(value) = value {
            let (command, args) = value.to_command()?;
            let response = match command {
                resp::Command::Ping => resp::RESPString::SimpleString("PONG".to_string()),
                resp::Command::Echo => args[0].clone(),
                resp::Command::Unknown => resp::RESPString::Error("Unknown command".to_string()),
            };
            println!("response: {:#?}", response.encode());
            connection.write_value(response).await?;
        } else {
            break;
        }
    }

    Ok(())
}
