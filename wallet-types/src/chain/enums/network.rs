#[derive(Clone, Copy, Debug, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NetworkKind {
    Mainnet,
    Testnet,
    Regtest,
}
impl From<&str> for NetworkKind {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_ref() {
            "mainnet" => NetworkKind::Mainnet,
            "testnet" => NetworkKind::Testnet,
            "regtest" => NetworkKind::Regtest,
            _ => panic!("Invalid network kind: {}", value),
        }
    }
}

impl Into<bitcoin::NetworkKind> for NetworkKind {
    fn into(self) -> bitcoin::NetworkKind {
        match self {
            NetworkKind::Mainnet => bitcoin::NetworkKind::Main,
            NetworkKind::Testnet | NetworkKind::Regtest => bitcoin::NetworkKind::Test,
        }
    }
}
