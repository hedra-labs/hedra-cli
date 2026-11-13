//! The v3 sandbox client: the generated sandbox client over executors of
//! the verbs' own, with retries disabled and a base URL the verbs set.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use fern_cli_sdk::openapi::discovery::RetriesConfig;
use fern_cli_sdk::openapi::AppContext;
use fern_cli_sdk::sdk_executor::{CliExecutor, SdkError, SdkRequestExecutor};
use hedra_cli_sdk::api::resources::sandbox::SecretsClient;
use hedra_cli_sdk::api::{SandboxClient, SavedLogsQueryRequest};
use hedra_cli_sdk::{ApiError, ClientConfig, HttpClient, RequestExecutor};

use super::error::{Class, Failure, HttpError};

/// Each call's attempt timeout, unless a verb sets its own.
pub(crate) const ATTEMPT: Duration = Duration::from_secs(15);
/// The stored copy's read: up to 8 MiB, which takes some 70 s at 1 Mbit/s,
/// often over the slow link that sent `logs` to it.
const STORED_READ: Duration = Duration::from_secs(120);

/// How a v3 call failed.
#[derive(Debug)]
pub(crate) enum CallError {
    /// The route answered an error status.
    Http(HttpError),
    /// No answer: a transport failure.
    Transport(String),
    /// No answer within the attempt timeout.
    Timeout,
    /// No credential to send.
    NoCredential(String),
    /// An answer the client could not read.
    Unreadable(String),
}

impl CallError {
    /// The exit-code class and line. `subject` names the sandbox asked for.
    pub(crate) fn failure(&self, subject: Option<&str>) -> Failure {
        match self {
            CallError::Http(e) => e.classify(subject),
            CallError::Transport(detail) => Failure::transport(detail),
            CallError::Timeout => Failure::transport("no answer in time"),
            CallError::NoCredential(_) => Failure::new(
                Class::NotSignedIn,
                "Sign in with `hedra-cli auth login`, or set HEDRA_API_KEY.",
            ),
            CallError::Unreadable(detail) => Failure::new(
                Class::Internal,
                format!("Hedra sent an answer hedra-cli could not read: {detail}"),
            ),
        }
    }

    /// A failure that may pass on a retry: no answer, or a 408, 429 or 5xx.
    pub(crate) fn transient(&self) -> bool {
        match self {
            CallError::Http(h) => h.status == 408 || h.status == 429 || h.status >= 500,
            CallError::Transport(_) | CallError::Timeout => true,
            CallError::NoCredential(_) | CallError::Unreadable(_) => false,
        }
    }

    pub(crate) fn http(&self) -> Option<&HttpError> {
        match self {
            CallError::Http(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ApiError> for CallError {
    fn from(e: ApiError) -> Self {
        match e {
            ApiError::Http { status, message } => CallError::Http(HttpError::new(status, &message)),
            ApiError::Executor(boxed) => match boxed.downcast::<SdkError>() {
                Ok(sdk) => match *sdk {
                    SdkError::Http { status, body } => {
                        CallError::Http(HttpError::new(status, &body))
                    }
                    SdkError::Network(m) | SdkError::Timeout(m) => CallError::Transport(m),
                    SdkError::Auth(m) => CallError::NoCredential(m),
                    SdkError::Other(m) => CallError::Transport(m),
                },
                Err(other) => CallError::Transport(other.to_string()),
            },
            ApiError::Network(e) => CallError::Transport(e.to_string()),
            ApiError::Serialization(e) => CallError::Unreadable(e.to_string()),
            other => CallError::Unreadable(other.to_string()),
        }
    }
}

/// Runs one attempt under `limit`.
pub(crate) async fn attempt<T>(
    limit: Duration,
    call: impl Future<Output = Result<T, ApiError>>,
) -> Result<T, CallError> {
    match tokio::time::timeout(limit, call).await {
        Ok(result) => result.map_err(CallError::from),
        Err(_) => Err(CallError::Timeout),
    }
}

/// The executor adapter, recording each answer's `Date` for the session's
/// life (`Api::last_date`).
struct Adapter(
    Arc<CliExecutor>,
    Arc<std::sync::Mutex<Option<std::time::SystemTime>>>,
);

impl RequestExecutor for Adapter {
    fn execute(
        &self,
        request: reqwest::Request,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let response = SdkRequestExecutor::execute(&*self.0, request)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            let date = response
                .headers()
                .get(reqwest::header::DATE)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| httpdate::parse_http_date(v).ok());
            if let Ok(mut last) = self.1.lock() {
                *last = date;
            }
            Ok(response)
        })
    }
}

/// The sandbox client, twice: `client` honours `--debug`, and `quiet`
/// never dumps a request, for the calls whose bodies carry env values or a
/// secret.
pub(crate) struct Api {
    pub(crate) client: SandboxClient,
    pub(crate) quiet: SandboxClient,
    pub(crate) debug: bool,
    date: Arc<std::sync::Mutex<Option<std::time::SystemTime>>>,
}

impl Api {
    pub(crate) fn new(ctx: &AppContext, debug: bool) -> Self {
        let base_url = base_url(ctx.base_url_override(), &ctx.effective_base_url());
        let date = Arc::new(std::sync::Mutex::new(None));
        let loud: Arc<dyn RequestExecutor> =
            Arc::new(Adapter(Arc::new(retry_disabled(ctx)), date.clone()));
        let quiet: Arc<dyn RequestExecutor> = Arc::new(Adapter(
            Arc::new(retry_disabled(ctx).with_debug(false, vec![])),
            date.clone(),
        ));
        Self {
            client: client(loud, &base_url),
            quiet: client(quiet, &base_url),
            debug,
            date,
        }
    }

