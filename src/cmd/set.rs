use std::{time::Duration, sync::Arc};

use bytes::Bytes;

use crate::{DbHolder, Connection, Result, Frame};

pub struct Set {
    key: String,
    value: Bytes,
    expiration: Option<Duration>,
}

impl Set {
    pub fn new(key: impl ToString, value: Bytes, expiration: Option<Duration>) -> Set {
        Set {
            key: key.to_string(),
            value,
            expiration,
        }
    }

    pub async fn execute(self, db: &Arc<DbHolder>, connection: &mut Connection) -> Result<()> {
        db.set(self.key, self.value, self.expiration)?;
        connection.write_frame(Frame::into_simple("OK")).await 
    }

    pub fn get_frame(key: &str, value: Bytes, expiration: Option<Duration>) -> Frame {
        let mut frame = vec![Frame::into_simple("set"), Frame::into_simple(key), Frame::Bulk(value)]; 
        if let Some(exp) = expiration {
            frame.push(Frame::Integer(exp.as_secs() as i64))
        }
        Frame::Array(frame)
    }
}
