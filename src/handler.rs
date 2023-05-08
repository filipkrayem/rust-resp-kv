use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{
    resp::{Command, RESPString},
    store::Store,
};

pub struct CommandHandler<'a> {
    store: &'a Arc<RwLock<Store>>,
}

impl<'a> CommandHandler<'a> {
    pub fn new(store: &'a Arc<RwLock<Store>>) -> Self {
        Self { store }
    }

    pub async fn parse_command(
        &self,
        input: Option<RESPString>,
    ) -> Result<RESPString, anyhow::Error> {
        if let Some(value) = input {
            let (command, args) = value.to_command()?;
            let response = match command {
                Command::Ping => RESPString::SimpleString("PONG".to_string()),
                Command::Echo => args[0].clone(),
                Command::Unknown => RESPString::Error("Unknown command".to_string()),
                Command::Get => self.handle_get(args.first()).await,
                Command::Set => self.handle_set(args.first(), args.get(1)).await,
            };
            Ok(response)
        } else {
            Err(anyhow::anyhow!("No input"))
        }
    }

    async fn handle_get(&self, key: Option<&RESPString>) -> RESPString {
        if let Some(key) = key {
            if let Some(value) = self.store.read().await.get(key.to_string().as_str()) {
                RESPString::BulkString(value.clone())
            } else {
                RESPString::null_reply()
            }
        } else {
            RESPString::null_reply()
        }
    }

    async fn handle_set(&self, key: Option<&RESPString>, value: Option<&RESPString>) -> RESPString {
        if let Some(key) = key {
            if let Some(value) = value {
                self.store
                    .write()
                    .await
                    .set(key.to_string(), value.to_string());

                RESPString::ok_reply()
            } else {
                RESPString::null_reply()
            }
        } else {
            RESPString::null_reply()
        }
    }
}
