use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, RequestOptions};
use reqwest::Method;

pub struct LogDrainsClient {
    pub http_client: HttpClient,
}

impl LogDrainsClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }

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
    ///     client.log_drains.list_log_drains(None).await;
    /// }
    /// ```
    pub async fn list_log_drains(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<LogDrainListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.9.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::GET, "log-drains", None, None, options)
            .await
    }

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
    ///         .log_drains
    ///         .create_log_drain(
    ///             &LogDrainCreate {
    ///                 name: "name".to_string(),
    ///                 url: "url".to_string(),
    ///                 format: None,
    ///                 secret: None,
    ///                 headers: None,
    ///                 enabled: None,
    ///                 batch_size: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn create_log_drain(
        &self,
        request: &LogDrainCreate,
        options: Option<RequestOptions>,
    ) -> Result<LogDrainConfig, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.9.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "log-drains",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

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
    ///         .log_drains
    ///         .get_log_drain(&"drain_id".to_string(), None)
    ///         .await;
    /// }
    /// ```
    pub async fn get_log_drain(
        &self,
        drain_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<LogDrainConfig, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.9.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("log-drains/{}", drain_id),
                None,
                None,
                options,
            )
            .await
    }

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
    ///         .log_drains
    ///         .delete_log_drain(&"drain_id".to_string(), None)
    ///         .await;
    /// }
    /// ```
    pub async fn delete_log_drain(
        &self,
        drain_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<(), ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.9.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::DELETE,
                &format!("log-drains/{}", drain_id),
                None,
                None,
                options,
            )
            .await
    }

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
    ///         .log_drains
    ///         .update_log_drain(
    ///             &"drain_id".to_string(),
    ///             &LogDrainUpdate {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn update_log_drain(
        &self,
        drain_id: &str,
        request: &LogDrainUpdate,
        options: Option<RequestOptions>,
    ) -> Result<LogDrainConfig, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.9.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::PATCH,
                &format!("log-drains/{}", drain_id),
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

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
    ///         .log_drains
    ///         .test_log_drain(&"drain_id".to_string(), None)
    ///         .await;
    /// }
    /// ```
    pub async fn test_log_drain(
        &self,
        drain_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<LogDrainTestResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.9.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                &format!("log-drains/{}/test", drain_id),
                None,
                None,
                options,
            )
            .await
    }
}
