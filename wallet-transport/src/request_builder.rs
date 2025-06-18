use crate::{TransportError, errors::NodeResponseError, types::JsonRpcResult};
use reqwest::RequestBuilder;
use serde::{Serialize, de::DeserializeOwned};
use std::{fmt::Debug, str::FromStr};

pub struct ReqBuilder(pub RequestBuilder);

impl ReqBuilder {
    pub fn json(mut self, v: impl Serialize + Debug) -> Self {
        tracing::debug!("request params: {}", serde_json::to_string(&v).unwrap());
        self.0 = self.0.json(&v);
        self
    }

    pub fn query(mut self, v: impl Serialize + Debug) -> Self {
        tracing::debug!("request params: {:?}", v);
        self.0 = self.0.query(&v);
        self
    }

    pub fn body(mut self, body: String) -> Self {
        tracing::debug!("request params: {:?}", body);
        self.0 = self.0.body(body);
        self
    }

    pub async fn do_request(self) -> Result<String, crate::TransportError> {
        let res = self
            .0
            .send()
            .await
            .map_err(|e| TransportError::Utils(wallet_utils::Error::Http(e.into())))?;

        let status = res.status();
        if !status.is_success() {
            // 尝试解析出 json response:: btc now node 返回的不标准。
            match res.text().await {
                Ok(response) => {
                    if let Ok(rs) = try_to_paras_json(&response) {
                        return Err(TransportError::NodeResponseError(NodeResponseError::new(
                            rs.0,
                            Some(rs.1),
                        )));
                    } else {
                        // 尝试提出error
                        let message = match serde_json::Value::from_str(&response) {
                            Ok(value) => {
                                let e = value.get("error");
                                e.map(|e| e.to_string())
                            }
                            Err(_) => None,
                        };
                        return Err(TransportError::NodeResponseError(NodeResponseError::new(
                            status.as_u16() as i64,
                            message,
                        )));
                    }
                }
                Err(e) => {
                    return Err(TransportError::NodeResponseError(NodeResponseError::new(
                        status.as_u16() as i64,
                        Some(e.to_string()),
                    )));
                }
            }
        }

        let response = res
            .text()
            .await
            .map_err(|e| crate::TransportError::Utils(wallet_utils::Error::Http(e.into())))?;

        // tracing::info!("response = {}", response);
        Ok(response)
    }

    // 普通请求
    pub async fn send<T: DeserializeOwned>(self) -> Result<T, crate::TransportError> {
        let res = self.do_request().await?;
        Ok(wallet_utils::serde_func::serde_from_str(&res)?)
    }
}

fn try_to_paras_json(res: &str) -> Result<(i64, String), crate::TransportError> {
    if let Ok(reg) = regex::Regex::new(r#"\{.*\}"#) {
        let res = reg.find(res).map(|m| m.as_str().to_string());
        if let Some(res) = res {
            let res = res.replace("\\\"", "\"");
            let response = wallet_utils::serde_func::serde_from_str::<JsonRpcResult>(&res);

            if let Ok(res) = response {
                if let Some(res) = res.error {
                    return Ok((res.code, res.message));
                }
            }
        }
    }
    Err(crate::TransportError::EmptyResult)
}
