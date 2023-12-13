use crate::Result;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, BufWriter, AsyncWriteExt},
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

    pub async fn read_frame(&mut self) -> Result<Option<String>> {
        // 1. try to parse data to a frame
        // 2. read more data from stream

        loop {
            if let Some(frame) = self.parse_frame() {
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

    fn parse_frame(&self) -> Option<String> {
        if self.buf.is_empty() {
            return None;
        }
        // TODO:
        Some(String::from_utf8_lossy(&self.buf[..]).to_string())
    }

    pub async fn write_frame(&mut self, frame: &str) -> Result<()> {
        self.stream.write_all(frame.as_bytes()).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
