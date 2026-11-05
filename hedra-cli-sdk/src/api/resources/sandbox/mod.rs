use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, QueryBuilder, RequestOptions};
use reqwest::Method;

pub mod secrets;
pub use secrets::SecretsClient;
pub struct SandboxClient {
    pub http_client: HttpClient,
    pub secrets: SecretsClient,
}

impl SandboxClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
            secrets: SecretsClient::new(config.clone())?,
        })
    }

    /// Every GPU model your workspace can run, at its price after your
    /// discount. 404 while sandboxes are not enabled for your workspace.
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
    ///     client.sandbox.gpus(None).await;
    /// }
    /// ```
    pub async fn gpus(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<SandboxGpuListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::GET, "sandboxes/gpus", None, None, options)
            .await
    }

    /// How many more GPUs your workspace's sandboxes can hold under its GPU
    /// quota. 404 while sandboxes are not enabled for your workspace.
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
    ///     client.sandbox.quota(None).await;
    /// }
    /// ```
    pub async fn quota(&self, options: Option<RequestOptions>) -> Result<SandboxQuota, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::GET, "sandboxes/quota", None, None, options)
            .await
    }

    /// Revoke every session JWT and endpoint token of your workspace's
    /// sandboxes, every member's included; mint new ones after it answers. Only
    /// a workspace owner or admin can rotate the key.
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
    ///     client.sandbox.rotate_signing_key(None).await;
    /// }
    /// ```
    pub async fn rotate_signing_key(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<SandboxSigningKeyRotation, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "sandboxes/signing-key/rotate",
                None,
                None,
                options,
            )
            .await
    }

    /// Your sandboxes in this workspace, newest first, without `instances`
    /// and `started_at`: the creating, active and stopping ones, or every one
    /// with `all=true`. A cursor resumes only under the `name` and `all` it was
    /// issued with.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum items per page.
    /// * `cursor` - Opaque cursor from the previous page's `next_cursor`; omit for the first page.
    /// * `name` - Only sandboxes with exactly this name. A name is unique among your creating, active and stopping sandboxes, so without `all=true` the page holds at most one.
    /// * `all` - With `true`, sandboxes in every status; with `false`, the default, only creating, active and stopping sandboxes.
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
    ///         .ls(
    ///             &LsQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn ls(
        &self,
        request: &LsQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<SandboxListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                "sandboxes",
                None,
                QueryBuilder::new()
                    .int("limit", request.limit.clone())
                    .serialize("cursor", request.cursor.clone())
                    .serialize("name", request.name.clone())
                    .bool("all", request.all.clone())
                    .build(),
                options,
            )
            .await
    }

    /// Start one instance per replica, paying the first lease from the API
    /// wallet before any instance is created. The sandbox runs until you stop
    /// it, its command exits, the wallet cannot pay a lease, or its
    /// `ttl_seconds` ends, when you give one. 404 while sandboxes are not enabled
    /// for your workspace.
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
    ///         .create(
    ///             &CreateSandboxRequest {
    ///                 name: "name".to_string(),
    ///                 image: "image".to_string(),
    ///                 gpu: "gpu".to_string(),
    ///                 replicas: None,
    ///                 resources: None,
    ///                 command: None,
    ///                 idle: None,
    ///                 restart: None,
    ///                 env: None,
    ///                 ttl_seconds: None,
    ///                 idle_ttl_seconds: None,
    ///                 ports: None,
    ///                 secrets: None,
    ///                 egress: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn create(
        &self,
        request: &CreateSandboxRequest,
        options: Option<RequestOptions>,
    ) -> Result<Sandbox, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "sandboxes",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// One sandbox with its instances and its charge so far.
    ///
    /// # Arguments
    ///
    /// * `sandbox` - The sandbox's id (`sandbox_<uuid>`), or its name: your creating, active or stopping sandbox of that name, else your newest.
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
    ///     client.sandbox.get(&"sandbox".to_string(), None).await;
    /// }
    /// ```
    pub async fn get(
        &self,
        sandbox: &str,
        options: Option<RequestOptions>,
    ) -> Result<Sandbox, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("sandboxes/{}", sandbox),
                None,
                None,
                options,
            )
            .await
    }

    /// Stop every instance. Answers the sandbox, also when it is already
    /// stopping, ended or failed; settlement refunds the unused part of its
    /// lease once every instance has stopped.
    ///
    /// # Arguments
    ///
    /// * `sandbox` - The sandbox's id (`sandbox_<uuid>`), or its name: your creating, active or stopping sandbox of that name, else your newest.
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
    ///     client.sandbox.stop(&"sandbox".to_string(), None).await;
    /// }
    /// ```
    pub async fn stop(
        &self,
        sandbox: &str,
        options: Option<RequestOptions>,
    ) -> Result<Sandbox, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                &format!("sandboxes/{}/stop", sandbox),
                None,
                None,
                options,
            )
            .await
    }

    /// The newest log output of one instance, and whether older output was
    /// cut.
    ///
    /// # Arguments
    ///
    /// * `sandbox` - The sandbox's id (`sandbox_<uuid>`), or its name: your creating, active or stopping sandbox of that name, else your newest.
    /// * `replica` - The instance's replica number, from 0.
    /// * `since` - Only output written at or after this instant: an RFC 3339 timestamp with an offset and up to nanoseconds, such as `2026-11-02T18:32:30.123456789Z`. It is rounded up to a whole microsecond.
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
    ///         .saved_logs(
    ///             &"sandbox".to_string(),
    ///             &SavedLogsQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn saved_logs(
        &self,
        sandbox: &str,
        request: &SavedLogsQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<SandboxLogs, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("sandboxes/{}/logs", sandbox),
                None,
                QueryBuilder::new()
                    .int("replica", request.replica.clone())
                    .serialize("since", request.since.clone())
                    .build(),
                options,
            )
            .await
    }

    /// The lifecycle events of one instance.
    ///
    /// # Arguments
    ///
    /// * `sandbox` - The sandbox's id (`sandbox_<uuid>`), or its name: your creating, active or stopping sandbox of that name, else your newest.
    /// * `replica` - The instance's replica number, from 0.
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
    ///         .events(
    ///             &"sandbox".to_string(),
    ///             &EventsQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn events(
        &self,
        sandbox: &str,
        request: &EventsQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<SandboxEventListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("sandboxes/{}/events", sandbox),
                None,
                QueryBuilder::new()
                    .int("replica", request.replica.clone())
                    .build(),
                options,
            )
            .await
    }

    /// A short-lived session JWT for one instance of an `active` sandbox,
    /// to open `Logs` streams on the broker at `broker_url`; `PortForward` too
    /// when the key holds `sandboxes:write`, and `Exec` and `Invoke` when it
    /// holds `sandboxes:exec`. Mint a new one for each stream opened after
    /// `expires_at`.
    ///
    /// # Arguments
    ///
    /// * `sandbox` - The sandbox's id (`sandbox_<uuid>`), or its name: your creating, active or stopping sandbox of that name, else your newest.
    /// * `replica` - The instance's replica number, from 0.
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
    ///         .session(
    ///             &"sandbox".to_string(),
    ///             &SessionQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn session(
        &self,
        sandbox: &str,
        request: &SessionQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<SandboxSession, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                &format!("sandboxes/{}/session", sandbox),
                None,
                QueryBuilder::new()
                    .int("replica", request.replica.clone())
                    .build(),
                options,
            )
            .await
    }

    /// A token for the `url` of an `active` sandbox created with `ports`, sent
    /// as `Authorization: Bearer`, and `browser_url`, the link that opens `url`
    /// in a browser. Both work for every replica until the end of the sandbox's
    /// `ttl_seconds` or 30 days after the mint, whichever is earlier; rotating
    /// the signing key revokes them.
    ///
    /// # Arguments
    ///
    /// * `sandbox` - The sandbox's id (`sandbox_<uuid>`), or its name: your creating, active or stopping sandbox of that name, else your newest.
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
    ///     client.sandbox.token(&"sandbox".to_string(), None).await;
    /// }
    /// ```
    pub async fn token(
        &self,
        sandbox: &str,
        options: Option<RequestOptions>,
    ) -> Result<SandboxEndpointToken, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.22.0".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                &format!("sandboxes/{}/endpoint-token", sandbox),
                None,
                None,
                options,
            )
            .await
    }
}
