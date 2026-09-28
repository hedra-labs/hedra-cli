//! Select a single idempotency key before entering the HTTP retry loop.
use serde_json::Value;

use super::discovery::RestMethod;

pub(super) struct RequestIdempotency {
    pub header_to_add: Option<String>,
    pub retry_safe: bool,
}

pub(super) fn select(
    method: &RestMethod,
    headers: &[(String, String)],
    body: Option<&Value>,
) -> RequestIdempotency {
    // Auto Idempotency-Key: generate once before the retry loop so the
    // same key is sent on every attempt. Only for POST/PUT/PATCH, and
    // only when the caller didn't already supply one, unless opted out
    // via `x-fern-cli-idempotency: false`.
    //
    // The suppression condition deliberately does NOT include
    // `method.idempotent`. `x-fern-idempotent: true` only means the
    // operation *exposes* `--idempotency-key` — it is not a promise that
    // the user passed it. Treating the marker as "the caller provides a
    // key" inverted the safety property it exists for: the marker also
    // makes the operation retry-eligible (`method_allows_retry`), so a
    // marked POST that the user invoked without the flag retried with no
    // key at all, while the same POST *without* the marker got an
    // auto-generated key. A 5xx on a marked send could therefore deliver
    // twice. Only a key actually present on this invocation suppresses
    // generation.
    let header_supplied = headers
        .iter()
        .any(|(k, _)| k.eq_ignore_ascii_case("idempotency-key"));
    // Hedra's submission schema exposes this key in the JSON body. Mirror it
    // into the header instead of inventing a conflicting second key. Explicit
    // headers retain precedence; a caller-supplied mismatch is left for the API
    // to reject rather than silently rewriting either value.
    let body_key = body
        .and_then(|value| value.get("idempotency_key"))
        .and_then(Value::as_str);
    let user_supplied_key = header_supplied || body_key.is_some();
    let idempotency_key = if header_supplied {
        None
    } else if let Some(key) = body_key {
        Some(key.to_owned())
    } else if !method.no_auto_idempotency_key
        && crate::http::needs_idempotency_key(&method.http_method)
    {
        Some(crate::http::generate_idempotency_key())
    } else {
        None
    };

    // Retry-safety for POST/PATCH requires a key the *server* is known to
    // honor, which a key we invented does not establish. This used to read
    // `method.idempotent || idempotency_key.is_some()`, and since the auto
    // key is generated for every POST/PUT/PATCH, that made every
    // non-idempotent operation retry-eligible — a 5xx on a create retried
    // ~4x against an endpoint with no idempotency support at all and could
    // duplicate the resource.
    //
    // Two things do establish it:
    //   * `x-fern-idempotent: true` — the spec declares the operation
    //     supports an idempotency key, and the block above now guarantees
    //     one is actually sent;
    //   * an explicit `--idempotency-key` — the caller asserting the
    //     server dedupes on it. This also removes the surprise that
    //     supplying a key made the CLI *less* willing to retry.
    //
    // An auto-generated key is still sent (it is what makes a retry safe
    // on an endpoint that does consume it) but no longer licenses one.
    RequestIdempotency {
        header_to_add: idempotency_key,
        retry_safe: method.idempotent || user_supplied_key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn post() -> RestMethod {
        RestMethod {
            http_method: "POST".to_owned(),
            ..Default::default()
        }
    }

    #[test]
    fn explicit_header_is_not_duplicated_or_replaced_by_body_key() {
        let headers = vec![("iDeMpOtEnCy-KeY".to_owned(), "header-key".to_owned())];
        for body in [None, Some(json!({"idempotency_key": "body-key"}))] {
            let selected = select(&post(), &headers, body.as_ref());
            assert!(selected.header_to_add.is_none());
            assert!(selected.retry_safe);
        }
    }

    #[test]
    fn automatic_key_does_not_authorize_unsafe_retries() {
        for body in [
            None,
            Some(json!({})),
            Some(json!({"idempotency_key": null})),
        ] {
            let selected = select(&post(), &[], body.as_ref());
            assert!(selected.header_to_add.is_some());
            assert!(!selected.retry_safe);
        }
        let marked = RestMethod {
            idempotent: true,
            ..post()
        };
        let selected = select(&marked, &[], None);
        assert!(selected.header_to_add.is_some());
        assert!(selected.retry_safe);
    }

    #[test]
    fn opt_out_disables_automatic_keys_but_preserves_explicit_body_key() {
        let opted_out = RestMethod {
            no_auto_idempotency_key: true,
            ..post()
        };
        let selected = select(&opted_out, &[], None);
        assert!(selected.header_to_add.is_none());
        assert!(!selected.retry_safe);
        let selected = select(
            &opted_out,
            &[],
            Some(&json!({"idempotency_key": "body-key"})),
        );
        assert_eq!(selected.header_to_add.as_deref(), Some("body-key"));
        assert!(selected.retry_safe);
    }

    #[test]
    fn read_only_methods_do_not_get_automatic_keys() {
        let method = RestMethod {
            http_method: "GET".to_owned(),
            ..Default::default()
        };
        assert!(select(&method, &[], None).header_to_add.is_none());
    }
}
