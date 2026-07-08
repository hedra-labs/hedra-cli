//! Service clients and API endpoints
//!
//! This module contains client implementations for:
//!
//! - **Queue**
//! - **Requests**
//! - **Models**
//! - **Keys**
//! - **Tokens**
//! - **Files**
//! - **Webhooks**

use crate::{ApiError, ClientConfig};

pub mod files;
pub mod keys;
pub mod models;
pub mod queue;
pub mod requests;
pub mod tokens;
pub mod webhooks;
pub struct ApiClient {
    pub config: ClientConfig,
    pub queue: QueueClient,
    pub requests: RequestsClient,
    pub models: ModelsClient,
    pub keys: KeysClient,
    pub tokens: TokensClient,
    pub files: FilesClient,
    pub webhooks: WebhooksClient,
}

impl ApiClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            config: config.clone(),
            queue: QueueClient::new(config.clone())?,
            requests: RequestsClient::new(config.clone())?,
            models: ModelsClient::new(config.clone())?,
            keys: KeysClient::new(config.clone())?,
            tokens: TokensClient::new(config.clone())?,
            files: FilesClient::new(config.clone())?,
            webhooks: WebhooksClient::new(config.clone())?,
        })
    }
}

pub use files::FilesClient;
pub use keys::KeysClient;
pub use models::ModelsClient;
pub use queue::QueueClient;
pub use requests::RequestsClient;
pub use tokens::TokensClient;
pub use webhooks::WebhooksClient;
