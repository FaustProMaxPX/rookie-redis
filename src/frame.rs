use atoi::atoi;
use bytes::{Buf, Bytes};
use lazy_static::lazy_static;
use regex::Regex;
use std::{fmt::Display, io::Cursor};

lazy_static! {
    static ref INTEGER: Regex = Regex::new(r"([+|-]?\d+)").unwrap();
}

#[derive(Clone, Debug, PartialEq)]
pub enum Frame {
    Simple(String),
    Bulk(Bytes),
    Integer(i64),
    Error(String),
    Array(Vec<Frame>),
    Null,
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
            }
            b'$' => {
                if b'-' == Self::peek_u8(src)? {
                    // skip "-1\r\n"
                    let line = Self::get_line(src)?;
                    if b"-1" != line {
                        return Err(Error::Other("invalid frame type".to_string()));
                    }
                    src.advance(4);
                } else {
                    let len = Self::get_number(src)? as usize;
                    src.advance(len + 2);
                }
                Ok(())
            }
            b':' => {
                let line = String::from_utf8_lossy(Self::get_line(src)?);
                if INTEGER.is_match(&line) {
                    Ok(())
                } else {
                    Err(Error::Other("invalid frame type".to_string()))
                }
            }
            b'*' => {
                let len = Self::get_line(src)?;
                let len: u64 = atoi::<u64>(len).ok_or_else(|| Error::from("invalid frame type"))?;
                for _ in 0..len {
                    Self::check(src)?;
                }
                Ok(())
            }
            b'_' => Ok(()),
            _ => Err(Error::Other("invalid frame type".to_string())),
        }
    }

    /// Get a frame from buffer. This method must be invoked after `Frame::check`
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
            b':' => {
                let line = String::from_utf8_lossy(Self::get_line(src)?);
                INTEGER
                    .captures(&line)
                    .and_then(|cap| cap.get(1))
                    .map_or_else(
                        || Err(Error::from("invalid frame type")),
                        |c| {
                            atoi::<i64>(c.as_str().as_bytes())
                                .map(Frame::Integer)
                                .ok_or_else(|| Error::from("invalid frame type"))
                        },
                    )
            }
            b'$' => {
                if b'-' == Self::peek_u8(src)? {
                    src.advance(4);
                    Ok(Frame::Null)
                } else {
                    let len = Self::get_number(src)? as usize;
                    let data = &src.chunk()[0..len];
                    Ok(Frame::Bulk(Bytes::copy_from_slice(data)))
                }
            }
            b'*' => {
                let line = Self::get_line(src)?;
                let len: u64 =
                    atoi::<u64>(line).ok_or_else(|| Error::from("invalid frame type"))?;
                let mut arr = vec![];
                for _ in 0..len {
                    let frame = Self::parse(src)?;
                    arr.push(frame);
                }
                Ok(Frame::Array(arr))
            }
            b'_' => Ok(Frame::Null),
            _ => Err(Error::Incomplete),
        }
    }

    fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
        if !src.has_remaining() {
            Err(Error::Incomplete)
        } else {
            Ok(src.chunk()[0])
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

    fn get_number(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
        let len = Self::get_line(src)?;
        atoi::<u64>(len).ok_or_else(|| Error::from("invalid number"))
    }

    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Frame::Simple(s) => format!("+{}\r\n", s).into_bytes(),
            Frame::Error(s) => format!("-{}\r\n", s).into_bytes(),
            Frame::Integer(value) => format!(":{}\r\n", value).into_bytes(),
            Frame::Bulk(data) => {
                let len = data.len();
                let pre = format!("${}\r\n", len).into_bytes();
                let suffix = "\r\n".as_bytes();
                [pre, data.to_vec(), suffix.to_vec()].concat()
            }
            Frame::Array(arr) => {
                let len = arr.len();
                let mut bytes = format!("*{}\r\n", len).into_bytes();
                for frame in arr {
                    bytes.append(&mut frame.into_bytes());
                }
                bytes
            }
            Frame::Null => "_\r\n".as_bytes().to_vec(),
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

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Frame::Simple(msg) => write!(f, "{}", msg),
            Frame::Error(msg) => write!(f, "{}", msg),
            Frame::Integer(value) => write!(f, "{}", value),
            Frame::Bulk(data) => write!(f, "{}", String::from_utf8_lossy(data)),
            Frame::Array(arr) => {
                write!(f, "[")?;
                for frame in arr {
                    write!(f, "{}", frame)?;
                    write!(f, ",")?;
                }
                write!(f, "]")
            }
            Frame::Null => write!(f, "null"),
        }
    }
}

#[test]
fn check_integer_frame_test() {
    let mut src: Cursor<&[u8]> = Cursor::new(b":-123\r\n");
    let result = Frame::check(&mut src);
    assert!(result.is_ok(), "error: {}", result.unwrap_err());
    src.set_position(0);
    let result = Frame::parse(&mut src);
    assert!(result.is_ok(), "error: {}", result.unwrap_err());
    assert_eq!(Frame::Integer(-123), result.unwrap());
}
