mod config;
mod network;
mod room;
mod storage;

#[tokio::main]
async fn main() {
    match config::Config::from_env() {
        Ok(config) => {
            let address = network::primary_ipv4(config.host_override);
            let _room_token = room::RoomToken::generate();
            let room_store = match storage::RoomStore::new().await {
                Ok(store) => store,
                Err(error) => {
                    eprintln!("Could not create the temporary room: {error}");
                    std::process::exit(1);
                }
            };
            println!("Handoff Rust server scaffold on port {}", config.port);
            println!("Local network address: {address:?}");
            println!("Temporary room created at {}", room_store.directory().display());
            let _ = room_store.cleanup().await;
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
