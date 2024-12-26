use crate::{errors::NodeResponseError, types::RpcResult, TransportError};
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

    async fn do_request<T: DeserializeOwned>(self) -> Result<String, crate::TransportError> {
        let res = self
            .0
            .send()
            .await
            .map_err(|e| TransportError::Utils(wallet_utils::Error::Http(e.into())))?;

        let status = res.status();
        if !status.is_success() {
            return Err(TransportError::NodeResponseError(NodeResponseError::new(
                status.as_u16() as i64,
                None,
            )));
        }

        let response = res
            .text()
            .await
            .map_err(|e| crate::TransportError::Utils(wallet_utils::Error::Http(e.into())))?;

        tracing::info!("response = {}", response);
        Ok(response)
    }

    pub async fn send<T: DeserializeOwned>(self) -> Result<T, crate::TransportError> {
        let res = self.do_request::<T>().await?;

        Ok(wallet_utils::serde_func::serde_from_str(&res)?)
    }

    // 结果解析为JsonResut
    pub async fn send_json_rpc<T: DeserializeOwned>(self) -> Result<T, crate::TransportError> {
        let response = self.do_request::<T>().await?;
        let rpc_result = wallet_utils::serde_func::serde_from_str::<RpcResult<T>>(&response)?;

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
