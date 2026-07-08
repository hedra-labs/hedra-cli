use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, RequestOptions};
use reqwest::Method;

pub struct FilesClient {
    pub http_client: HttpClient,
}

impl FilesClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }

    pub async fn upload(
        &self,
        request: &UploadRequest,
        options: Option<RequestOptions>,
    ) -> Result<FileUploadResponse, ApiError> {
        self.http_client
            .execute_multipart_request(
                Method::POST,
                "files",
                request.clone().to_multipart(),
                None,
                options,
            )
            .await
    }
}
