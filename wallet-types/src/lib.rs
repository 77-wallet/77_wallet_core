pub use rust_decimal::Decimal;
pub mod chain;
pub mod constant;
pub mod error;
pub mod valueobject;
pub use error::Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct BalanceTest {
    #[serde(with = "rust_decimal::serde::float")]
    balance: Decimal,
}

#[test]
fn test_deserialize_and_serialize_decimal() {
    let json_data = r#"{
        "balance": 0.002906173
    }"#;

    // 反序列化 JSON -> BalanceTest
    let result: Result<BalanceTest, _> = serde_json::from_str(json_data);
    match result {
        Ok(parsed) => {
            println!("Parsed balance: {}", parsed.balance);
            assert_eq!(
                parsed.balance,
                Decimal::from_str_exact("0.002906173").unwrap()
            );

            // 序列化 BalanceTest -> JSON
            let serialized = serde_json::to_string(&parsed).unwrap();
            println!("Serialized struct: {}", serialized);
        }
        Err(err) => {
            panic!("Failed to deserialize: {}", err);
        }
    }
}
