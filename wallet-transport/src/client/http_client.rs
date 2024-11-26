use std::{collections::HashMap, str::FromStr, time::Duration};

use crate::{errors::TransportError, request_builder::ReqBuilder};
use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};

#[derive(Debug, Clone)]
pub struct HttpClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new(
        base_url: &str,
        headers_opt: Option<HashMap<String, String>>,
    ) -> Result<Self, TransportError> {
        let mut headers = HeaderMap::new();

        headers.append(header::ACCEPT, "application/json".parse().unwrap());
        headers.append(header::CONTENT_TYPE, "application/json".parse().unwrap());
        // headers.append("decodeRequest", "false".parse().unwrap());
        // headers.append("encryptResponse", "true".parse().unwrap());

        if let Some(opt) = headers_opt {
            for (key, value) in opt {
                headers.append(
                    HeaderName::from_str(&key).unwrap(),
                    HeaderValue::from_str(&value).unwrap(),
                );
            }
        };

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| crate::TransportError::Utils(wallet_utils::Error::Http(e.into())))?;

        Ok(Self {
            base_url: base_url.to_owned(),
            client,
        })
    }

    pub fn replace_base_url(&mut self, base_url: &str) {
        self.base_url = base_url.to_owned();
    }

    pub fn post(&self, endpoint: &str) -> ReqBuilder {
        let url = format!("{}/{}", self.base_url, endpoint);
        tracing::info!("request url = {}", url);
        let build = self.client.post(url);
        ReqBuilder(build)
    }

    pub fn get(&self, endpoint: &str) -> ReqBuilder {
        let url = format!("{}/{}", self.base_url, endpoint);
        tracing::info!("request url = {}", url);
        let build = self.client.get(url);
        ReqBuilder(build)
    }

    pub async fn get_request<R>(&self, endpoint: &str) -> Result<R, TransportError>
    where
        R: serde::de::DeserializeOwned,
    {
        let url = format!("{}/{}", self.base_url, endpoint);
        tracing::info!("request url = {}", url);
        self.get(endpoint).send::<R>().await
    }

    pub async fn post_request<T, U>(&self, endpoint: &str, payload: T) -> Result<U, TransportError>
    where
        T: serde::Serialize + std::fmt::Debug,
        U: serde::de::DeserializeOwned,
    {
        self.post(endpoint).json(payload).send::<U>().await
    }
}
