mod command;
mod resp;
mod server;
mod store;

use server::Server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = std::env::var("RUDIS_ADDR").unwrap_or_else(|_| "127.0.0.1:6379".to_string());

    println!("Starting Rudis (Rust Redis) server...");
    println!("Version: 0.1.0");

    let server = Server::new(addr);
    server.run().await
}
