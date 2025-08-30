use serde::Deserialize;

pub fn serde_to_string<T: ?Sized + serde::Serialize>(value: &T) -> Result<String, crate::Error> {
    serde_json::to_string(value).map_err(|e| crate::Error::Serde(e.into()))
}

pub fn serde_from_str<T: serde::de::DeserializeOwned>(value: &str) -> Result<T, crate::Error> {
    serde_json::from_str::<T>(value).map_err(|e| {
        // 太长进行截断
        const MAX_LEN: usize = 300;
        let shown = if value.len() > MAX_LEN {
            format!(
                "{}... [truncated, total_len={}]",
                &value[..MAX_LEN],
                value.len()
            )
        } else {
            value.to_string()
        };

        crate::Error::Serde(crate::error::serde::SerdeError::Deserialize(format!(
            "error = {} value = {}",
            e, shown
        )))
    })
}

pub fn serde_from_value<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
) -> Result<T, crate::Error> {
    serde_json::from_value(value).map_err(|e| crate::Error::Serde(e.into()))
}

pub fn serde_to_value<T: serde::Serialize>(value: T) -> Result<serde_json::Value, crate::Error> {
    serde_json::to_value(value).map_err(|e| crate::Error::Serde(e.into()))
}

pub fn serde_to_vec<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, crate::Error> {
    serde_json::to_vec(value).map_err(|e| crate::Error::Serde(e.into()))
}

pub fn serde_from_slice<T: for<'a> serde::de::Deserialize<'a>>(
    value: &[u8],
) -> Result<T, crate::Error> {
    serde_json::from_slice(value).map_err(|e| crate::Error::Serde(e.into()))
}

pub fn deserialize_uppercase_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|opt| opt.map(|s| s.to_uppercase()))
}

pub fn deserialize_uppercase<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer).map(|s| s.to_uppercase())
}

pub fn deserialize_default_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<bool>::deserialize(deserializer)?.unwrap_or(false))
}

pub fn deserialize_decimal_from_str<'de, D>(
    deserializer: D,
) -> Result<rust_decimal::Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use std::str::FromStr;
    let s = String::deserialize(deserializer)?;
    rust_decimal::Decimal::from_str(&s).map_err(serde::de::Error::custom)
}

pub fn deserialize_decimal_from_f64<'de, D>(
    deserializer: D,
) -> Result<rust_decimal::Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use rust_decimal::prelude::FromPrimitive;
    let float_val = f64::deserialize(deserializer)?;
    rust_decimal::Decimal::from_f64(float_val)
        .ok_or_else(|| serde::de::Error::custom("Failed to convert f64 to Decimal"))
}

pub fn deserialize_empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.is_empty()))
}

pub fn serialize_lowercase<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_lowercase())
}

pub fn string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse()
        .map_err(serde::de::Error::custom)
}

pub fn vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer)
}

pub fn toml_from_str<'de, T: serde::de::DeserializeOwned>(value: &str) -> Result<T, crate::Error> {
    toml::from_str(value).map_err(|e| crate::Error::Serde(e.into()))
}

pub fn toml_to_string<T: ?Sized + serde::Serialize>(value: &T) -> Result<String, crate::Error> {
    toml::to_string(value).map_err(|e| crate::Error::Serde(e.into()))
}

pub fn serde_yaml_from_str<'de, T: serde::de::DeserializeOwned>(
    value: &str,
) -> Result<T, crate::Error> {
    serde_yaml::from_str(value).map_err(|e| crate::Error::Serde(e.into()))
}

pub fn serde_yaml_to_string<T: ?Sized + serde::Serialize>(
    value: &T,
) -> Result<String, crate::Error> {
    serde_yaml::to_string(value).map_err(|e| crate::Error::Serde(e.into()))
}

pub fn serde_yaml_from_value<'de, T: serde::de::DeserializeOwned>(
    value: serde_yaml::Value,
) -> Result<T, crate::Error> {
    serde_yaml::from_value(value).map_err(|e| crate::Error::Serde(e.into()))
}

pub fn bcs_to_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, crate::Error> {
    bcs::to_bytes(value).map_err(|e| crate::Error::Serde(e.into()))
}
