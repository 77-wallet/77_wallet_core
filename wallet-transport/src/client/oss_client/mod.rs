use oss::Oss;
use request::RequestBuilder;

mod async_impl;
mod auth;
mod entity;
pub(crate) mod error;
mod macros;
mod metadata;
mod oss;
mod request;
mod url;
mod util;

pub const ACCESS_KEY_ID: &str = "";
pub const ACCESS_KEY_SECRET: &str = "";
pub const BUCKET_NAME: &str = "ossbuk23";

pub const ENDPOINT: &str = "https://oss-cn-hongkong.aliyuncs.com/";

#[derive(Debug, Clone)]
pub struct OssClient {
    oss: Oss,
    builder: RequestBuilder,
}

impl OssClient {
    pub fn new() -> Self {
        let oss = Oss::new(ACCESS_KEY_ID, ACCESS_KEY_SECRET, ENDPOINT, BUCKET_NAME);
        let builder = request::RequestBuilder::new()
            // .oss_header_put("Transfer-Encoding", "chuncked")
            .with_expire(60);
        Self { oss, builder }
    }

    pub async fn upload_local_file(
        &self,
        src_file_path: &str,
        dst_file_name: &str,
    ) -> Result<(), crate::errors::TransportError> {
        let oss = &self.oss;
        let builder = self.builder.clone();
        // let oss_path = format!("/logs/{}", file_name);
        let oss_path = format!("logs/{}", dst_file_name);
        // let oss_path = format!("./");
        tracing::warn!(
            "[upload_local_file] oss_path: {}, file_path: {}",
            oss_path,
            src_file_path
        );

        oss.put_object_from_file(oss_path, src_file_path.to_string(), builder)
            .await?;
        Ok(())
    }

    pub async fn get_object(
        &self,
        file_name: &str,
    ) -> Result<Vec<u8>, crate::errors::TransportError> {
        let oss = &self.oss;
        let builder = self.builder.clone();
        // .oss_header_put("Content-Length", &metadata.to_string());
        let oss_path = format!("/logs/{}", file_name);
        tracing::warn!("oss_path: {}", oss_path);
        let res = oss.get_object(oss_path, builder).await?;
        Ok(res)
    }

    pub async fn get_object_metadata(
        &self,
        file_name: &str,
    ) -> Result<metadata::ObjectMetadata, crate::errors::TransportError> {
        let oss = &self.oss;
        let builder = self.builder.clone();
        // .oss_header_put("Content-Length", &metadata.to_string());
        let oss_path = format!("/logs/{}", file_name);
        tracing::warn!("oss_path: {}", oss_path);
        let res = oss.get_object_metadata(oss_path, builder).await?;
        Ok(res)
    }

    pub async fn upload_buffer(
        &self,
        file_path: &str,
        file_name: &str,
    ) -> Result<(), crate::errors::TransportError> {
        let oss = &self.oss;
        let builder = self.builder.clone();
        let oss_path = format!("logs/{}", file_name);
        let buffer = std::fs::read(file_path).unwrap();

        oss.pub_object_from_buffer(oss_path, &buffer, builder)
            .await?;
        Ok(())
    }
}

impl Default for OssClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_oss_client() {
        let oss_client = OssClient::new();
        let file_path =
            "/Users/qiuwenjing/workspace/work/rust/wallet-sdk/wallet-transport/test.txt";
        let file_name = "test.txt";
        let result = oss_client.upload_local_file(file_path, file_name).await;
        println!("result: {result:?}");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_buffer() {
        let oss_client = OssClient::new();
        let file_path =
            "/Users/qiuwenjing/workspace/work/rust/wallet-sdk/wallet-transport/test.txt";
        let file_name = "test.txt";
        let result = oss_client.upload_buffer(file_path, file_name).await;
        println!("result: {result:?}");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_object() {
        let oss_client = OssClient::new();
        let _file_name = "test.txt";
        let file_name = "sdk:2024-10-07 10:36:00.txt";
        let result = oss_client.get_object(file_name).await.unwrap();
        println!(
            "file content: {}",
            String::from_utf8_lossy(result.as_slice())
        );
    }

    #[tokio::test]
    async fn test_get_object_metadata() {
        let oss_client = OssClient::new();
        let file_name = "test.txt";
        let result = oss_client.get_object_metadata(file_name).await.unwrap();
        println!("result: {:?}", result);
    }
}
