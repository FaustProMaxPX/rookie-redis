use crate::Frame;
use std::fmt::Display;
use std::vec;

pub struct Parser {
    frames: vec::IntoIter<Frame>,
}

#[derive(Debug)]
pub enum Error {
    EndOfStream,
    Other(String),
}

impl Parser {
    pub fn new(frame: Frame) -> Result<Parser, Error> {
        match frame {
            Frame::Array(frames) => Ok(Parser {
                frames: frames.into_iter(),
            }),
            frame => Err(format!("invalid frame type: {:?} for parser", frame).into()),
        }
    }

    fn next(&mut self) -> Result<Frame, Error> {
        self.frames.next().ok_or(Error::EndOfStream)
    }

    pub fn next_string(&mut self) -> Result<String, Error> {
        match self.next()? {
            Frame::Simple(s) => Ok(s),
            Frame::Bulk(data) => Ok(String::from_utf8_lossy(&data).to_string()),
            frame => Err(format!("can't get a string from {:?}", frame).into()),
        }
    }
    pub fn check_finished(&mut self) -> Result<(), Error> {
        if self.frames.next().is_none() {
            Ok(())
        } else {
            Err("expected end of stream, but there was more data".into())
        }
    }
}
impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::Other(value)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Other(s.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
