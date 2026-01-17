use std::io::{self, BufRead, BufReader, Read};

#[derive(Debug, Clone, PartialEq)]
pub enum RESPValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<Vec<u8>>),
    Array(Option<Vec<RESPValue>>),
}

impl RESPValue {
    pub fn parse<R: Read>(reader: &mut BufReader<R>) -> io::Result<RESPValue> {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        
        if line.is_empty() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Empty line"));
        }

        let first_byte = line.as_bytes()[0];
        let content = &line[1..line.len() - 2]; // Remove prefix and \r\n

        match first_byte {
            b'+' => Ok(RESPValue::SimpleString(content.to_string())),
            b'-' => Ok(RESPValue::Error(content.to_string())),
            b':' => {
                let num = content.parse::<i64>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Ok(RESPValue::Integer(num))
            }
            b'$' => {
                let len = content.parse::<i64>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                
                if len == -1 {
                    return Ok(RESPValue::BulkString(None));
                }

                let mut buffer = vec![0u8; len as usize];
                reader.read_exact(&mut buffer)?;
                
                let mut crlf = [0u8; 2];
                reader.read_exact(&mut crlf)?;

                Ok(RESPValue::BulkString(Some(buffer)))
            }
            b'*' => {
                let count = content.parse::<i64>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                
                if count == -1 {
                    return Ok(RESPValue::Array(None));
                }

                let mut array = Vec::new();
                for _ in 0..count {
                    array.push(RESPValue::parse(reader)?);
                }
                Ok(RESPValue::Array(Some(array)))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown RESP type: {}", first_byte as char),
            )),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            RESPValue::SimpleString(s) => format!("+{}\r\n", s).into_bytes(),
            RESPValue::Error(e) => format!("-{}\r\n", e).into_bytes(),
            RESPValue::Integer(i) => format!(":{}\r\n", i).into_bytes(),
            RESPValue::BulkString(None) => b"$-1\r\n".to_vec(),
            RESPValue::BulkString(Some(data)) => {
                let mut result = format!("${}\r\n", data.len()).into_bytes();
                result.extend_from_slice(data);
                result.extend_from_slice(b"\r\n");
                result
            }
            RESPValue::Array(None) => b"*-1\r\n".to_vec(),
            RESPValue::Array(Some(arr)) => {
                let mut result = format!("*{}\r\n", arr.len()).into_bytes();
                for item in arr {
                    result.extend_from_slice(&item.serialize());
                }
                result
            }
        }
    }

    pub fn as_bulk_string(&self) -> Option<Vec<u8>> {
        match self {
            RESPValue::BulkString(Some(data)) => Some(data.clone()),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        self.as_bulk_string()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }
}
