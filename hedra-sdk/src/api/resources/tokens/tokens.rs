use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, RequestOptions};
use reqwest::Method;

pub struct TokensClient {
    pub http_client: HttpClient,
}

impl TokensClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }

    pub async fn create(
        &self,
        request: &TokenCreateRequest,
        options: Option<RequestOptions>,
    ) -> Result<TokenCreateResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::POST,
                "tokens",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }
}
