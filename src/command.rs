use crate::resp::RESPValue;
use crate::store::Store;
use std::time::Duration;

pub struct Command {
    pub name: String,
    pub args: Vec<Vec<u8>>,
}

impl Command {
    pub fn from_resp(value: RESPValue) -> Option<Self> {
        match value {
            RESPValue::Array(Some(arr)) if !arr.is_empty() => {
                let name = arr[0].as_string()?.to_uppercase();
                let args = arr[1..]
                    .iter()
                    .filter_map(|v| v.as_bulk_string())
                    .collect();
                Some(Command { name, args })
            }
            _ => None,
        }
    }

    pub fn execute(&self, store: &Store) -> RESPValue {
        match self.name.as_str() {
            "PING" => self.handle_ping(),
            "ECHO" => self.handle_echo(),
            "GET" => self.handle_get(store),
            "SET" => self.handle_set(store),
            "DEL" => self.handle_del(store),
            "EXISTS" => self.handle_exists(store),
            "KEYS" => self.handle_keys(store),
            "INCR" => self.handle_incr(store),
            "DECR" => self.handle_decr(store),
            "FLUSHDB" => self.handle_flushdb(store),
            "DBSIZE" => self.handle_dbsize(store),
            "EXPIRE" => self.handle_expire(store),
            "TTL" => self.handle_ttl(),
            "LPUSH" => self.handle_lpush(store),
            "RPUSH" => self.handle_rpush(store),
            "LPOP" => self.handle_lpop(store),
            "RPOP" => self.handle_rpop(store),
            "LRANGE" => self.handle_lrange(store),
            "LLEN" => self.handle_llen(store),
            "LINDEX" => self.handle_lindex(store),
            _ => RESPValue::Error(format!("ERR unknown command '{}'", self.name)),
        }
    }

    fn handle_ping(&self) -> RESPValue {
        if self.args.is_empty() {
            RESPValue::SimpleString("PONG".to_string())
        } else {
            RESPValue::BulkString(Some(self.args[0].clone()))
        }
    }

    fn handle_echo(&self) -> RESPValue {
        if self.args.is_empty() {
            RESPValue::Error("ERR wrong number of arguments for 'echo' command".to_string())
        } else {
            RESPValue::BulkString(Some(self.args[0].clone()))
        }
    }

    fn handle_get(&self, store: &Store) -> RESPValue {
        if self.args.len() != 1 {
            return RESPValue::Error("ERR wrong number of arguments for 'get' command".to_string());
        }

        let key = String::from_utf8_lossy(&self.args[0]).to_string();
        match store.get(&key) {
            Some(value) => RESPValue::BulkString(Some(value)),
            None => RESPValue::BulkString(None),
        }
    }

    fn handle_set(&self, store: &Store) -> RESPValue {
        if self.args.len() < 2 {
            return RESPValue::Error("ERR wrong number of arguments for 'set' command".to_string());
        }

        let key = String::from_utf8_lossy(&self.args[0]).to_string();
        let value = self.args[1].clone();

        // Check for EX, PX, EXAT, PXAT options
        if self.args.len() > 2 {
            let option = String::from_utf8_lossy(&self.args[2]).to_uppercase();
            match option.as_str() {
                "EX" => {
                    if self.args.len() < 4 {
                        return RESPValue::Error("ERR syntax error".to_string());
                    }
                    let seconds = String::from_utf8_lossy(&self.args[3])
                        .parse::<u64>()
                        .unwrap_or(0);
                    store.set_with_expiry(key, value, Duration::from_secs(seconds));
                }
                "PX" => {
                    if self.args.len() < 4 {
                        return RESPValue::Error("ERR syntax error".to_string());
                    }
                    let millis = String::from_utf8_lossy(&self.args[3])
                        .parse::<u64>()
                        .unwrap_or(0);
                    store.set_with_expiry(key, value, Duration::from_millis(millis));
                }
                _ => store.set(key, value),
            }
        } else {
            store.set(key, value);
        }

        RESPValue::SimpleString("OK".to_string())
    }

    fn handle_del(&self, store: &Store) -> RESPValue {
        if self.args.is_empty() {
            return RESPValue::Error("ERR wrong number of arguments for 'del' command".to_string());
        }

        let keys: Vec<String> = self.args
            .iter()
            .map(|k| String::from_utf8_lossy(k).to_string())
            .collect();

        let count = store.del(&keys);
        RESPValue::Integer(count as i64)
    }

    fn handle_exists(&self, store: &Store) -> RESPValue {
        if self.args.is_empty() {
            return RESPValue::Error("ERR wrong number of arguments for 'exists' command".to_string());
        }

        let keys: Vec<String> = self.args
            .iter()
            .map(|k| String::from_utf8_lossy(k).to_string())
            .collect();

        let count = store.exists(&keys);
        RESPValue::Integer(count as i64)
    }

    fn handle_keys(&self, store: &Store) -> RESPValue {
        if self.args.len() != 1 {
            return RESPValue::Error("ERR wrong number of arguments for 'keys' command".to_string());
        }

        let pattern = String::from_utf8_lossy(&self.args[0]).to_string();
        let keys = store.keys(&pattern);
        
        let resp_keys: Vec<RESPValue> = keys
            .into_iter()
            .map(|k| RESPValue::BulkString(Some(k.into_bytes())))
            .collect();

        RESPValue::Array(Some(resp_keys))
    }

