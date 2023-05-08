use crate::resp::{Command, RESPString};

pub struct CommandHandler;

impl CommandHandler {
    pub fn parse_command(input: Option<RESPString>) -> Result<RESPString, anyhow::Error> {
        if let Some(value) = input {
            let (command, args) = value.to_command()?;
            let response = match command {
                Command::Ping => RESPString::SimpleString("PONG".to_string()),
                Command::Echo => args[0].clone(),
                Command::Unknown => RESPString::Error("Unknown command".to_string()),
                Command::Get => RESPString::BulkString("Get".to_string()),
                Command::Set => RESPString::BulkString("Set".to_string()),
            };
            Ok(response)
        } else {
            Err(anyhow::anyhow!("No input"))
        }
    }

    fn handle_get() {}

    fn handle_set() {}
}
