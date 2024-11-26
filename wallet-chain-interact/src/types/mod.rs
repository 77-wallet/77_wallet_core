mod request;
pub use request::*;
mod response;
pub use response::*;
pub mod transaction;
pub use transaction::*;

/// Trait for calculating transaction fees.
pub trait FeeCalculator {
    fn calculate_fee(&self) -> f64;
}
