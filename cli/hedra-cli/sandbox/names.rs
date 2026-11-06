//! Sandbox names and ids, and resolving a name to one sandbox.

use hedra_cli_sdk::api::Sandbox;

use super::api::{attempt, Api, ATTEMPT};
use super::error::Failure;

const ID_PREFIX: &str = "sandbox_";
/// The longest name the create route takes.
const MAX_NAME_LENGTH: usize = 32;
/// The static segments under `/sandboxes`, which the create route refuses as
/// names: `GET /sandboxes/gpus` is the GPU list, not a sandbox named `gpus`.
const RESERVED_NAMES: [&str; 4] = ["gpus", "quota", "secrets", "signing-key"];

/// What the caller named: an id, used as given, or a name to resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Target {
    Id(String),
    Name(String),
}

impl Target {
    /// Refuses an argument that is neither a `sandbox_` id nor a valid
    /// sandbox name, before any request.
    pub(crate) fn parse(arg: &str) -> Result<Self, Failure> {
        if is_sandbox_id(arg) {
            return Ok(Target::Id(arg.to_string()));
        }
        if RESERVED_NAMES.contains(&arg) {
            return Err(Failure::rejected(format!(
                "`{arg}` is not a sandbox name: no sandbox can be named `{arg}`."
            )));
        }
        if is_sandbox_name(arg) {
            return Ok(Target::Name(arg.to_string()));
        }
        Err(bad_name(arg))
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            Target::Id(s) | Target::Name(s) => s,
        }
    }
}

/// `sandbox_<uuid>`: 32 hex digits, hyphenated or not. Checked because the
/// id goes into the request path as given, so `/`, `?` or `..` in it would
/// reach another route.
pub(crate) fn is_sandbox_id(arg: &str) -> bool {
    let Some(tail) = arg.strip_prefix(ID_PREFIX) else {
        return false;
    };
    tail.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-')
        && tail.bytes().filter(u8::is_ascii_hexdigit).count() == 32
        && matches!(tail.len(), 32 | 36)
}

pub(crate) fn bad_name(arg: &str) -> Failure {
    Failure::rejected(format!(
        "`{arg}` is not a sandbox name or id: a name is 1 to {MAX_NAME_LENGTH} lowercase \
         letters, digits and dashes, starting and ending with a letter or digit."
    ))
}

/// The create route's name pattern:
/// `^[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?$`.
pub(crate) fn is_sandbox_name(s: &str) -> bool {
    let bytes = s.as_bytes();
    let edge = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    !bytes.is_empty()
        && bytes.len() <= MAX_NAME_LENGTH
        && edge(bytes[0])
        && edge(bytes[bytes.len() - 1])
        && bytes.iter().all(|&b| edge(b) || b == b'-')
}

/// Resolves `target` to one sandbox's id. A verb resolves a name once and
/// keeps the id, so a re-mint or reconnect cannot reach a later sandbox given
/// the same name.
pub(crate) async fn resolve(api: &Api, target: &Target) -> Result<String, Failure> {
    match target {
        Target::Id(id) => Ok(id.clone()),
        Target::Name(name) => Ok(resolve_name(api, name).await?.id),
    }
}

/// `GET /sandboxes/{name}`, which answers the caller's live sandbox of that
/// name, else the newest, else Not found.
pub(crate) async fn resolve_name(api: &Api, name: &str) -> Result<Sandbox, Failure> {
    attempt(ATTEMPT, api.client.get(name, None))
        .await
        .map_err(|e| e.failure(Some(name)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reserved names are exactly the spec's static segments under
    /// `/sandboxes`, so a regeneration that adds a route adds its name here.
    #[test]
    fn the_reserved_names_are_the_specs_static_segments() {
        let spec: serde_json::Value =
            serde_json::from_str(include_str!("../openapi0.json")).unwrap();
        let mut segments: Vec<&str> = spec["paths"]
            .as_object()
            .expect("the spec's paths")
            .keys()
            .filter_map(|p| p.strip_prefix("/sandboxes/"))
            .map(|rest| rest.split('/').next().unwrap_or_default())
            .filter(|s| !s.starts_with('{'))
            .collect();
        segments.sort_unstable();
        segments.dedup();
        assert_eq!(segments, RESERVED_NAMES);
    }

    #[test]
    fn names_follow_the_create_routes_pattern() {
        for ok in ["a", "train", "my-run-2", "0", &"a".repeat(32)] {
            assert!(is_sandbox_name(ok), "{ok}");
        }
        for bad in [
            "",
            "-a",
            "a-",
            "Train",
            "a_b",
            "a.b",
            &"a".repeat(33),
            "a b",
        ] {
            assert!(!is_sandbox_name(bad), "{bad}");
        }
    }

    #[test]
    fn an_id_is_used_as_given_and_a_bad_argument_is_refused() {
        assert_eq!(
            Target::parse("sandbox_0193a1b2-c3d4-4e5f-8a9b-0c1d2e3f4a5b").unwrap(),
            Target::Id("sandbox_0193a1b2-c3d4-4e5f-8a9b-0c1d2e3f4a5b".into())
        );
        assert_eq!(
            Target::parse("train").unwrap(),
            Target::Name("train".into())
        );
        let refused = Target::parse("Bad_Name").unwrap_err();
        assert_eq!(refused.class, super::super::error::Class::Rejected);
    }

    #[test]
    fn an_id_must_be_a_uuid_so_it_cannot_reach_another_route() {
        for ok in [
            "sandbox_0193a1b2-c3d4-4e5f-8a9b-0c1d2e3f4a5b",
            "sandbox_0193a1b2c3d44e5f8a9b0c1d2e3f4a5b",
        ] {
            assert!(is_sandbox_id(ok), "{ok}");
        }
        for bad in [
            "sandbox_",
            "sandbox_x",
            "sandbox_x/../../keys/K/revoke?",
            "sandbox_0193a1b2-c3d4-4e5f-8a9b-0c1d2e3f4a5b/../x",
            "sandbox_0193a1b2-c3d4-4e5f-8a9b-0c1d2e3f4a5?",
            "sandbox_0193a1b2-c3d4-4e5f-8a9b-0c1d2e3f4a5b-",
        ] {
            assert!(!is_sandbox_id(bad), "{bad}");
            assert!(Target::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_reserved_segment_is_refused_as_a_name() {
        for reserved in ["gpus", "quota", "secrets", "signing-key"] {
            let refused = Target::parse(reserved).unwrap_err();
            assert_eq!(
                refused.class,
                super::super::error::Class::Rejected,
                "{reserved}"
            );
        }
    }
}
