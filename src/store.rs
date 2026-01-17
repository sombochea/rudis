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

#[derive(Clone, Debug)]
pub enum Value {
    String(ValueWithExpiry),
    List(Vec<Vec<u8>>),
}

#[derive(Clone)]
pub struct Store {
    data: Arc<RwLock<HashMap<String, Value>>>,
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
            match v {
                Value::String(val) => {
                    if val.is_expired() {
                        None
                    } else {
                        Some(val.data.clone())
                    }
                }
                Value::List(_) => None,
            }
        })
    }

    pub fn set(&self, key: String, value: Vec<u8>) {
        let mut data = self.data.write().unwrap();
        data.insert(key, Value::String(ValueWithExpiry::new(value)));
    }

    pub fn set_with_expiry(&self, key: String, value: Vec<u8>, ttl: Duration) {
        let mut data = self.data.write().unwrap();
        data.insert(key, Value::String(ValueWithExpiry::with_expiry(value, ttl)));
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
                data.get(key.as_str()).is_some()
            })
            .count()
    }

    pub fn keys(&self, pattern: &str) -> Vec<String> {
        let data = self.data.read().unwrap();
        
        if pattern == "*" {
            data.keys().cloned().collect()
        } else {
            let prefix = pattern.trim_end_matches('*');
            data.keys()
                .filter(|k| k.starts_with(prefix))
                .cloned()
                .collect()
        }
    }

    pub fn incr(&self, key: &str) -> Result<i64, String> {
        let mut data = self.data.write().unwrap();
        
        let current = if let Some(Value::String(val)) = data.get(key) {
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
            Value::String(ValueWithExpiry::new(new_value.to_string().into_bytes())),
        );
        Ok(new_value)
    }

    pub fn decr(&self, key: &str) -> Result<i64, String> {
        let mut data = self.data.write().unwrap();
        
        let current = if let Some(Value::String(val)) = data.get(key) {
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
            Value::String(ValueWithExpiry::new(new_value.to_string().into_bytes())),
        );
        Ok(new_value)
    }

    pub fn flush(&self) {
        let mut data = self.data.write().unwrap();
        data.clear();
    }

    pub fn dbsize(&self) -> usize {
        let data = self.data.read().unwrap();
        data.len()
    }

    // List operations
    pub fn lpush(&self, key: &str, values: Vec<Vec<u8>>) -> usize {
        let mut data = self.data.write().unwrap();
        
        match data.get_mut(key) {
            Some(Value::List(list)) => {
                for value in values.into_iter().rev() {
                    list.insert(0, value);
                }
                list.len()
            }
            Some(Value::String(_)) => {
                // Key exists but is not a list - error handled in command layer
                0
            }
            None => {
                let mut list: Vec<Vec<u8>> = Vec::new();
                for value in values.into_iter().rev() {
                    list.insert(0, value);
                }
                let len = list.len();
                data.insert(key.to_string(), Value::List(list));
                len
            }
        }
    }

    pub fn rpush(&self, key: &str, values: Vec<Vec<u8>>) -> usize {
        let mut data = self.data.write().unwrap();
        
        match data.get_mut(key) {
            Some(Value::List(list)) => {
                list.extend(values);
                list.len()
            }
            Some(Value::String(_)) => {
                0
            }
            None => {
                let len = values.len();
                data.insert(key.to_string(), Value::List(values));
                len
            }
        }
    }

    pub fn lpop(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        let mut data = self.data.write().unwrap();
        
        match data.get_mut(key) {
            Some(Value::List(list)) => {
                if list.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(list.remove(0)))
                }
            }
            Some(Value::String(_)) => {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())
            }
            None => Ok(None),
        }
    }

    pub fn rpop(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        let mut data = self.data.write().unwrap();
        
        match data.get_mut(key) {
            Some(Value::List(list)) => {
                Ok(list.pop())
            }
            Some(Value::String(_)) => {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())
            }
            None => Ok(None),
        }
    }

    pub fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<Vec<u8>>, String> {
        let data = self.data.read().unwrap();
        
        match data.get(key) {
            Some(Value::List(list)) => {
                let len = list.len() as i64;
                
                // Convert negative indices
                let start_idx = if start < 0 { (len + start).max(0) } else { start };
                let stop_idx = if stop < 0 { (len + stop).max(-1) } else { stop };
                
                // Clamp to valid range
                let start_idx = (start_idx as usize).min(list.len());
                let stop_idx = ((stop_idx + 1) as usize).min(list.len());
                
                if start_idx >= stop_idx {
                    Ok(Vec::new())
                } else {
                    Ok(list[start_idx..stop_idx].to_vec())
                }
            }
            Some(Value::String(_)) => {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())
            }
            None => Ok(Vec::new()),
        }
    }

    pub fn llen(&self, key: &str) -> Result<usize, String> {
        let data = self.data.read().unwrap();
        
        match data.get(key) {
            Some(Value::List(list)) => Ok(list.len()),
            Some(Value::String(_)) => {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())
            }
            None => Ok(0),
        }
    }

    pub fn lindex(&self, key: &str, index: i64) -> Result<Option<Vec<u8>>, String> {
        let data = self.data.read().unwrap();
        
        match data.get(key) {
            Some(Value::List(list)) => {
                let len = list.len() as i64;
                let idx = if index < 0 { len + index } else { index };
                
                if idx < 0 || idx >= len {
                    Ok(None)
                } else {
                    Ok(Some(list[idx as usize].clone()))
                }
            }
            Some(Value::String(_)) => {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())
            }
            None => Ok(None),
        }
    }
}
