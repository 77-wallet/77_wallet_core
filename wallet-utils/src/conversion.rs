use rust_decimal::{
    Decimal,
    prelude::{FromPrimitive, ToPrimitive as _},
};

pub fn str_to_vec(raw: &str) -> Vec<u8> {
    raw.as_bytes().to_vec()
}

pub fn vec_to_string(raw: &[u8]) -> Result<String, crate::Error> {
    String::from_utf8(raw.to_vec()).map_err(|e| crate::Error::Parse(e.into()))
}

pub fn decimal_to_f64(decimal: &Decimal) -> Result<f64, crate::Error> {
    decimal
        .to_f64()
        .ok_or(crate::Error::Parse(crate::ParseError::DecimalToF64Failed))
}

pub fn decimal_from_f64(val: f64) -> Result<Decimal, crate::Error> {
    Decimal::from_f64(val).ok_or(crate::Error::Parse(
        crate::ParseError::FromF64ToDecimalFailed,
    ))
}
