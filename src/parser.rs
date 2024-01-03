use bytes::Bytes;

use crate::Frame;
use std::fmt::Display;
use std::vec;

pub struct Parser {
    frames: vec::IntoIter<Frame>,
}

#[derive(Debug)]
pub enum ParseError {
    EndOfStream,
    Other(String),
}

impl Parser {
    pub fn new(frame: Frame) -> Result<Parser, ParseError> {
        match frame {
            Frame::Array(frames) => Ok(Parser {
                frames: frames.into_iter(),
            }),
            frame => Err(format!("invalid frame type: {:?} for parser", frame).into()),
        }
    }

    fn next(&mut self) -> Result<Frame, ParseError> {
        self.frames.next().ok_or(ParseError::EndOfStream)
    }

    pub fn next_string(&mut self) -> Result<String, ParseError> {
        match self.next()? {
            Frame::Simple(s) => Ok(s),
            Frame::Bulk(data) => Ok(String::from_utf8_lossy(&data).to_string()),
            frame => Err(format!("can't get a string from {:?}", frame).into()),
        }
    }

    pub fn next_bytes(&mut self) -> Result<Bytes, ParseError> {
        match self.next()? {
            Frame::Bulk(data) => Ok(data),
            frame => Err(format!("can't get a bytes from {:?}", frame).into()),
        }
    }

    pub fn next_int(&mut self) -> Result<i64, ParseError> {
        match self.next()? {
            Frame::Integer(i) => Ok(i),
            frame => Err(format!("can't get an integer from {:?}", frame).into()),
        } 
    }

    pub fn check_finished(&mut self) -> Result<(), ParseError> {
        if self.frames.next().is_none() {
            Ok(())
        } else {
            Err("expected end of stream, but there was more data".into())
        }
    }
}
impl From<String> for ParseError {
    fn from(value: String) -> Self {
        ParseError::Other(value)
    }
}

impl From<&str> for ParseError {
    fn from(s: &str) -> Self {
        ParseError::Other(s.to_string())
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseError {}
