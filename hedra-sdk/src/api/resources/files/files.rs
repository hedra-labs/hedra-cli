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

    /// Store a file and return a short-lived URL to pass in a model's `input`.
    ///
    /// Free, and available on an empty API wallet — funding is enforced when you
    /// submit a generation, not when you upload its inputs. `GET /v3/balance`
    /// reports what the wallet holds.
    ///
    /// # Arguments
    ///
    /// * `options` - Additional request options such as headers, timeout, etc.
    ///
    /// # Returns
    ///
    /// JSON response from the API
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
