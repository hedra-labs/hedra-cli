use crate::{ApiError, ClientConfig, HttpClient, RequestOptions};
use reqwest::Method;

pub struct RunAModelClient {
    pub http_client: HttpClient,
}

impl RunAModelClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }

    /// OpenAI-compatible Responses API. Each request runs as a chat completion, and is charged, limited and recorded as one. Each request must send the whole conversation in `input`: `store: true`, a `previous_response_id`, a `conversation` or an `item_reference` input item gets a 400 that says so. A request without `max_output_tokens` is bounded at 32,768 output tokens, or the model's lower limit. Function and namespace tools are served, and `web_search` tools are ignored. Each image the model does not accept becomes a text note in its place: every image on a model without image input, and on any other model the oldest images beyond the model's `max_images_per_request`. `text.format` becomes chat's `response_format`. Beside tools under `tool_choice` `auto`, the route instead offers a `json_schema` format's schema to the model as one more function, `final_answer`, and sets `tool_choice` to `required`, so the model can call tools before it answers; the route returns a call to `final_answer` as the answer message, with `phase` `final_answer`, and leaves out any text the model writes, because openai-python's `responses.parse` parses every message as the answer. The route accepts `text.verbosity` and `service_tier` and ignores them. Errors are those of `POST /chat/completions`, and the 400 for a value that chat rejects names the chat field, such as `max_tokens` or `messages[3].content`. With `stream: true` the response is `text/event-stream` in the Responses event format, ending with `response.completed` or `response.incomplete`, or with `response.failed` for a failure after streaming has begun.
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
    ///     client.run_a_model.responses_create(None).await;
    /// }
    /// ```
    pub async fn responses_create(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<serde_json::Value, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.21.14".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::POST, "responses", None, None, options)
            .await
    }

    /// Anthropic-compatible Messages API. Set the base URL of an Anthropic SDK or of Claude Code to the v3 base URL followed by `/anthropic`, and send an API key in `x-api-key` or `Authorization: Bearer`. Each request runs as a chat completion, and is charged, limited and recorded as one. The route serves client tools, and answers a tool of any other type with a 400. Each image the model does not accept becomes a text note in its place: every image on a model without image input, and on any other model the oldest images beyond the model's `max_images_per_request`. `output_config.effort` becomes chat's `reasoning_effort`, and `thinking` of type `disabled` becomes `reasoning_effort: none`, whatever `output_config.effort` says. `output_config.format` becomes chat's `response_format`, so the answer's `text` block holds JSON that matches the schema. Beside tools, a request with `output_config.format` must set `tool_choice` to `any`, `tool` or `none`, and gets a 400 under `auto`. The route accepts `thinking.budget_tokens`, `cache_control` and `context_management`, and they have no effect. The model's reasoning comes back as `thinking` blocks with an empty `signature`. Errors are those of `POST /chat/completions` in Anthropic's error body: the body's `type` is `error`, `error.type` is Anthropic's error type for the status, and `error` keeps the v3 error fields. The 400 for a value that chat rejects names the chat field, such as `reasoning_effort` or `messages[3].content`, under the message or `tool_result` block the caller sent. With `stream: true` the response is `text/event-stream` in the Messages event format, ending with `message_stop`, or with an `error` event for a failure after streaming has begun.
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
    ///     client.run_a_model.messages_create(None).await;
    /// }
    /// ```
    pub async fn messages_create(
        &self,
        options: Option<RequestOptions>,
    ) -> Result<serde_json::Value, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.21.14".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(Method::POST, "anthropic/v1/messages", None, None, options)
            .await
    }
}
