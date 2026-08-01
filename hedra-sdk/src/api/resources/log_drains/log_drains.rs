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

    pub async fn list_log_drains(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<LogDrainListResponse, ApiError> {
        self.http_client
            .execute_request(Method::GET, "log-drains", None, None, options)
            .await
    }

    pub async fn create_log_drain(
        &self,
        request: &LogDrainCreate,
        options: Option<RequestOptions>,
    ) -> Result<LogDrainConfig, ApiError> {
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

    pub async fn get_log_drain(
        &self,
        drain_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<LogDrainConfig, ApiError> {
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

    pub async fn delete_log_drain(
        &self,
        drain_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<(), ApiError> {
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

    pub async fn update_log_drain(
        &self,
        drain_id: &str,
        request: &LogDrainUpdate,
        options: Option<RequestOptions>,
    ) -> Result<LogDrainConfig, ApiError> {
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

    pub async fn test_log_drain(
        &self,
        drain_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<LogDrainTestResponse, ApiError> {
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
