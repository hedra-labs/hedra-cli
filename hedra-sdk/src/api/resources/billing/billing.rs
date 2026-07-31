use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, QueryBuilder, RequestOptions};
use reqwest::Method;

pub struct BillingClient {
    pub http_client: HttpClient,
}

impl BillingClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }

    pub async fn get_balance(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<BalanceResponse, ApiError> {
        self.http_client
            .execute_request(Method::GET, "balance", None, None, options)
            .await
    }

    pub async fn get_usage(
        &self,
        request: &GetUsageQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<UsageResponse, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                "usage",
                None,
                QueryBuilder::new()
                    .serialize("start", request.start.clone())
                    .serialize("end", request.end.clone())
                    .serialize("group_by", request.group_by.clone())
                    .build(),
                options,
            )
            .await
    }
}
