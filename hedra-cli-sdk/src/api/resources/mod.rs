//! Service clients and API endpoints
//!
//! This module contains client implementations for:
//!
//! - **Jobs**
//! - **Models**
//! - **Keys**
//! - **Tokens**
//! - **Files**
//! - **Billing**
//! - **Webhooks**
//! - **Log drains**
//! - **Chat**

use crate::{ApiError, ClientConfig};

pub mod billing;
pub mod chat;
pub mod files;
pub mod jobs;
pub mod keys;
pub mod log_drains;
pub mod models;
pub mod tokens;
pub mod webhooks;
pub struct ApiClient {
    pub config: ClientConfig,
    pub jobs: JobsClient,
    pub models: ModelsClient,
    pub keys: KeysClient,
    pub tokens: TokensClient,
    pub files: FilesClient,
    pub billing: BillingClient,
    pub webhooks: WebhooksClient,
    pub log_drains: LogDrainsClient,
    pub chat: ChatClient,
}

impl ApiClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            config: config.clone(),
            jobs: JobsClient::new(config.clone())?,
            models: ModelsClient::new(config.clone())?,
            keys: KeysClient::new(config.clone())?,
            tokens: TokensClient::new(config.clone())?,
            files: FilesClient::new(config.clone())?,
            billing: BillingClient::new(config.clone())?,
            webhooks: WebhooksClient::new(config.clone())?,
            log_drains: LogDrainsClient::new(config.clone())?,
            chat: ChatClient::new(config.clone())?,
        })
    }
}

pub use billing::BillingClient;
pub use chat::ChatClient;
pub use files::FilesClient;
pub use jobs::JobsClient;
pub use keys::KeysClient;
pub use log_drains::LogDrainsClient;
pub use models::ModelsClient;
pub use tokens::TokensClient;
pub use webhooks::WebhooksClient;
