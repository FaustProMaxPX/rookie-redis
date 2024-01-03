use std::{collections::HashMap, sync::Mutex, time::Duration};
use crate::Result;
use bytes::Bytes;

pub struct DbHolder {
    holder: Mutex<Database>,
}

struct Database {
    entries: HashMap<String, Bytes>,
}

impl DbHolder {
    pub fn new() -> DbHolder {
        DbHolder {
            holder: Mutex::new(Database::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        self.holder.lock().unwrap().entries.get(key).cloned()
    }

    pub fn set(&self, key: String, value: Bytes, expiration: Option<Duration>) -> Result<()> {
        self.holder.lock().unwrap().entries.insert(key, value);
        Ok(()) 
    }
}

impl Database {
    fn new() -> Database {
        Database {
            entries: HashMap::new(),
        }
    }
}
