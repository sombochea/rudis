use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

#[derive(Clone, Debug)]
pub struct ValueWithExpiry {
    pub data: Vec<u8>,
    pub expires_at: Option<SystemTime>,
}

impl ValueWithExpiry {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            expires_at: None,
        }
    }

    pub fn with_expiry(data: Vec<u8>, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Some(SystemTime::now() + ttl),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct Store {
    data: Arc<RwLock<HashMap<String, ValueWithExpiry>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let data = self.data.read().unwrap();
        data.get(key).and_then(|v| {
            if v.is_expired() {
                None
            } else {
                Some(v.data.clone())
            }
        })
    }

    pub fn set(&self, key: String, value: Vec<u8>) {
        let mut data = self.data.write().unwrap();
        data.insert(key, ValueWithExpiry::new(value));
    }

    pub fn set_with_expiry(&self, key: String, value: Vec<u8>, ttl: Duration) {
        let mut data = self.data.write().unwrap();
        data.insert(key, ValueWithExpiry::with_expiry(value, ttl));
    }

    pub fn del(&self, keys: &[String]) -> usize {
        let mut data = self.data.write().unwrap();
        let mut count = 0;
        for key in keys {
            if data.remove(key).is_some() {
                count += 1;
            }
        }
        count
    }

    pub fn exists(&self, keys: &[String]) -> usize {
        let data = self.data.read().unwrap();
        keys.iter()
            .filter(|key| {
                data.get(key.as_str())
                    .map_or(false, |v| !v.is_expired())
            })
            .count()
    }

    pub fn keys(&self, pattern: &str) -> Vec<String> {
        let data = self.data.read().unwrap();
        
        if pattern == "*" {
            data.iter()
                .filter(|(_, v)| !v.is_expired())
                .map(|(k, _)| k.clone())
                .collect()
        } else {
            // Simple pattern matching
            let prefix = pattern.trim_end_matches('*');
            data.iter()
                .filter(|(k, v)| !v.is_expired() && k.starts_with(prefix))
                .map(|(k, _)| k.clone())
                .collect()
        }
    }

    pub fn incr(&self, key: &str) -> Result<i64, String> {
        let mut data = self.data.write().unwrap();
        
        let current = if let Some(val) = data.get(key) {
            if val.is_expired() {
                0
            } else {
                String::from_utf8(val.data.clone())
                    .map_err(|_| "ERR value is not an integer or out of range")?
                    .parse::<i64>()
                    .map_err(|_| "ERR value is not an integer or out of range")?
            }
        } else {
            0
        };

        let new_value = current + 1;
        data.insert(
            key.to_string(),
            ValueWithExpiry::new(new_value.to_string().into_bytes()),
        );
        Ok(new_value)
    }

    pub fn decr(&self, key: &str) -> Result<i64, String> {
        let mut data = self.data.write().unwrap();
        
        let current = if let Some(val) = data.get(key) {
            if val.is_expired() {
                0
            } else {
                String::from_utf8(val.data.clone())
                    .map_err(|_| "ERR value is not an integer or out of range")?
                    .parse::<i64>()
                    .map_err(|_| "ERR value is not an integer or out of range")?
            }
        } else {
            0
        };

        let new_value = current - 1;
        data.insert(
            key.to_string(),
            ValueWithExpiry::new(new_value.to_string().into_bytes()),
        );
        Ok(new_value)
    }

    pub fn flush(&self) {
        let mut data = self.data.write().unwrap();
        data.clear();
    }

    pub fn dbsize(&self) -> usize {
        let data = self.data.read().unwrap();
        data.iter().filter(|(_, v)| !v.is_expired()).count()
    }
}
