mod config;
mod network;
mod room;
mod storage;
mod web;

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
            println!("Shared files: {}", room_store.list().await.len());
            let share_url = address
                .map(|address| network::share_url(address, config.port, _room_token.as_str()))
                .unwrap_or_default();
            let files = room_store.list().await;
            let rendered_page = web::render_room_page(
                _room_token.as_str(),
                &share_url,
                "/qr/room.svg",
                &files,
            );
            println!("Rendered room page: {} bytes", rendered_page.into_string().len());
            let _ = room_store.cleanup().await;
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
