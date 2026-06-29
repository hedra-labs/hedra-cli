//! Service clients and API endpoints
//!
//! This module provides the client implementations for all available services.

use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, QueryBuilder, RequestOptions};
use reqwest::Method;

pub struct ApiClient {
    pub config: ClientConfig,
    pub http_client: HttpClient,
}

impl ApiClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            config: config.clone(),
            http_client: HttpClient::new(config.clone())?,
        })
    }

    pub async fn list_models(
        &self,
        request: &ListModelsQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<Vec<AiModel>, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                "models",
                None,
                QueryBuilder::new()
                    .serialize("types", request.types.clone())
                    .build(),
                options,
            )
            .await
    }

    pub async fn list_voices(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<Vec<Asset>, ApiError> {
        self.http_client
            .execute_request(Method::GET, "voices", None, None, options)
            .await
    }

    pub async fn list_assets(
        &self,
        request: &ListAssetsQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<Vec<Asset>, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                "assets",
                None,
                QueryBuilder::new()
                    .serialize("type", Some(request.r#type.clone()))
                    .serialize("ids", request.ids.clone())
                    .build(),
                options,
            )
            .await
    }

    pub async fn create_asset(
        &self,
        request: &CreateAssetRequest,
        options: Option<RequestOptions>,
    ) -> Result<CreateAssetResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::POST,
                "assets",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    pub async fn upload_asset(
        &self,
        id: &str,
        request: &UploadAssetRequest,
        options: Option<RequestOptions>,
    ) -> Result<Asset, ApiError> {
        self.http_client
            .execute_multipart_request(
                Method::POST,
                &format!("assets/{}/upload", id),
                request.clone().to_multipart(),
                None,
                options,
            )
            .await
    }

    pub async fn list_generations(
        &self,
        request: &ListGenerationsQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<PagedResponseGeneration, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                "generations",
                None,
                QueryBuilder::new()
                    .serialize("type", request.r#type.clone())
                    .serialize("created_before", request.created_before.clone())
                    .serialize("created_after", request.created_after.clone())
                    .serialize("prompt_query", request.prompt_query.clone())
                    .serialize("agent_thread_id", request.agent_thread_id.clone())
                    .serialize("ids", request.ids.clone())
                    .int("limit", request.limit.clone())
                    .int("offset", request.offset.clone())
                    .build(),
                options,
            )
            .await
    }

    pub async fn generate_asset(
        &self,
        request: &GenerateAssetRequest,
        options: Option<RequestOptions>,
    ) -> Result<GenerateAssetResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::POST,
                "generations",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    pub async fn get_status(
        &self,
        generation_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<GenerationStatusResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                &format!("generations/{}/status", generation_id),
                None,
                None,
                options,
            )
            .await
    }

    pub async fn get_credits(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<CreditBalance, ApiError> {
        self.http_client
            .execute_request(Method::GET, "billing/credits", None, None, options)
            .await
    }
}
