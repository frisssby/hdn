use chrono::{DateTime, Utc};
use std::{collections::HashMap, sync::Mutex};

pub type Key = String;
pub type Hash = String;
pub type Time = DateTime<Utc>;

#[derive(Debug)]
struct Value {
    pub hash: Hash,
    pub time: Time,
}

#[derive(Debug, Default)]
pub struct Storage {
    hash_map: Mutex<HashMap<Key, Value>>,
}

impl Storage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store(&self, key: Key, hash: Hash) -> Time {
        let mut hash_map = self.hash_map.lock().unwrap();
        let time = Utc::now();
        hash_map.insert(key, Value { hash, time });
        time
    }

    pub fn load(&self, key: &Key) -> Option<Hash> {
        let hash_map = self.hash_map.lock().unwrap();
        hash_map.get(key).map(|val| val.hash.clone())
    }

    pub fn synchronize(&self, key: Key, hash: Hash, time: Time) {
        let mut hash_map = self.hash_map.lock().unwrap();
        if let Some(old_value) = hash_map.get(&key) {
            if old_value.time > time {
                return;
            }
        }
        hash_map.insert(key, Value { hash, time });
    }

    pub fn size(&self) -> usize {
        let hash_map = self.hash_map.lock().unwrap();
        hash_map.len()
    }
}
