use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, QueryBuilder, RequestOptions};
use reqwest::Method;
use std::collections::HashMap;

pub struct ModelsClient {
    pub http_client: HttpClient,
}

impl ModelsClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }

    pub async fn list(
        &self,
        request: &ModelsListQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<ModelListResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                "models",
                None,
                QueryBuilder::new()
                    .serialize("modality", request.modality.clone())
                    .build(),
                options,
            )
            .await
    }

    pub async fn get(
        &self,
        model: &str,
        options: Option<RequestOptions>,
    ) -> Result<ModelDetail, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                &format!("models/{}", model),
                None,
                None,
                options,
            )
            .await
    }

    pub async fn list_model_jobs(
        &self,
        model: &str,
        request: &ListModelJobsQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<JobListResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                &format!("models/{}/jobs", model),
                None,
                QueryBuilder::new()
                    .int("limit", request.limit.clone())
                    .serialize("cursor", request.cursor.clone())
                    .build(),
                options,
            )
            .await
    }

    /// Voices this model accepts — scoped to the model's voice provider.
    ///
    /// # Arguments
    ///
    /// * `options` - Additional request options such as headers, timeout, etc.
    ///
    /// # Returns
    ///
    /// JSON response from the API
    pub async fn list_voices(
        &self,
        model: &str,
        options: Option<RequestOptions>,
    ) -> Result<VoiceListResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                &format!("models/{}/voices", model),
                None,
                None,
                options,
            )
            .await
    }

    /// A standalone one-operation OpenAPI spec for this model's submit call.
    ///
    /// # Arguments
    ///
    /// * `options` - Additional request options such as headers, timeout, etc.
    ///
    /// # Returns
    ///
    /// JSON response from the API
    pub async fn get_openapi(
        &self,
        model: &str,
        options: Option<RequestOptions>,
    ) -> Result<HashMap<String, serde_json::Value>, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                &format!("models/{}/openapi.json", model),
                None,
                None,
                options,
            )
            .await
    }

    pub async fn estimate(
        &self,
        model: &str,
        request: &EstimateRequest,
        options: Option<RequestOptions>,
    ) -> Result<EstimateResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::POST,
                &format!("models/{}/estimate", model),
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }
}
