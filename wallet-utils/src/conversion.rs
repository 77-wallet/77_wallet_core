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

pub fn decimal_from_str(val: &str) -> Result<Decimal, crate::Error> {
    val.parse::<Decimal>()
        .map_err(|e| crate::Error::Parse(e.into()))
}

#[cfg(test)]
mod test {
    use crate::parse_func::decimal_from_str;

    #[test]
    fn test_decimal_from_str() {
        let val = "0.78";

        let res = decimal_from_str(&val).unwrap();
        println!("res = {res}");
    }
}
