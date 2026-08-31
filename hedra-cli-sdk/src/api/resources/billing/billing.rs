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
                .or_insert_with(|| "3.15.5".to_string());
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
                .or_insert_with(|| "3.15.5".to_string());
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

    /// Every movement of the API wallet's balance, newest first: funds added,
    /// jobs charged, charges refunded, and corrections. Scoped to the workspace
    /// the credential bills, the same one `GET /v3/balance` reports.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum items per page.
    /// * `cursor` - Opaque cursor from the previous page's `next_cursor`; omit for the first page.
    /// * `options` - Additional request options such as headers, timeout, etc.
    ///
    /// # Returns
    ///
    /// JSON response from the API
    ///
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
    ///         .list_transactions(
    ///             &ListTransactionsQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn list_transactions(
        &self,
        request: &ListTransactionsQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<TransactionListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.15.5".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                "transactions",
                None,
                QueryBuilder::new()
                    .int("limit", request.limit.clone())
                    .serialize("cursor", request.cursor.clone())
                    .build(),
                options,
            )
            .await
    }
}
