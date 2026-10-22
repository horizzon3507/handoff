use std::net::Ipv4Addr;

use crate::config::Config;
use crate::network;
use crate::room::RoomToken;
use crate::storage::RoomStore;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub primary_ip: Option<Ipv4Addr>,
    pub token: RoomToken,
    pub store: RoomStore,
}

impl AppState {
    pub fn new(config: Config, store: RoomStore) -> Self {
        let primary_ip = network::primary_ipv4(config.host_override);
        Self {
            config,
            primary_ip,
            token: RoomToken::generate(),
            store,
        }
    }

    pub fn share_url(&self) -> Option<String> {
        self.primary_ip
            .map(|address| network::share_url(address, self.config.port, self.token.as_str()))
    }

    pub fn is_room_token(&self, candidate: &str) -> bool {
        self.token.matches(candidate)
    }
}
