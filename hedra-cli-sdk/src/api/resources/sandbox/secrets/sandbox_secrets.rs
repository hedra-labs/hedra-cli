use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, RequestOptions};
use reqwest::Method;

pub struct SecretsClient {
    pub http_client: HttpClient,
}

impl SecretsClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }

    /// The names of your workspace's secrets, in one page. A secret's value is
    /// never returned.
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
    ///     client.sandbox.secrets.ls(None).await;
    /// }
    /// ```
    pub async fn ls(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<SandboxSecretListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::GET, "sandboxes/secrets", None, None, options)
            .await
    }

    /// Store a secret that any member's sandbox can name in `secrets`: 201
    /// for a new name, 200 when it replaces the value of a name your workspace
    /// holds. Only a workspace owner or admin can set one.
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
    ///         .sandbox
    ///         .secrets
    ///         .set(
    ///             &SetSandboxSecretRequest {
    ///                 name: "name".to_string(),
    ///                 value: "value".to_string(),
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn set(
        &self,
        request: &SetSandboxSecretRequest,
        options: Option<RequestOptions>,
    ) -> Result<SandboxSecret, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "sandboxes/secrets",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Delete a secret. Only a workspace owner or admin can delete one.
    ///
    /// # Arguments
    ///
    /// * `name` - The secret's name.
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
    ///     client.sandbox.secrets.rm(&"name".to_string(), None).await;
    /// }
    /// ```
    pub async fn rm(
        &self,
        name: &str,
        options: Option<RequestOptions>,
    ) -> Result<SandboxSecret, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::DELETE,
                &format!("sandboxes/secrets/{}", name),
                None,
                None,
                options,
            )
            .await
    }
}
