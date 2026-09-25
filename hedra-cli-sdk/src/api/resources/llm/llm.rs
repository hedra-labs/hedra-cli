use crate::{ApiError, ClientConfig, HttpClient};

pub struct LlmClient {
    pub http_client: HttpClient,
}

impl LlmClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }
}
