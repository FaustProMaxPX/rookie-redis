use bytes::Buf;
use std::io::Cursor;
use atoi::atoi;

#[derive(Clone, Debug)]
pub enum Frame {
    Simple(String),
    Error(String),
    Array(Vec<Frame>),
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Other(String),
}

impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match Self::get_sign(src)? {
            b'+' | b'-' => {
                Self::get_line(src)?;
                Ok(())
            },
            b'*' => {
                let len = Self::get_line(src)?;
                let len: u64 = atoi::<u64>(len).ok_or_else(|| Error::from("invalid frame type"))?;                                                                                                                                                  
                for _ in 0..len {
                    Self::check(src)?;
                }
                Ok(())
            }
            _ => Err(Error::Other("invalid frame type".to_string())),
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match Self::get_sign(src)? {
            b'+' => {
                let line = Self::get_line(src)?;
                let string = String::from_utf8_lossy(line).to_string();
                Ok(Frame::Simple(string))
            }
            b'-' => {
                let line = Self::get_line(src)?;
                let string = String::from_utf8_lossy(line).to_string();
                Ok(Frame::Error(string))
            }
            b'*' => {
                let line = Self::get_line(src)?;
                let len: u64 = atoi::<u64>(line).ok_or_else(|| Error::from("invalid frame type"))?;                                                                                                                                                  
                let mut arr = vec![];
                for _ in 0..len {
                    let frame = Self::parse(src)?;
                    arr.push(frame);
                }
                Ok(Frame::Array(arr))
            }
            _ => Err(Error::Incomplete),
        }
    }

    fn get_line<'a>(src: &'a mut Cursor<&[u8]>) -> Result<&'a [u8], Error> {
        let start = src.position() as usize;
        let end = src.get_ref().len() - 1;

        for i in start..end {
            if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
                src.set_position((i + 2) as u64);
                return Ok(&src.get_ref()[start..i]);
            }
        }
        Err(Error::Incomplete)
    }

    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Frame::Simple(s) => format!("+{}\r\n", s).into_bytes(),
            Frame::Error(s) => format!("-{}\r\n", s).into_bytes(),
            Frame::Array(arr) => {
                let len = arr.len();
                let mut bytes = format!("*{}\r\n", len).into_bytes();
                for frame in arr {
                    bytes.append(&mut frame.into_bytes());
                }
                bytes
            }
        }
    }

    pub fn into_simple(msg: &str) -> Frame {
        Frame::Simple(msg.to_string())
    }

    fn get_sign(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
        if !src.has_remaining() {
            return Err(Error::Incomplete);
        }
        Ok(src.get_u8())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::Other(value.to_string())
    }
}
