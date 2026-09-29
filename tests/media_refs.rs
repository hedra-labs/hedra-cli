//! What a server receives for a media-reference flag (ENG-11602), driving the
//! **real compiled binary** against a mock server.
//!
//! The shorthand is a parameter rewrite installed from `cli/hedra-cli/custom.rs`
//! through a Replay-patched runtime seam, so a unit test of the rewrite alone
//! would pass even if the seam were lost in a regeneration. These tests read
//! the request body the server actually got, which only a working seam and a
//! working rewrite can produce together.
//!
//! Hand-written and .fernignore-protected (the `tests/` entry).

use std::process::{Command, Output, Stdio};

use serde_json::{json, Value};
use tempfile::TempDir;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

const ASSET: &str = "asset_341d2ba0-c784-4756-9a67-914127fd9b23";

/// What one invocation did: the process result, and the JSON body of every
/// request that reached the server.
struct Outcome {
    out: Output,
    bodies: Vec<Value>,
}

impl Outcome {
    /// The single request body the server received.
    fn body(&self) -> &Value {
        assert_eq!(
            self.bodies.len(),
            1,
            "expected exactly one request; the CLI exited with {:?}\n{}",
            self.out.status.code(),
            self.output_text(),
        );
        &self.bodies[0]
    }

    fn output_text(&self) -> String {
        format!(
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&self.out.stdout),
            String::from_utf8_lossy(&self.out.stderr)
        )
    }
}

/// Run `hedra-cli <args> --base-url <mock>` in an isolated HOME with a key in
/// the environment, answering every POST with `status`.
fn run(args: &[&str], status: u16, response: Value) -> Outcome {
    let home = TempDir::new().expect("temp HOME");
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(status).set_body_json(response))
            .mount(&server)
            .await;

        let uri = server.uri();
        let out = Command::new(env!("CARGO_BIN_EXE_hedra-cli"))
            .args(args)
            .args(["--base-url", &uri, "--format", "json"])
            .env("HOME", home.path())
            .env("FERN_CLI_CREDENTIAL_STORE", "file")
            .env("HEDRA_API_KEY", "test-key")
            .env("NO_COLOR", "1")
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("APPDATA")
            .env_remove("HEDRA_ENV")
            .env_remove("HEDRA_CLI_BASE_URL")
            .stdin(Stdio::null())
            .output()
            .expect("run hedra-cli");

        let bodies = server
            .received_requests()
            .await
            .expect("mock server recorded requests")
            .iter()
            .map(|r| serde_json::from_slice(&r.body).expect("request body is JSON"))
            .collect();
        Outcome { out, bodies }
    })
}

fn submit(args: &[&str]) -> Outcome {
    run(args, 202, json!({ "id": "job_1", "status": "queued" }))
}

fn upscale(source_image: &str) -> Outcome {
    submit(&[
        "jobs",
        "submit-topaz-image-upscaler",
        "--input.source-image",
        source_image,
        "--input.target-resolution",
        "4K",
    ])
}

fn relight(images: &[&str]) -> Outcome {
    let mut args = vec!["jobs", "submit-eyeline-id-relight"];
    for image in images {
        args.extend(["--input.images", image]);
    }
    args.extend(["--input.prompt", "p", "--input.source-video", "asset_video"]);
    submit(&args)
}

/// Rejected locally: nothing on the wire, a failing exit, and an error that
/// names every accepted form.
fn assert_rejected(outcome: &Outcome, flag: &str) {
    assert!(
        outcome.bodies.is_empty(),
        "a request was sent: {:?}",
        outcome.bodies
    );
    assert!(!outcome.out.status.success(), "{}", outcome.output_text());
    let text = outcome.output_text();
    assert!(text.contains(&format!("{flag} expects")), "{text}");
    for form in ["asset id (asset_…)", "http(s):// URL", "JSON object"] {
        assert!(text.contains(form), "error does not name `{form}`:\n{text}");
    }
}

// ---------------------------------------------------------------------------
// Single fields
// ---------------------------------------------------------------------------

#[test]
fn a_bare_asset_id_is_sent_as_an_asset_ref() {
    // The ENG-11602 repro, verbatim.
    let outcome = upscale(ASSET);
    assert_eq!(
        outcome.body(),
        &json!({
            "input": {
                "source_image": { "source": "asset", "asset_id": ASSET },
                "target_resolution": "4K",
            }
        })
    );
}

#[test]
fn a_bare_url_is_sent_as_a_url_ref() {
    let outcome = upscale("https://cdn.example.com/cat.png");
    assert_eq!(
        outcome.body()["input"]["source_image"],
        json!({ "source": "url", "url": "https://cdn.example.com/cat.png" })
    );
}

#[test]
fn the_json_object_form_is_sent_unchanged() {
    let object = json!({ "source": "asset", "asset_id": ASSET });
    let outcome = upscale(&object.to_string());
    assert_eq!(outcome.body()["input"]["source_image"], object);
}

#[test]
fn any_other_string_is_rejected_before_sending() {
    for bad in ["foo", "asset_", "ftp://example.com/a.png", "{\"source\":"] {
        assert_rejected(&upscale(bad), "--input.source-image");
    }
}

// ---------------------------------------------------------------------------
// Array items
// ---------------------------------------------------------------------------

#[test]
fn repeated_array_items_are_each_expanded() {
    let url_ref = json!({ "source": "url", "url": "https://x.test/c.png" });
    let outcome = relight(&["asset_a", "https://x.test/b.png", &url_ref.to_string()]);
    let input = &outcome.body()["input"];
    assert_eq!(
        input["images"],
        json!([
            { "source": "asset", "asset_id": "asset_a" },
            { "source": "url", "url": "https://x.test/b.png" },
            url_ref,
        ])
    );
    // A single media field on the same operation is expanded alongside.
    assert_eq!(
        input["source_video"],
        json!({ "source": "asset", "asset_id": "asset_video" })
    );
}

#[test]
fn a_bad_array_item_is_rejected_before_sending() {
    assert_rejected(&relight(&["asset_a", "nope"]), "--input.images (item 2)");
}

// ---------------------------------------------------------------------------
// A plain object field is not a media reference
// ---------------------------------------------------------------------------

fn create_log_drain(headers: &str) -> Outcome {
    run(
        &[
            "log-drains",
            "create-log-drain",
            "--name",
            "n",
            "--url",
            "https://drain.example.com",
            "--headers",
            headers,
        ],
        201,
        json!({ "id": "drain_1" }),
    )
}

#[test]
fn a_non_media_object_flag_keeps_its_behaviour() {
    // `headers` is `type: object` with no MediaRef union: the shorthand must
    // not fire, so a bare id or URL is still the runtime's own rejection...
    for bare in [ASSET, "https://x.test/a"] {
        let outcome = create_log_drain(bare);
        assert!(
            outcome.bodies.is_empty(),
            "a request was sent: {:?}",
            outcome.bodies
        );
        let text = outcome.output_text();
        assert!(
            text.contains("Object-shorthand flag must be a JSON object, got string"),
            "{text}"
        );
    }
    // ...and the JSON object still goes through untouched.
    let outcome = create_log_drain(r#"{"X-Token":"abc"}"#);
    assert_eq!(outcome.body()["headers"], json!({ "X-Token": "abc" }));
}
