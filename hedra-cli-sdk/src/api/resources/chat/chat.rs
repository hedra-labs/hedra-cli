use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, RequestOptions};
use reqwest::Method;

pub struct ChatClient {
    pub http_client: HttpClient,
}

impl ChatClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }

    /// OpenAI-compatible chat completions. Errors use the OpenAI error body, not the v3 envelope. An empty API wallet answers 402 (deliberate divergence from OpenAI's 429 `insufficient_quota`: retrying cannot fix an empty wallet).
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
    ///     client.chat.completions_create(None).await;
    /// }
    /// ```
    pub async fn completions_create(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<serde_json::Value, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.13.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::POST, "chat/completions", None, None, options)
            .await
    }

    /// OpenAI-compatible model list for the chat surface: exactly `{"object": "list", "data": [...]}` with additive extension fields per model. The published rate card here is the pricing reference for chat completions.
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
    ///     client.chat.llms_list(None).await;
    /// }
    /// ```
    pub async fn llms_list(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<LlmModelList, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.13.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::GET, "llms", None, None, options)
            .await
    }

    /// A single OpenAI-shaped model object with additive extensions.
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
    ///     client.chat.llms_get(&"model".to_string(), None).await;
    /// }
    /// ```
    pub async fn llms_get(
        &self,
        model: &str,
        options: Option<RequestOptions>,
    ) -> Result<LlmModelObject, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.13.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::GET, &format!("llms/{}", model), None, None, options)
            .await
    }
}
