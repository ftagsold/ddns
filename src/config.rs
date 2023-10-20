use serde::Deserialize;
use std::fmt::{Display, Formatter};

#[derive(PartialEq, Clone, Deserialize)]
pub enum IpType {
    A,
    AAAA,
}

impl Display for IpType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IpType::A => write!(f, "A"),
            IpType::AAAA => write!(f, "AAAA"),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub ttl: u64,
    pub sleep: u64,
    pub zone: String,
    pub token: String,
    pub ip_type: IpType,
    pub sub_domains: Vec<String>,
}
