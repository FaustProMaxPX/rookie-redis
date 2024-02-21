use crate::{Frame, Result};

mod blocking_client;
pub use blocking_client::BlockingClient;

fn parse_frame(frame: Frame) -> Result<Option<Vec<u8>>> {
    match frame {
        Frame::Simple(s) => Ok(Some(s.into_bytes())),
        Frame::Bulk(s) => Ok(Some(s.to_vec())),
        Frame::Null => Ok(None),
        Frame::Error(s) => Err(s.into()),
        Frame::Integer(s) => Ok(Some(s.to_le_bytes().to_vec())),
        Frame::Array(arr) => {
            let mut bytes = Vec::new();
            for frame in arr {
                let b = parse_frame(frame)?;
                if let Some(mut b) = b {
                    bytes.append(&mut b);
                }
            }
            Ok(Some(bytes))
        }
    }
}
