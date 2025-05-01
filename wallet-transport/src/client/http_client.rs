use crate::{errors::TransportError, request_builder::ReqBuilder};
use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone)]
pub struct HttpClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new(
        base_url: &str,
        headers_opt: Option<HashMap<String, String>>,
        timeout: Option<std::time::Duration>,
    ) -> Result<Self, TransportError> {
        let mut headers = HeaderMap::new();

        headers.append(header::ACCEPT, "application/json".parse().unwrap());
        headers.append(header::CONTENT_TYPE, "application/json".parse().unwrap());

        if let Some(opt) = headers_opt {
            for (key, value) in opt {
                headers.append(
                    HeaderName::from_str(&key).unwrap(),
                    HeaderValue::from_str(&value).unwrap(),
                );
            }
        };

        let mut client = reqwest::ClientBuilder::new().default_headers(headers);
        #[cfg(feature = "accept_invalid_certs")]
        let mut client = client.danger_accept_invalid_certs(true);

        if let Some(timeout) = timeout {
            client = client.timeout(timeout);
        }
        let client = client
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
        let url;
        if !endpoint.is_empty() {
            url = format!("{}/{}", self.base_url, endpoint);
        } else {
            url = self.base_url.to_string();
        }
        let build = self.client.post(url);
        ReqBuilder(build)
    }

    pub fn get(&self, endpoint: &str) -> ReqBuilder {
        let url = format!("{}/{}", self.base_url, endpoint);
        tracing::debug!("request url = {}", url);
        let build = self.client.get(url);
        ReqBuilder(build)
    }

    pub async fn get_request<R>(&self, endpoint: &str) -> Result<R, TransportError>
    where
        R: serde::de::DeserializeOwned,
    {
        self.get(endpoint).send::<R>().await
    }

    pub async fn get_with_params<T, R>(
        &self,
        endpoint: &str,
        payload: T,
    ) -> Result<R, TransportError>
    where
        R: serde::de::DeserializeOwned,
        T: serde::Serialize + std::fmt::Debug,
    {
        self.get(endpoint).query(payload).send::<R>().await
    }

    pub async fn post_request<T, U>(&self, endpoint: &str, payload: T) -> Result<U, TransportError>
    where
        T: serde::Serialize + std::fmt::Debug,
        U: serde::de::DeserializeOwned,
    {
        self.post(endpoint).json(payload).send::<U>().await
    }
}
