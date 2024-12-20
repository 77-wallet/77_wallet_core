pub mod delegate;
pub use delegate::*;

pub mod freeze;
pub use freeze::*;

pub mod undelegate;
pub use undelegate::*;

pub mod unfreeze;
pub use unfreeze::*;

pub mod vote;
pub use vote::*;

#[derive(serde::Serialize, Debug, serde::Deserialize, Clone, Copy)]
pub enum ResourceType {
    ENERGY,
    BANDWIDTH,
}

impl ResourceType {
    pub fn to_i8(&self) -> i8 {
        match self {
            ResourceType::ENERGY => 1,
            ResourceType::BANDWIDTH => 0,
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
