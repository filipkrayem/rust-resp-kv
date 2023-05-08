mod handler;
mod resp;
mod store;

use std::sync::Arc;

use handler::CommandHandler;
use store::Store;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    // initialize the store with atomic reference counting and read write lock to prevent data races and ownership issues
    let store = Arc::new(RwLock::new(Store::new()));
    // initialize the command handler
    loop {
        let (socket, _) = listener.accept().await?;
        // This creates a new Arc that points to the location of the store
        let store_clone = store.clone();
        tokio::spawn(async move {
            handle_stream(socket, &store_clone).await.unwrap();
        });
    }
}

async fn handle_stream(socket: TcpStream, store: &Arc<RwLock<Store>>) -> Result<(), anyhow::Error> {
    let mut connection = resp::RespConnection::new(socket);
    let handler = handler::CommandHandler::new(store);
    loop {
        let value = connection.read_value().await?;

        match handler.parse_command(value).await {
            Ok(value) => connection.write_value(value).await?,
            Err(_) => break,
        }
    }
    Ok(())
}
