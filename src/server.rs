use crate::command::Command;
use crate::resp::RESPValue;
use crate::store::Store;
use std::io::BufReader;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    store: Store,
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Self {
            store: Store::new(),
            addr,
        }
    }

    pub async fn run(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("Rudis server listening on {}", self.addr);

        loop {
            let (socket, addr) = listener.accept().await?;
            println!("New connection from: {}", addr);

            let store = self.store.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(socket, store).await {
                    eprintln!("Error handling client {}: {}", addr, e);
                }
            });
        }
    }

    async fn handle_client(mut socket: TcpStream, store: Store) -> std::io::Result<()> {
        let mut buffer = vec![0u8; 4096];

        loop {
            let n = socket.read(&mut buffer).await?;
            if n == 0 {
                return Ok(());
            }

            let cursor = std::io::Cursor::new(&buffer[..n]);
            let mut reader = BufReader::new(cursor);

            match RESPValue::parse(&mut reader) {
                Ok(value) => {
                    let response = if let Some(cmd) = Command::from_resp(value) {
                        println!("Executing command: {}", cmd.name);
                        cmd.execute(&store)
                    } else {
                        RESPValue::Error("ERR invalid command format".to_string())
                    };

                    socket.write_all(&response.serialize()).await?;
                }
                Err(e) => {
                    let error = RESPValue::Error(format!("ERR parse error: {}", e));
                    socket.write_all(&error.serialize()).await?;
                }
            }
        }
    }
}
