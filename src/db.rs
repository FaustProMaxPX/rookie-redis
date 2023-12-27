use std::{collections::HashMap, sync::Mutex};

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
}

impl Database {
    fn new() -> Database {
        Database {
            entries: HashMap::new(),
        }
    }
}
