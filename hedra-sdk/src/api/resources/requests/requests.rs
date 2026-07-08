use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, QueryBuilder, RequestOptions};
use reqwest::Method;

pub struct RequestsClient {
    pub http_client: HttpClient,
}

impl RequestsClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }

    pub async fn list(
        &self,
        request: &RequestsListQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<RequestListResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                "requests",
                None,
                QueryBuilder::new()
                    .int("limit", request.limit.clone())
                    .serialize("cursor", request.cursor.clone())
                    .build(),
                options,
            )
            .await
    }

    pub async fn get(
        &self,
        request_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<ResultResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                &format!("requests/{}", request_id),
                None,
                None,
                options,
            )
            .await
    }

    pub async fn get_status(
        &self,
        request_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<StatusResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                &format!("requests/{}/status", request_id),
                None,
                None,
                options,
            )
            .await
    }

    pub async fn stream(
        &self,
        request_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<serde_json::Value, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                &format!("requests/{}/stream", request_id),
                None,
                None,
                options,
            )
            .await
    }
}
