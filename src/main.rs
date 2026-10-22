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
            let state = state::AppState::new(config, room_store);
            let address = std::net::SocketAddr::from(([0, 0, 0, 0], state.config.port));
            let listener = match tokio::net::TcpListener::bind(address).await {
                Ok(listener) => listener,
                Err(error) => {
                    eprintln!("Could not listen on port {}: {error}", state.config.port);
                    std::process::exit(1);
                }
            };

            println!("Handoff is ready:");
            println!("  This device: http://127.0.0.1:{}/", state.config.port);
            if let Some(url) = state.share_url() {
                println!("  Same Wi-Fi:  {url}");
            } else {
                println!("  Same Wi-Fi:  network address not detected; set HANDOFF_HOST to advertise one");
            }
            println!("Press Ctrl-C to close the temporary room.");

            if let Err(error) = axum::serve(listener, web::router(state))
                .with_graceful_shutdown(shutdown_signal())
                .await
            {
                eprintln!("The server stopped with an error: {error}");
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
