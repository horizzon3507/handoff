mod config;
mod network;

fn main() {
    match config::Config::from_env() {
        Ok(config) => {
            let address = network::primary_ipv4(config.host_override);
            println!("Handoff Rust server scaffold on port {}", config.port);
            println!("Local network address: {address:?}");
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
