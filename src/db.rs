use crate::Result;
use bytes::Bytes;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Add,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    select, spawn,
    sync::{broadcast, mpsc, Notify},
    time::{sleep, Instant},
};

const DEFAULT_SLEEP_TIME: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct DbHolder {
    holder: Arc<SharedDb>,
}

pub struct SharedDb {
    database: Mutex<Database>,
    clean_task_notifier: Arc<Notify>,
}

struct DbCleaner {
    db: Arc<SharedDb>,
    clean_task_notifier: Arc<Notify>,
    shutdown_notifier: broadcast::Receiver<()>,
    _shutdown_completed_tx: mpsc::Sender<()>,
}

struct Database {
    entries: HashMap<String, Bytes>,
    expiration: BTreeMap<String, Instant>,
}

impl DbHolder {
    pub fn new(
        shutdown_notifier: broadcast::Receiver<()>,
        shutdown_completed_tx: mpsc::Sender<()>,
    ) -> DbHolder {
        let notifier = Arc::new(Notify::new());
        let holder = Arc::new(SharedDb {
            database: Mutex::new(Database::new()),
            clean_task_notifier: notifier.clone(),
        });

        let db = holder.clone();
        spawn(async move {
            let mut cleaner = DbCleaner {
                db,
                shutdown_notifier,
                _shutdown_completed_tx: shutdown_completed_tx,
                clean_task_notifier: notifier.clone(),
            };
            cleaner.clean_expired_keys().await;
        });
        DbHolder { holder }
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        self.holder
            .database
            .lock()
            .unwrap()
            .entries
            .get(key)
            .cloned()
    }

    pub fn set(&self, key: String, value: Bytes, expiration: Option<Duration>) -> Result<()> {
        let mut db = self.holder.database.lock().unwrap();
        let _prev = db.entries.insert(key.clone(), value);
        db.expiration.remove(&key);

        let next_expiration_time = db.expiration.values().min().cloned();

        if let Some(dur) = expiration {
            let expire_time = Instant::now().add(dur);
            db.expiration.insert(key, expire_time);
        }

        if let Some(next_expiration_time) = next_expiration_time {
            if expiration.is_none() {
                self.holder.clean_task_notifier.notify_one();
            } else if let Some(expiration) = expiration {
                let expiration = Instant::now() + expiration;
                if expiration < next_expiration_time {
                    self.holder.clean_task_notifier.notify_one();
                }
            }
        }

        Ok(())
    }
}

impl SharedDb {
    /// clean expired keys. return next expired duration if exists
    fn clean_expired_keys(&self) -> Option<Duration> {
        let mut db = self.database.lock().unwrap();
        let now = Instant::now();
        let mut expired_keys = vec![];
        for (key, time) in &db.expiration {
            if time < &now {
                expired_keys.push(key.clone());
            }
        }
        db.entries.retain(|x, _| !expired_keys.contains(x));
        db.expiration.retain(|x, _| !expired_keys.contains(x));

        db.expiration
            .iter()
            .min_by(|&this, &that| this.1.cmp(that.1))
            .map(|(_, time)| *time - Instant::now())
    }
}

impl DbCleaner {
    async fn clean_expired_keys(&mut self) {
        loop {
            let sleep_time = self.db.clean_expired_keys().unwrap_or(DEFAULT_SLEEP_TIME);
            select! {
                _ = self.shutdown_notifier.recv() => {
                    println!("background task stopped");
                    return;
                },
                _ = sleep(sleep_time) => {},
                _ = self.clean_task_notifier.notified() => {},
            }
        }
    }
}

impl Database {
    fn new() -> Database {
        Database {
            entries: HashMap::new(),
            expiration: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn clean_expired_keys_test() {
        let (shutdown_tx, _) = broadcast::channel(1);
        let (shutdown_completed_tx, mut shutdown_completed_rx) = mpsc::channel(1);
        let dbhodler = DbHolder::new(shutdown_tx.subscribe(), shutdown_completed_tx.clone());
        dbhodler
            .set(
                "test".to_string(),
                Bytes::from_static(b"h"),
                Some(Duration::from_secs(3)),
            )
            .unwrap();
        dbhodler
            .set(
                "test2".to_string(),
                Bytes::from_static(b"h"),
                Some(Duration::from_secs(1)),
            )
            .unwrap();
        assert_eq!(dbhodler.get("test"), Some(Bytes::from_static(b"h")));
        sleep(Duration::from_secs(4)).await;
        assert_eq!(dbhodler.get("test"), None);
        shutdown_tx.send(()).unwrap();
        drop(shutdown_completed_tx);
        shutdown_completed_rx.recv().await;
    }
}
