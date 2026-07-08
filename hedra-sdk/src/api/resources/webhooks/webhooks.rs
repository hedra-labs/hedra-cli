use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, RequestOptions};
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
}
