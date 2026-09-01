use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, QueryBuilder, RequestOptions};
use reqwest::Method;

pub struct WebhooksClient {
    pub http_client: HttpClient,
}

impl WebhooksClient {
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
    ///     client.webhooks.get_public_key(None).await;
    /// }
    /// ```
    pub async fn get_public_key(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<WebhookPublicKey, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.16.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::GET, "webhooks/public-key", None, None, options)
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
    ///     client.webhooks.get_default(None).await;
    /// }
    /// ```
    pub async fn get_default(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<WebhookDefaultConfig, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.16.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::GET, "webhooks/default", None, None, options)
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
    ///         .webhooks
    ///         .put_default(
    ///             &WebhookDefaultUpdate {
    ///                 url: "url".to_string(),
    ///                 enabled: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn put_default(
        &self,
        request: &WebhookDefaultUpdate,
        options: Option<RequestOptions>,
    ) -> Result<WebhookDefaultConfig, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.16.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::PUT,
                "webhooks/default",
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
    ///     client.webhooks.delete_default(None).await;
    /// }
    /// ```
    pub async fn delete_default(&self, options: Option<RequestOptions>) -> Result<(), ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.16.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::DELETE, "webhooks/default", None, None, options)
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
    ///     client.webhooks.test_default(None).await;
    /// }
    /// ```
    pub async fn test_default(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<WebhookTestResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.16.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::POST, "webhooks/default/test", None, None, options)
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
    ///         .webhooks
    ///         .list_deliveries(
    ///             &ListDeliveriesQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn list_deliveries(
        &self,
        request: &ListDeliveriesQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<WebhookDeliveryListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.16.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                "webhooks/deliveries",
                None,
                QueryBuilder::new()
                    .int("limit", request.limit.clone())
                    .serialize("cursor", request.cursor.clone())
                    .build(),
                options,
            )
            .await
    }

    /// Replay a finished delivery: reset it to PENDING and re-fire the signed POST.
    ///
    /// 404 if the delivery isn't visible to the caller; 409 if a delivery for the
    /// request is still in flight (a replay must not stack on it). The delivery is
    /// re-validated (SSRF) and re-signed at send time, and the receiver dedupes on
    /// ``X-Hedra-Webhook-Id``, so a replay is safe.
    ///
    /// The webhook id is stable across the original and every replay, because it
    /// identifies the event. Every attempt of a replayed cycle therefore also carries
    /// ``X-Hedra-Webhook-Redelivery: true`` — without it a receiver doing exactly what
    /// our guidance says (dedupe on the id) would silently discard the replay, which is
    /// the one case where the duplicate is the point.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The job's id (`job_<uuid>`).
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
    ///     client.webhooks.redeliver(&"job_id".to_string(), None).await;
    /// }
    /// ```
    pub async fn redeliver(
        &self,
        job_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<WebhookDeliverySummary, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.16.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                &format!("webhooks/deliveries/{}/redeliver", job_id),
                None,
                None,
                options,
            )
            .await
    }
}
