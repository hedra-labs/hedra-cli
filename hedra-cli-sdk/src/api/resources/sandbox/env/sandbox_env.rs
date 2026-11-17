use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, QueryBuilder, RequestOptions};
use reqwest::Method;

pub struct EnvClient {
    pub http_client: HttpClient,
}

impl EnvClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }

    /// Your workspace's Config and Secret variables by name, in one page,
    /// without their values. Read one Config by name for its value; a Secret's
    /// value is never returned.
    ///
    /// # Arguments
    ///
    /// * `type_` - Only variables of this type; omitted is both.
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
    ///         .sandbox
    ///         .env
    ///         .ls(
    ///             &SandboxEnvLsQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn ls(
        &self,
        request: &SandboxEnvLsQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<SandboxEnvVarListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.24.10".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                "sandboxes/environment-variables",
                None,
                QueryBuilder::new()
                    .serialize("type", request.r#type.clone())
                    .build(),
                options,
            )
            .await
    }

    /// Save a Config or Secret variable that any member's sandbox can
    /// reference in `env` as `{"ref": NAME}`: 201 for a new name, 200 when it
    /// replaces the value of a name your workspace holds as the same type. A
    /// sandbox reads the value when it starts; one already running keeps what it
    /// started with.
    ///
    /// # Arguments
    ///
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
    ///         .sandbox
    ///         .env
    ///         .set(
    ///             &SetSandboxEnvVarRequest {
    ///                 name: "name".to_string(),
    ///                 r#type: SandboxEnvVarType::Config,
    ///                 value: "value".to_string(),
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn set(
        &self,
        request: &SetSandboxEnvVarRequest,
        options: Option<RequestOptions>,
    ) -> Result<SandboxEnvVar, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.24.10".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "sandboxes/environment-variables",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// One variable: a Config with its `value`, a Secret without one. The
    /// answer is never cached.
    ///
    /// # Arguments
    ///
    /// * `name` - The variable's name.
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
    ///     client.sandbox.env.get(&"name".to_string(), None).await;
    /// }
    /// ```
    pub async fn get(
        &self,
        name: &str,
        options: Option<RequestOptions>,
    ) -> Result<GetEnvResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.24.10".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("sandboxes/environment-variables/{}", name),
                None,
                None,
                options,
            )
            .await
    }

    /// Delete a variable of either type. A sandbox already running keeps
    /// it; a later create that references it fails.
    ///
    /// # Arguments
    ///
    /// * `name` - The variable's name.
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
    ///     client.sandbox.env.rm(&"name".to_string(), None).await;
    /// }
    /// ```
    pub async fn rm(
        &self,
        name: &str,
        options: Option<RequestOptions>,
    ) -> Result<SandboxEnvVar, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.24.10".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::DELETE,
                &format!("sandboxes/environment-variables/{}", name),
                None,
                None,
                options,
            )
            .await
    }
}
