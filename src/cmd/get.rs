use bytes::Bytes;
use tracing::instrument;

use crate::{Connection, DbHolder, Frame, Result};

#[derive(Debug)]
pub struct Get {
    key: String,
}

impl Get {
    pub fn new(key: impl ToString) -> Get {
        Get {
            key: key.to_string(),
        }
    }

    #[instrument(skip(db, connection))]
    pub async fn execute(&self, connection: &mut Connection, db: &DbHolder) -> Result<()> {
        let value = db.get(&self.key);
        if let Some(value) = value {
            Self::write_response(connection, value).await
        } else {
            connection.write_frame(Frame::Null).await
        }
    }

    pub fn get_frame(key: &str) -> Frame {
        let data = Bytes::copy_from_slice(key.as_bytes());
        Frame::Array(vec![Frame::into_simple("get"), Frame::Bulk(data)])
    }

    async fn write_response(connection: &mut Connection, value: Bytes) -> Result<()> {
        let data = Frame::Bulk(value);
        let resp = Frame::Array(vec![data]);
        connection.write_frame(resp).await?;
        Ok(())
    }
}
