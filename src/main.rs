mod config;

fn main() {
    match config::Config::from_env() {
        Ok(config) => println!("Handoff Rust server scaffold on port {}", config.port),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
