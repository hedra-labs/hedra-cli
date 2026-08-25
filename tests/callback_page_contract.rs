//! Pins the wording the loopback listener puts in `error_description`, because
//! a page in a different repository reads it.
//!
//! hedra-labs/website#7057 serves `https://www.hedra.com/cli/auth/error`, which
//! `cli/hedra-cli/auth.rs` wires in as the CLI's failure callback. Two of that
//! page's states arrive indistinguishable on the wire: a callback missing its
//! `code` and a callback whose `state` did not match are **both**
//! `error=invalid_request`, and the only thing separating them is the
//! description text. The page therefore matches that text verbatim, and its
//! `callback-copy.test.ts` names these strings.
//!
//! What makes that fragile is where the strings live. `src/auth/oauth_login.rs`
//! is **generated**, and deliberately not in `.fernignore`. A regeneration that
//! rewords them breaks the cross-repo match *silently*: the page degrades to
//! its generic headline while still showing the right code and the real
//! description, so neither side looks broken and one state has quietly stopped
//! being distinguishable.
//!
//! A source-text pin rather than a behavioural one because there is no way to
//! drive a bad callback from here: `accept_callback`, `write_failure`, and
//! `build_error_redirect` are all private to the SDK crate, and the only
//! public entry point is a full browser login. `include_str!` reads the file
//! at compile time, so this costs no runtime IO and cannot go stale against a
//! moved checkout.
//!
//! **If this test fails, do not "fix" it by pasting the new wording in.**
//! Either restore the old string, or change it in lockstep with
//! `apps/www/src/app/(site)/cli/auth/callback-copy.ts` and its test in
//! hedra-labs/website — and expect the two to be deployed together.
//!
//! Hand-written and `.fernignore`-protected via the `tests/` entry: the
//! generator never emits this tree, which is what makes that entry
//! load-bearing rather than redundant.
//!
//! No new dev-dependency, for the reason `auth_commands.rs` gives: `Cargo.toml`
//! is generator-owned, so anything added for a test is reverted by the next
//! regeneration. This file needs nothing beyond `include_str!`.

/// The generated listener, read at compile time.
const OAUTH_LOGIN_RS: &str = include_str!("../src/auth/oauth_login.rs");

/// Selects the website's "We stopped this sign-in." copy. This is the match
/// that matters most: it is the one the page uses to say a callback may have
/// been a CSRF attempt, so a false positive is worse than a miss. A miss
/// degrades to the generic headline; a false positive tells a user with a
/// merely malformed callback that they were attacked.
#[test]
fn the_state_mismatch_description_is_unchanged() {
    let pinned = "The callback `state` parameter did not match the one the CLI sent.";
    assert!(
        OAUTH_LOGIN_RS.contains(pinned),
        "the `state`-mismatch description changed; hedra-labs/website matches it verbatim \
         to tell this case apart from a plain malformed callback — both are `invalid_request`"
    );
}

/// Selects the website's "That didn't complete." copy for a truncated
/// callback. Pinned in two halves because the parameter name is interpolated:
/// the listener formats `code` or `state` into the middle of the sentence.
#[test]
fn the_missing_parameter_description_is_unchanged() {
    let (head, tail) = ("The callback was missing its `", "` parameter.");
    assert!(
        OAUTH_LOGIN_RS.contains(&format!("{head}{{missing}}{tail}")),
        "the missing-parameter description changed; hedra-labs/website matches \
         `{head}…{tail}` verbatim"
    );
}

/// The one description the CLI sends that is a fixed constant rather than a
/// pass-through. It is deliberately vague — the real reason stays in the
/// terminal, because a token-endpoint body has no business in a URL — so the
/// page's whole job here is to point back at the terminal. Reword it and the
/// page stops agreeing with what the user is being told to do.
#[test]
fn the_post_callback_failure_description_is_unchanged() {
    let pinned = "The CLI could not complete the login. Check your terminal for details.";
    assert!(
        OAUTH_LOGIN_RS.contains(pinned),
        "the post-callback failure description changed; it must keep sending the reader \
         back to the terminal, which is the only place the real reason is printed"
    );
}

/// The codes themselves. The descriptions above only ever get consulted
/// because these two collide, and `server_error` is what separates "the
/// browser leg failed" from "everything after it did".
#[test]
fn the_synthesized_error_codes_are_unchanged() {
    for (code, why) in [
        (
            "\"invalid_request\"",
            "both synthesized callback failures report it; the website keys its two \
             `invalid_request` states off the description",
        ),
        (
            "\"server_error\"",
            "reported when the callback was fine but the token exchange or keyring \
             write failed after it",
        ),
    ] {
        assert!(
            OAUTH_LOGIN_RS.contains(code),
            "{code} no longer synthesized: {why}"
        );
    }
}

/// The guarantee the pages are written against: a hosted error page always has
/// something to render, because the listener invents a code for callbacks that
/// arrive without one. If a regeneration ever let a bare failure through, the
/// page would render an empty state none of its six branches cover.
#[test]
fn every_failure_path_still_carries_a_code() {
    let calls = OAUTH_LOGIN_RS.matches("write_failure(").count();
    assert!(
        calls >= 4,
        "expected every failure to funnel through `write_failure` \
         (3 call sites + the definition); found {calls}"
    );
}

/// The success redirect is written into `Location` verbatim — no code, no
/// email, no workspace — which is why the success page is static by contract
/// and why no credential can leak through a URL on the happy path. If a
/// regeneration ever started appending parameters there, the page would need
/// to be reviewed before it shipped, not after.
#[test]
fn the_success_redirect_still_carries_no_parameters() {
    assert!(
        OAUTH_LOGIN_RS.contains("write_redirect(&mut self.socket, url)"),
        "the success redirect no longer passes the configured URL through untouched; \
         anything appended here lands in browser history, the Referer header, and \
         every proxy log between the user and www.hedra.com"
    );
}
