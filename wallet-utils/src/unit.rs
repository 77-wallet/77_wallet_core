use crate::error::parse;
use crate::error::Error;
use std::str::FromStr;

use alloy::primitives::{
    utils::{format_units, parse_units, ParseUnits},
    U256,
};

pub fn convert_to_u256(value: &str, unit: u8) -> Result<U256, crate::Error> {
    Ok(parse_units(value, unit)
        .map_err(|e| {
            Error::Parse(parse::ParseError::UnitConvertFailed(format!(
                "convert_to_u256() value = {},unit = {} error:{}",
                value, unit, e
            )))
        })?
        .into())
}

pub fn u256_from_str(value: &str) -> Result<U256, crate::Error> {
    U256::from_str(value).map_err(|e| {
        Error::Parse(parse::ParseError::UnitConvertFailed(format!(
            " u256_from_str() value = {},error = {}",
            value, e
        )))
    })
}

pub fn format_to_string<T: Into<ParseUnits>>(value: T, unit: u8) -> Result<String, crate::Error> {
    let res = format_units(value, unit).map_err(|e| {
        Error::Parse(parse::ParseError::UnitConvertFailed(format!(
            "format_to_string() from str error:{}",
            e
        )))
    })?;
    let res = res.trim_end_matches('0').trim_end_matches('.');
    Ok(res.to_string())
}

pub fn format_to_f64<T: Into<ParseUnits>>(value: T, unit: u8) -> Result<f64, crate::Error> {
    let res = format_to_string(value, unit)?;
    let res = res.parse::<f64>().map_err(|e| {
        Error::Parse(parse::ParseError::UnitConvertFailed(format!(
            "format_to_f64() from str error:{}",
            e
        )))
    })?;
    Ok(res)
}
pub fn string_to_f64(value: &str) -> Result<f64, crate::Error> {
    let res = value.parse::<f64>().map_err(|e| {
        Error::Parse(parse::ParseError::UnitConvertFailed(format!(
            "string_to_f64() from str error:{}",
            e
        )))
    })?;
    Ok(res)
}

pub fn truncate_to_8_decimals(input: &str) -> String {
    if input.is_empty() {
        return "0".to_string(); // 空字符串直接返回
    }

    // 尝试将字符串解析为 f64
    let value = match f64::from_str(input) {
        Ok(v) => v,
        Err(_) => return "0".to_string(), // 解析失败，返回空字符串
    };

    // 如果是 0，直接返回 "0"
    if value == 0.0 {
        return "0".to_string();
    }

    // 截断小数点后 8 位
    let multiplier = 10f64.powi(8);
    let truncated = (value * multiplier).trunc() / multiplier;

    // 转换为字符串，去掉多余的 0
    let result = truncated.to_string();
    if result.contains('.') {
        result
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        result
    }
}

#[cfg(test)]
mod test {
    use super::string_to_f64;

    #[test]
    fn test_string_to_f64() {
        let a = "1.0";
        let a = string_to_f64(a).unwrap();
        assert_eq!(a, 1.0);

        let a = "1";
        let a = string_to_f64(a).unwrap();
        assert_eq!(a, 1.0)
    }
}
