use std::net::Ipv4Addr;

pub const DEFAULT_PORT: u16 = 4173;
pub const MAX_FILE_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_ROOM_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const MAX_ROOM_FILES: usize = 200;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub host_override: Option<Ipv4Addr>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let port = match std::env::var("HANDOFF_PORT") {
            Ok(value) => value
                .parse::<u16>()
                .map_err(|_| "HANDOFF_PORT must be a number from 1024 to 65535.".to_owned())?,
            Err(std::env::VarError::NotPresent) => DEFAULT_PORT,
            Err(error) => return Err(format!("Could not read HANDOFF_PORT: {error}")),
        };

        if port < 1024 {
            return Err("HANDOFF_PORT must be a number from 1024 to 65535.".to_owned());
        }

        let host_override = match std::env::var("HANDOFF_HOST") {
            Ok(value) => Some(
                value
                    .parse::<Ipv4Addr>()
                    .map_err(|_| "HANDOFF_HOST must be an IPv4 address.".to_owned())?,
            ),
            Err(std::env::VarError::NotPresent) => None,
            Err(error) => return Err(format!("Could not read HANDOFF_HOST: {error}")),
        };

        Ok(Self { port, host_override })
    }
}