    /// The `Date` of the last answer. Calls in one invocation run one at a
    /// time, so read right after a call it belongs to.
    pub(crate) fn last_date(&self) -> Option<std::time::SystemTime> {
        self.date.lock().ok().and_then(|d| *d)
    }

    /// `GET /sandboxes/{sandbox}/logs?replica=N`, the stored copy of that
    /// replica's output.
    pub(crate) async fn logs(
        &self,
        sandbox_id: &str,
        replica: i64,
        since: Option<&str>,
    ) -> Result<StoredLogs, CallError> {
        let request = SavedLogsQueryRequest {
            replica: Some(replica),
            since: since.map(str::to_string),
        };
        let logs = attempt(
            STORED_READ,
            self.client.saved_logs(sandbox_id, &request, None),
        )
        .await?;
        Ok(StoredLogs {
            body: logs.content.into_bytes(),
            truncated: logs.truncated,
        })
    }
}

/// A plain-reqwest executor, so the loops can be tested against a mock
/// without an `AppContext`.
#[cfg(test)]
struct Plain(reqwest::Client);

#[cfg(test)]
impl RequestExecutor for Plain {
    fn execute(
        &self,
        request: reqwest::Request,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            self.0
                .execute(request)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })
    }
}

#[cfg(test)]
impl Api {
    pub(crate) fn for_tests(base_url: &str) -> Self {
        let raw: Arc<dyn RequestExecutor> = Arc::new(Plain(reqwest::Client::new()));
        Self {
            client: client(raw.clone(), base_url),
            quiet: client(raw, base_url),
            debug: false,
            date: Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

pub(crate) struct StoredLogs {
    pub(crate) body: Vec<u8>,
    pub(crate) truncated: bool,
}

fn retry_disabled(ctx: &AppContext) -> CliExecutor {
    // The Arc is fresh, so it has one owner.
    Arc::try_unwrap(ctx.build_sdk_executor())
        .unwrap_or_else(|_| unreachable!("a fresh executor has one owner"))
        .with_retries(RetriesConfig::disabled())
}

fn client(executor: Arc<dyn RequestExecutor>, base_url: &str) -> SandboxClient {
    let config = ClientConfig {
        base_url: base_url.to_string(),
        max_retries: 0,
        ..Default::default()
    };
    let http_client = HttpClient::with_executor(executor, config);
    SandboxClient {
        secrets: SecretsClient {
            http_client: http_client.clone(),
        },
        http_client,
    }
}

/// The base URL the group's client is built with.
///
/// The executor puts a base-URL override's path in front of every request's
/// path, so a client based at the override itself would request
/// `/v3/v3/sandboxes`. With an override, the client is based at its origin
/// and the executor adds the path once; otherwise at the effective base URL.
pub(crate) fn base_url(override_url: Option<&str>, effective: &str) -> String {
    match override_url.and_then(|raw| reqwest::Url::parse(raw).ok()) {
        Some(url) => url.origin().ascii_serialization(),
        None => effective.trim_end_matches('/').to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_override_bases_the_client_at_its_origin() {
        assert_eq!(
            base_url(
                Some("https://api.staging.hedra.com/v3"),
                "https://api.staging.hedra.com/v3"
            ),
            "https://api.staging.hedra.com"
        );
        assert_eq!(
            base_url(Some("http://127.0.0.1:8080/v3/"), "ignored"),
            "http://127.0.0.1:8080"
        );
    }

    #[test]
    fn without_an_override_the_effective_base_is_used() {
        assert_eq!(
            base_url(None, "https://api.hedra.com/v3/"),
            "https://api.hedra.com/v3"
        );
    }
}
