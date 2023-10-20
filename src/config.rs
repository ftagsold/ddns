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
    pub sleep: u64,
    pub zone: String,
    pub name: String,
    pub token: String,
    pub ip_type: IpType,
}
