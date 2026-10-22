use std::net::{IpAddr, Ipv4Addr, UdpSocket};

pub fn primary_ipv4(override_ip: Option<Ipv4Addr>) -> Option<Ipv4Addr> {
    if let Some(address) = override_ip {
        return Some(address);
    }

    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("192.0.2.1:80").ok()?;

    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(address) if !address.is_loopback() && !address.is_unspecified() => Some(address),
        _ => None,
    }
}

pub fn share_url(address: Ipv4Addr, port: u16, token: &str) -> String {
    format!("http://{address}:{port}/share/{token}")
}
