#[derive(Debug, thiserror::Error)]
pub enum IcmpError {
    #[error("Icmp error: {0}")]
    Ping(#[from] surge_ping::SurgeError),
    #[error("No IP addresses resolved")]
    NoIpAddressesResolved,
    #[error("Invalid URL")]
    InvalidURL,
}
