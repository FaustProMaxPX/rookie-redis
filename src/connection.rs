use std::io::Cursor;

use crate::{Frame, Result};
use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

const MAX_BUF_SIZE: usize = 1024;

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buf: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            buf: BytesMut::with_capacity(MAX_BUF_SIZE),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        // 1. try to parse data to a frame
        // 2. read more data from stream

        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buf).await? {
                if self.buf.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        use crate::frame::Error::Incomplete;
        if self.buf.is_empty() {
            return Ok(None);
        }
        let mut src = Cursor::new(&self.buf[..]);
        match Frame::check(&mut src) {
            Ok(_) => {
                // `check` will have moved cursor util end of frame. 
                // we can get the length of the frame by cursor's position
                let len = src.position();
                // reset the position then parse the frame
                src.set_position(0);
                let frame = Frame::parse(&mut src)?;
                // discard the used data
                self.buf.advance(len as usize);
                Ok(Some(frame))
            }
            Err(Incomplete) => Ok(None),
            Err(_) => Err("invalid frame".into()),
        }
    }

    pub async fn write_frame(&mut self, frame: Frame) -> Result<()> {
        self.stream.write_all(&frame.into_bytes()).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
