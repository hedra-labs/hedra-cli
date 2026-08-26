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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hedra_cli_sdk::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = ClientConfig {
    ///         token: Some("<token>".to_string()),
    ///         ..Default::default()
    ///     };
    ///     let client = HedraCliClient::new(config).expect("Failed to build client");
    ///     client
    ///         .files
    ///         .upload(
    ///             &UploadRequest {
    ///                 file: b"test file content".to_vec(),
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn upload(
        &self,
        request: &UploadRequest,
        options: Option<RequestOptions>,
    ) -> Result<FileUploadResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.13.3".to_string());
            Some(o)
        };
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
