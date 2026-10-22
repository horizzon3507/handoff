mod config;
mod network;
mod room;
mod state;
mod storage;
mod web;

#[tokio::main]
async fn main() {
    match config::Config::from_env() {
        Ok(config) => {
            let room_store = match storage::RoomStore::new().await {
                Ok(store) => store,
                Err(error) => {
                    eprintln!("Could not create the temporary room: {error}");
                    std::process::exit(1);
                }
            };
            let state = state::AppState::new(config, room_store.clone());
            println!(
                "Handoff Rust runtime state ready on port {}",
                state.config.port
            );
            println!("Local network address: {:?}", state.primary_ip);
            println!(
                "Temporary room created at {}",
                room_store.directory().display()
            );
            println!("Shared files: {}", room_store.list().await.len());
            let share_url = state.share_url().unwrap_or_default();
            let files = room_store.list().await;
            let rendered_page =
                web::render_room_page(state.token.as_str(), &share_url, "/qr/room.svg", &files);
            println!(
                "Rendered room page: {} bytes",
                rendered_page.into_string().len()
            );
            let rendered_qr = web::render_qr_svg(&share_url).unwrap_or_default();
            println!("Rendered room QR: {} bytes", rendered_qr.len());
            let _ = room_store.cleanup().await;
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
