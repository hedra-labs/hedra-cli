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

    pub async fn get_public_key(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<WebhookPublicKey, ApiError> {
        self.http_client
            .execute_request(Method::GET, "webhooks/public-key", None, None, options)
            .await
    }

    pub async fn get_default(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<WebhookDefaultConfig, ApiError> {
        self.http_client
            .execute_request(Method::GET, "webhooks/default", None, None, options)
            .await
    }

    pub async fn put_default(
        &self,
        request: &WebhookDefaultUpdate,
        options: Option<RequestOptions>,
    ) -> Result<WebhookDefaultConfig, ApiError> {
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

    pub async fn delete_default(&self, options: Option<RequestOptions>) -> Result<(), ApiError> {
        self.http_client
            .execute_request(Method::DELETE, "webhooks/default", None, None, options)
            .await
    }

    pub async fn test_default(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<WebhookTestResponse, ApiError> {
        self.http_client
            .execute_request(Method::POST, "webhooks/default/test", None, None, options)
            .await
    }

    pub async fn list_deliveries(
        &self,
        request: &ListDeliveriesQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<WebhookDeliveryListResponse, ApiError> {
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
    /// * `options` - Additional request options such as headers, timeout, etc.
    ///
    /// # Returns
    ///
    /// JSON response from the API
    pub async fn redeliver(
        &self,
        job_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<WebhookDeliverySummary, ApiError> {
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
