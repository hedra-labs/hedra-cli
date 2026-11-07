//! Text from a server or a sandbox, made safe to print.

/// `s` with every control character escaped as Rust's `char::escape_default`
/// writes it (`\u{1b}`, `\n`), so text from a server or a sandbox cannot
/// carry a terminal escape sequence into a line hedra-cli prints. The CLI
/// escapes with this too, so every line has the one format.
pub fn escape_controls(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_control() {
            out.extend(c.escape_default());
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_controls_escapes_osc() {
        assert_eq!(escape_controls("a\x1b]52;b\x07 é"), "a\\u{1b}]52;b\\u{7} é");
    }
}
