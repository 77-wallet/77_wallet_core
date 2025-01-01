use serde::Deserialize;

pub fn serde_to_string<T: ?Sized + serde::Serialize>(value: &T) -> Result<String, crate::Error> {
    serde_json::to_string(value).map_err(|e| crate::Error::Serde(e.into()))
}

pub fn serde_from_str<T: serde::de::DeserializeOwned>(value: &str) -> Result<T, crate::Error> {
    serde_json::from_str::<T>(value).map_err(|e| {
        crate::Error::Serde(crate::error::serde::SerdeError::Deserialize(format!(
            "error = {} value = {}",
            e, value
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
