use std::str::Chars;

use anyhow::{Error, Result};
use bytes::BytesMut;
use tokio::net::TcpStream;

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
    pub fn to_command(&self) -> Result<String, anyhow::Error> {
        match self {
            RESPString::Array(array) => {
                let command = array.first().unwrap();
                command.to_command()
            }
            RESPString::BulkString(string) => Ok(string.to_owned()),
            _ => Err(Error::msg("Invalid command")),
        }
    }
    // TODO: copilot code, check for correctness
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
}

pub fn parse_resp_string(string: &str) -> RESPString {
    let chars = string.chars();
    let iter = chars.into_iter();
    let first_char: char = iter.clone().next().unwrap();

    let string_type = match first_char {
        '+' => RESPString::SimpleString(iter.collect()),
        '-' => RESPString::Error(iter.collect()),
        ':' => {
            let num = iter.skip(1).collect::<String>().parse::<i64>().unwrap();
            RESPString::Integer(num)
        }
        '$' => {
            let collect = iter.clone().collect::<String>();

            let result_string = collect.chars().into_iter().skip(2).collect::<String>();
            RESPString::BulkString(result_string.to_owned())
        }
        '*' => parse_bulk_string(iter),
        _ => panic!("Invalid RESP string"),
    };

    string_type
}

fn parse_bulk_string(iter: Chars) -> RESPString {
    let collect = iter.clone().collect::<String>();
    let mut split = collect.split("\r\n").collect::<Vec<&str>>();

    split.remove(0); // remove the first element which is the number of elements in the array

    let new = combine_strings_with_delimiters(&split, &DELIMITERS);

    let mut array: Vec<RESPString> = Vec::with_capacity(2 as usize);

    for str in new {
        array.push(parse_resp_string(&str));
    }

    RESPString::Array(array)
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

enum Commands {
    Ping,
    Echo,
}

impl Commands {}

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

    pub fn read_stream(&mut self) -> Result<Option<RESPString>> {
        todo!()
    }

    pub fn write_stream(&mut self) -> Result<()> {
        todo!()
    }
}
