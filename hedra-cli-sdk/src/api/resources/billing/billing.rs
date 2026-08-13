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
    ///     client.billing.get_balance(None).await;
    /// }
    /// ```
    pub async fn get_balance(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<BalanceResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::GET, "balance", None, None, options)
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
    ///         .billing
    ///         .get_usage(
    ///             &GetUsageQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn get_usage(
        &self,
        request: &GetUsageQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<UsageResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
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
