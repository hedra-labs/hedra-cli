use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, QueryBuilder, RequestOptions};
use reqwest::Method;

pub struct KeysClient {
    pub http_client: HttpClient,
}

impl KeysClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }

    pub async fn list(
        &self,
        request: &KeysListQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<KeyListResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                "keys",
                None,
                QueryBuilder::new()
                    .serialize("workspace_id", request.workspace_id.clone())
                    .build(),
                options,
            )
            .await
    }

    pub async fn create(
        &self,
        request: &KeyCreateRequest,
        options: Option<RequestOptions>,
    ) -> Result<KeyCreateResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::POST,
                "keys",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    pub async fn rotate(
        &self,
        key_id: &str,
        request: &KeyRotateRequest,
        options: Option<RequestOptions>,
    ) -> Result<KeyRotateResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::POST,
                &format!("keys/{}/rotate", key_id),
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    pub async fn revoke(
        &self,
        key_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<(), ApiError> {
        self.http_client
            .execute_request(
                Method::DELETE,
                &format!("keys/{}", key_id),
                None,
                None,
                options,
            )
            .await
    }
}