    fn handle_incr(&self, store: &Store) -> RESPValue {
        if self.args.len() != 1 {
            return RESPValue::Error("ERR wrong number of arguments for 'incr' command".to_string());
        }

        let key = String::from_utf8_lossy(&self.args[0]).to_string();
        match store.incr(&key) {
            Ok(value) => RESPValue::Integer(value),
            Err(e) => RESPValue::Error(e),
        }
    }

    fn handle_decr(&self, store: &Store) -> RESPValue {
        if self.args.len() != 1 {
            return RESPValue::Error("ERR wrong number of arguments for 'decr' command".to_string());
        }

        let key = String::from_utf8_lossy(&self.args[0]).to_string();
        match store.decr(&key) {
            Ok(value) => RESPValue::Integer(value),
            Err(e) => RESPValue::Error(e),
        }
    }

    fn handle_flushdb(&self, store: &Store) -> RESPValue {
        store.flush();
        RESPValue::SimpleString("OK".to_string())
    }

    fn handle_dbsize(&self, store: &Store) -> RESPValue {
        let size = store.dbsize();
        RESPValue::Integer(size as i64)
    }

    fn handle_expire(&self, store: &Store) -> RESPValue {
        if self.args.len() != 2 {
            return RESPValue::Error("ERR wrong number of arguments for 'expire' command".to_string());
        }

        let key = String::from_utf8_lossy(&self.args[0]).to_string();
        let seconds = String::from_utf8_lossy(&self.args[1])
            .parse::<u64>()
            .unwrap_or(0);

        if let Some(value) = store.get(&key) {
            store.set_with_expiry(key, value, Duration::from_secs(seconds));
            RESPValue::Integer(1)
        } else {
            RESPValue::Integer(0)
        }
    }

    fn handle_ttl(&self) -> RESPValue {
        RESPValue::Integer(-1)
    }

    fn handle_lpush(&self, store: &Store) -> RESPValue {
        if self.args.len() < 2 {
            return RESPValue::Error("ERR wrong number of arguments for 'lpush' command".to_string());
        }

        let key = String::from_utf8_lossy(&self.args[0]).to_string();
        let values: Vec<Vec<u8>> = self.args[1..].to_vec();
        
        let len = store.lpush(&key, values);
        RESPValue::Integer(len as i64)
    }

    fn handle_rpush(&self, store: &Store) -> RESPValue {
        if self.args.len() < 2 {
            return RESPValue::Error("ERR wrong number of arguments for 'rpush' command".to_string());
        }

        let key = String::from_utf8_lossy(&self.args[0]).to_string();
        let values: Vec<Vec<u8>> = self.args[1..].to_vec();
        
        let len = store.rpush(&key, values);
        RESPValue::Integer(len as i64)
    }

    fn handle_lpop(&self, store: &Store) -> RESPValue {
        if self.args.len() != 1 {
            return RESPValue::Error("ERR wrong number of arguments for 'lpop' command".to_string());
        }

        let key = String::from_utf8_lossy(&self.args[0]).to_string();
        
        match store.lpop(&key) {
            Ok(Some(value)) => RESPValue::BulkString(Some(value)),
            Ok(None) => RESPValue::BulkString(None),
            Err(e) => RESPValue::Error(e),
        }
    }

    fn handle_rpop(&self, store: &Store) -> RESPValue {
        if self.args.len() != 1 {
            return RESPValue::Error("ERR wrong number of arguments for 'rpop' command".to_string());
        }

        let key = String::from_utf8_lossy(&self.args[0]).to_string();
        
        match store.rpop(&key) {
            Ok(Some(value)) => RESPValue::BulkString(Some(value)),
            Ok(None) => RESPValue::BulkString(None),
            Err(e) => RESPValue::Error(e),
        }
    }

    fn handle_lrange(&self, store: &Store) -> RESPValue {
        if self.args.len() != 3 {
            return RESPValue::Error("ERR wrong number of arguments for 'lrange' command".to_string());
        }

        let key = String::from_utf8_lossy(&self.args[0]).to_string();
        let start = String::from_utf8_lossy(&self.args[1])
            .parse::<i64>()
            .unwrap_or(0);
        let stop = String::from_utf8_lossy(&self.args[2])
            .parse::<i64>()
            .unwrap_or(-1);
        
        match store.lrange(&key, start, stop) {
            Ok(values) => {
                let resp_values: Vec<RESPValue> = values
                    .into_iter()
                    .map(|v| RESPValue::BulkString(Some(v)))
                    .collect();
                RESPValue::Array(Some(resp_values))
            }
            Err(e) => RESPValue::Error(e),
        }
    }

    fn handle_llen(&self, store: &Store) -> RESPValue {
        if self.args.len() != 1 {
            return RESPValue::Error("ERR wrong number of arguments for 'llen' command".to_string());
        }

        let key = String::from_utf8_lossy(&self.args[0]).to_string();
        
        match store.llen(&key) {
            Ok(len) => RESPValue::Integer(len as i64),
            Err(e) => RESPValue::Error(e),
        }
    }

    fn handle_lindex(&self, store: &Store) -> RESPValue {
        if self.args.len() != 2 {
            return RESPValue::Error("ERR wrong number of arguments for 'lindex' command".to_string());
        }

        let key = String::from_utf8_lossy(&self.args[0]).to_string();
        let index = String::from_utf8_lossy(&self.args[1])
            .parse::<i64>()
            .unwrap_or(0);
        
        match store.lindex(&key, index) {
            Ok(Some(value)) => RESPValue::BulkString(Some(value)),
            Ok(None) => RESPValue::BulkString(None),
            Err(e) => RESPValue::Error(e),
        }
    }
}
