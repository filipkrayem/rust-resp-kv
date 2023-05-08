use std::str::{self, Chars};

use anyhow::Error;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug, Clone, PartialEq, Eq)]

pub enum RESPString {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RESPString>),
}

const DELIMITERS: [char; 4] = [':', '+', '-', '$'];

impl RESPString {
    //    Gets the command from the RESPString
    pub fn to_command(&self) -> Result<(Command, Vec<RESPString>), anyhow::Error> {
        match self {
            RESPString::Array(array) => {
                let command = array.first().unwrap().to_string();

                return Ok((Command::from_str(&command), array[1..].to_vec()));
            }
            _ => Err(Error::msg("Invalid command")),
        }
    }

    pub fn encode(&self) -> String {
        match self {
            RESPString::SimpleString(string) => format!("+{}\r\n", string),
            RESPString::Error(string) => format!("-{}\r\n", string),
            RESPString::Integer(num) => format!(":{}\r\n", num),
            RESPString::BulkString(string) => format!("${}\r\n{}\r\n", string.len(), string),
            RESPString::Array(array) => {
                let mut result = String::new();
                for resp_string in array {
                    result.push_str(&resp_string.encode());
                }
                result
            }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            RESPString::SimpleString(string) => string.to_string(),
            RESPString::Error(string) => string.to_string(),
            RESPString::Integer(num) => num.to_string(),
            RESPString::BulkString(string) => string.to_string(),
            RESPString::Array(array) => {
                let mut result = String::new();
                for resp_string in array {
                    result.push_str(&resp_string.to_string());
                }
                result
            }
        }
    }

    pub fn null_reply() -> RESPString {
        RESPString::BulkString("$-1\r\n".to_string())
    }

    pub fn ok_reply() -> RESPString {
        RESPString::SimpleString("+OK\r\n".to_string())
    }
}

pub fn parse_resp_string(string: &str) -> Result<RESPString, anyhow::Error> {
    let string = string.trim_matches(char::from(0));
    let chars = string.chars();
    let iter = chars.into_iter();
    let first_char: char = iter.clone().next().unwrap();

    let string_type = match first_char {
        '+' => Ok(RESPString::SimpleString(iter.collect())),
        '-' => Ok(RESPString::Error(iter.collect())),
        ':' => {
            let num = iter.skip(1).collect::<String>().parse::<i64>().unwrap();
            Ok(RESPString::Integer(num))
        }
        '$' => {
            let collect = iter.clone().collect::<String>();

            let result_string = collect.chars().into_iter().skip(2).collect::<String>();
            Ok(RESPString::BulkString(result_string.to_owned()))
        }
        '*' => parse_bulk_string(iter),
        _ => Err(Error::msg("Invalid RESP string")),
    };

    string_type
}

fn parse_bulk_string(iter: Chars) -> Result<RESPString, anyhow::Error> {
    let collect = iter.clone().collect::<String>();
    let mut split = collect.split("\r\n").collect::<Vec<&str>>();

    split.remove(0); // remove the first element which is the number of elements in the array

    let new = combine_strings_with_delimiters(&split, &DELIMITERS);

    let mut array: Vec<RESPString> = Vec::with_capacity(2 as usize);

    for str in new {
        if let Ok(resp_string) = parse_resp_string(&str) {
            array.push(resp_string);
        }
    }

    Ok(RESPString::Array(array))
}

fn combine_strings_with_delimiters(strings: &Vec<&str>, delimiters: &[char]) -> Vec<String> {
    let mut result = vec![];
    let mut current_string = String::new();
    let mut combine = false;

    for string in strings {
        if string == &"" {
            continue;
        };

        if combine {
            current_string.push_str(string);
        } else {
            current_string = string.clone().to_owned();
        }

        if delimiters.contains(&string.chars().next().unwrap()) {
            combine = true;
        } else {
            combine = false;
            result.push(current_string.clone());
            current_string.clear();
        }
    }

    if !current_string.is_empty() {
        result.push(current_string);
    }

    result
}

#[derive(Debug)]
pub enum Command {
    Ping,
    Echo,
    Get,
    Set,
    Unknown,
}

impl Command {
    pub fn from_str(string: &str) -> Command {
        match string.to_ascii_lowercase().as_ref() {
            "ping" => Command::Ping,
            "echo" => Command::Echo,
            "get" => Command::Get,
            "set" => Command::Set,
            _ => Command::Unknown,
        }
    }
}

pub struct RespConnection {
    stream: TcpStream,
    buffer: BytesMut,
}

impl RespConnection {
    pub fn new(stream: TcpStream) -> RespConnection {
        RespConnection {
            stream,
            buffer: BytesMut::with_capacity(512),
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<RESPString>, anyhow::Error> {
        loop {
            let bytes_read = self.stream.read_buf(&mut self.buffer).await?;

            if bytes_read == 0 {
                return Ok(None);
            }

            if let Ok(resp) = parse_resp_string(str::from_utf8(&self.buffer).unwrap()) {
                // println!("buf: {:#?}", resp);
                // println!("command: {:#?}", resp.to_command());

                return Ok(Some(resp));
            }
        }
    }

    pub async fn write_value(&mut self, value: RESPString) -> Result<(), anyhow::Error> {
        self.stream.write(value.encode().as_bytes()).await?;

        Ok(())
    }
}
