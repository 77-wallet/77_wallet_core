use crate::{
    errors::{NodeResponseError, TransportError},
    request_builder::ReqBuilder,
    types::JsonRpcResult,
};
use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fmt::Debug, str::FromStr};

pub struct RpcClient {
    base_url: String,
    client: reqwest::Client,
    base_auth: Option<BaseAuth>,
}

pub struct BaseAuth {
    name: String,
    password: Option<String>,
}

impl RpcClient {
    pub fn new(
        base_url: &str,
        header_opt: Option<HashMap<String, String>>,
        timeout: Option<std::time::Duration>,
    ) -> Result<Self, TransportError> {
        let mut headers = HeaderMap::new();

        headers.append(header::ACCEPT, "application/json".parse().unwrap());
        headers.append(header::CONTENT_TYPE, "application/json".parse().unwrap());

        if let Some(opt) = header_opt {
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
            base_auth: None,
        })
    }

    pub fn new_with_base_auth(
        base_url: &str,
        username: &str,
        password: &str,
        timeout: Option<std::time::Duration>,
    ) -> Result<Self, TransportError> {
        let mut headers = HeaderMap::new();

        headers.append(header::ACCEPT, "application/json".parse().unwrap());
        headers.append(header::CONTENT_TYPE, "application/json".parse().unwrap());

        let base_auth = Some(BaseAuth {
            name: username.to_owned(),
            password: Some(password.to_owned()),
        });

        let mut client = reqwest::ClientBuilder::new().default_headers(headers);
        if let Some(timeout) = timeout {
            client = client.timeout(timeout);
        }

        let client = client
            .build()
            .map_err(|e| crate::TransportError::Utils(wallet_utils::Error::Http(e.into())))?;

        Ok(Self {
            base_url: base_url.to_owned(),
            client,
            base_auth,
        })
    }

    pub fn set_params<T: Serialize + Debug>(&self, p: T) -> ReqBuilder {
        // tracing::info!("[req url] = {:?}", self.base_url);
        tracing::debug!("[req params] = {:?}", p);

        let build = if let Some(auth) = &self.base_auth {
            self.client
                .post(&self.base_url)
                .basic_auth(&auth.name, auth.password.clone())
                .json(&p)
        } else {
            self.client.post(&self.base_url).json(&p)
        };

        ReqBuilder(build)
    }

    pub async fn invoke_request<T, R>(&self, params: T) -> Result<R, TransportError>
    where
        T: Serialize + Debug,
        R: DeserializeOwned,
    {
        let response = self.set_params(params).do_request().await?;
        tracing::info!("response: {:?}", response);
        let rpc_result = wallet_utils::serde_func::serde_from_str::<JsonRpcResult<R>>(&response)?;

        if let Some(err) = rpc_result.error {
            return Err(TransportError::NodeResponseError(NodeResponseError::new(
                err.code,
                Some(err.message),
            )));
        }

        match rpc_result.result {
            Some(res) => Ok(res),
            None => Err(TransportError::EmptyResult),
        }
    }
}
