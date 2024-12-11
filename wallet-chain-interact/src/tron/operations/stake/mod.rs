pub mod delegate;
pub use delegate::*;

pub mod freeze;
pub use freeze::*;

pub mod undelegate;
pub use undelegate::*;

pub mod unfreeze;
pub use unfreeze::*;

pub mod vote;

#[derive(serde::Serialize, Debug)]
pub enum ResourceType {
    ENERGY,
    BANDWIDTH,
}

impl ResourceType {
    pub fn to_int_str(&self) -> String {
        match self {
            ResourceType::ENERGY => "1".to_string(),
            ResourceType::BANDWIDTH => "0".to_string(),
        }
    }
}
impl TryFrom<&str> for ResourceType {
    type Error = crate::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_ref() {
            "energy" => Ok(ResourceType::ENERGY),
            "bandwidth" => Ok(ResourceType::BANDWIDTH),
            _ => panic!("invalid resource type {:?}", value),
        }
    }
}
