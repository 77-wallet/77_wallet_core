use crate::{types::RpcResult, TransportError};
use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub struct ReqBuilder(pub RequestBuilder);

impl ReqBuilder {
    pub fn json(mut self, v: impl Serialize + Debug) -> Self {
        tracing::info!("request params: {:?}", serde_json::to_string(&v).unwrap());
        self.0 = self.0.json(&v);
        self
    }

    pub fn query(mut self, v: impl Serialize + Debug) -> Self {
        tracing::debug!("request params: {:?}", v);
        self.0 = self.0.query(&v);
        self
    }

    pub fn body(mut self, body: String) -> Self {
        tracing::info!("request params: {:?}", body);
        self.0 = self.0.body(body);
        self
    }

    pub async fn send<T: DeserializeOwned>(self) -> Result<T, crate::TransportError> {
        let res = self
            .0
            .send()
            .await
            .map_err(|e| crate::TransportError::Utils(wallet_utils::Error::Http(e.into())))?;

        if !res.status().is_success() {
            return Err(crate::TransportError::NodeResponseError(format!(
                "error_code {}",
                res.status()
            )));
        }

        let response = res
            .text()
            .await
            .map_err(|e| crate::TransportError::Utils(wallet_utils::Error::Http(e.into())))?;
        tracing::info!("response = {}", response);

        Ok(wallet_utils::serde_func::serde_from_str(&response)?)
    }

    pub async fn send_string(self) -> Result<String, crate::TransportError> {
        let res = self
            .0
            .send()
            .await
            .map_err(|e| crate::TransportError::Utils(wallet_utils::Error::Http(e.into())))?;

        if !res.status().is_success() {
            return Err(crate::TransportError::NodeResponseError(format!(
                "error_code {}",
                res.status()
            )));
        }

        let response = res
            .text()
            .await
            .map_err(|e| crate::TransportError::Utils(wallet_utils::Error::Http(e.into())))?;
        tracing::debug!("[rpc response] = {}", response);

        Ok(response)
    }

    pub async fn send_json_rpc<T: DeserializeOwned>(self) -> Result<T, crate::TransportError> {
        let res = self
            .0
            .send()
            .await
            .map_err(|e| crate::TransportError::Utils(wallet_utils::Error::Http(e.into())))?;

        if !res.status().is_success() {
            return Err(crate::TransportError::NodeResponseError(format!(
                "error_code {}",
                res.status()
            )));
        }

        let response_str = res
            .text()
            .await
            .map_err(|e| crate::TransportError::Utils(wallet_utils::Error::Http(e.into())))?;
        tracing::info!("[rpc response] = {}", response_str);

        let rpc_result = wallet_utils::serde_func::serde_from_str::<RpcResult<T>>(&response_str)?;
        if let Some(err) = rpc_result.error {
            return Err(TransportError::NodeResponseError(err.message));
        }

        match rpc_result.result {
            Some(res) => Ok(res),
            None => Err(TransportError::EmptyResult),
        }
    }

    pub async fn send_json_stream<T: DeserializeOwned + Debug>(
        self,
    ) -> Result<T, crate::TransportError> {
        let res = self
            .0
            .send()
            .await
            .map_err(|e| crate::TransportError::Utils(wallet_utils::Error::Http(e.into())))?;

        if !res.status().is_success() {
            return Err(crate::TransportError::NodeResponseError(format!(
                "error_code {}",
                res.status()
            )));
        }

        let response_bytes = res
            .bytes()
            .await
            .map_err(|e| crate::TransportError::Utils(wallet_utils::Error::Http(e.into())))?;

        let rpc_result = serde_json::from_slice::<RpcResult<T>>(&response_bytes).unwrap();
        if let Some(err) = rpc_result.error {
            return Err(TransportError::NodeResponseError(err.message));
        }

        match rpc_result.result {
            Some(res) => Ok(res),
            None => Err(TransportError::EmptyResult),
        }
    }
}
