//! Exercise the shipped command parser and HTTP executor without real credentials.
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde_json::{json, Value};
use tempfile::TempDir;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const KEY: &str = "stable-user-request";

async fn submit(server: &MockServer, args: &[&str]) {
    let directory = TempDir::new().expect("temporary CLI directory");
    let mut command = Command::new(env!("CARGO_BIN_EXE_hedra-cli"));
    command
        .env_clear()
        .current_dir(directory.path())
        .env("HEDRA_API_KEY", "key_fixture:fake_secret")
        .env("FERN_CLI_CREDENTIAL_STORE", "file")
        .env("NO_COLOR", "1")
        .stdin(Stdio::null())
        .args(["--base-url", &server.uri(), "jobs"])
        .args(args)
        .args(["--retries", "1"]);
    let output = tokio::task::spawn_blocking(move || command.output())
        .await
        .expect("CLI task")
        .expect("CLI process");
    assert!(
        output.status.success(),
        "status {}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

async fn server(first_status: u16) -> MockServer {
    let server = MockServer::start().await;
    let attempts = Arc::new(AtomicUsize::new(0));
    Mock::given(method("POST"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).expect("JSON request");
            let header = request.headers.get("Idempotency-Key").expect("key header");
            if let Some(key) = body["idempotency_key"].as_str() {
                if header.to_str().expect("header string") != key {
                    return ResponseTemplate::new(400).set_body_json(json!({
                        "message": "Idempotency-Key header and body idempotency_key differ."
                    }));
                }
            }
            if attempts.fetch_add(1, Ordering::SeqCst) == 0 && first_status != 0 {
                return ResponseTemplate::new(first_status).insert_header("Retry-After", "0");
            }
            ResponseTemplate::new(202).set_body_json(json!({
                "job_id": "job_test", "model": "gpt-image-2", "status": "IN_QUEUE",
                "status_url": "/status", "result_url": "/result"
            }))
        })
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn caller_key_survives_typed_generic_and_json_submissions_and_retries() {
    let body = json!({"input": {"prompt": "test", "aspect_ratio": "16:9", "resolution": "1K"}, "idempotency_key": KEY}).to_string();
    let input = json!({"prompt": "test", "aspect_ratio": "16:9", "resolution": "1K"}).to_string();
    let commands = [
        vec![
            "submit-gpt-image-2",
            "--input.prompt",
            "test",
            "--input.aspect-ratio",
            "16:9",
            "--input.resolution",
            "1K",
            "--idempotency-key",
            KEY,
        ],
        vec![
            "submit",
            "--model",
            "gpt-image-2",
            "--input",
            &input,
            "--idempotency-key",
            KEY,
        ],
        vec!["submit-gpt-image-2", "--json", &body],
        vec!["submit", "--model", "gpt-image-2", "--json", &body],
    ];
    for args in commands {
        let server = server(503).await;
        submit(&server, &args).await;
        // A second invocation with the same caller key must send it unchanged too.
        submit(&server, &args).await;
        let requests = server.received_requests().await.expect("requests");
        assert_eq!(requests.len(), 3, "one retry plus one repeat: {args:?}");
        for request in requests {
            assert_eq!(request.headers["Idempotency-Key"], KEY);
            let body: Value = serde_json::from_slice(&request.body).expect("JSON body");
            assert_eq!(body["idempotency_key"], KEY);
        }
    }
}

#[tokio::test]
async fn automatic_key_is_stable_across_rate_limit_retry_and_fresh_per_invocation() {
    let server = server(429).await;
    let body = json!({"input": {"prompt": "test", "aspect_ratio": "16:9", "resolution": "1K"}, "idempotency_key": null}).to_string();
    let args = ["submit-gpt-image-2", "--json", &body];
    submit(&server, &args).await;
    submit(&server, &args).await;
    let requests = server.received_requests().await.expect("requests");
    assert_eq!(requests.len(), 3);
    assert_eq!(
        requests[0].headers["Idempotency-Key"],
        requests[1].headers["Idempotency-Key"]
    );
    let first = requests[0].headers["Idempotency-Key"]
        .to_str()
        .expect("key");
    let second = requests[2].headers["Idempotency-Key"]
        .to_str()
        .expect("key");
    assert!(!first.is_empty());
    assert!(!second.is_empty());
    assert_ne!(first, second);
}
