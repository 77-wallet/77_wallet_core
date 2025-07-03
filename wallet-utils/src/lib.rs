pub mod conversion;
mod crypto;
pub mod error;
pub mod file_func;
pub mod hex_func;
pub mod log;
pub mod parse_func;
pub mod ping;
pub mod serde_func;
pub mod snowflake;
pub mod system_info;
pub mod time;
pub use crypto::*;
pub mod address;
pub use error::{Error, http::HttpError, parse::ParseError, serde::SerdeError};
pub use log::{init_log, init_test_log};
mod global_value;
pub mod sign;
pub mod unit;

pub use global_value::*;

#[macro_export]
macro_rules! here {
    () => {
        &wallet_utils::error::Location {
            file: file!(),
            line: line!(),
            column: column!(),
        }
    };
}
