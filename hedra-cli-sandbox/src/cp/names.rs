//! Path and name rules shared by both directions of a copy.

/// `s` with every control character escaped, so a name from the sandbox
/// cannot carry a terminal escape sequence.
pub fn escape_controls(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_control() {
            if (c as u32) < 0x100 {
                out.push_str(&format!("\\x{:02x}", c as u32));
            } else {
                out.push_str(&format!("\\u{{{:x}}}", c as u32));
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The most characters of a name a message shows.
const SHOWN_MAX: usize = 200;

/// A name from the sandbox as a message shows it: escaped, and cut short.
pub(crate) fn show(s: &str) -> String {
    let escaped = escape_controls(s);
    match escaped.char_indices().nth(SHOWN_MAX) {
        Some((at, _)) => format!("{}…", &escaped[..at]),
        None => escaped,
    }
}

/// A port of Go's `path.Clean` for slash-separated sandbox paths.
pub(crate) fn clean(p: &str) -> String {
    if p.is_empty() {
        return ".".into();
    }
    let rooted = p.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for part in p.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.last().is_some_and(|l| *l != "..") {
                    parts.pop();
                } else if !rooted {
                    parts.push("..");
                }
            }
            _ => parts.push(part),
        }
    }
    let joined = parts.join("/");
    match (rooted, joined.is_empty()) {
        (true, _) => format!("/{joined}"),
        (false, true) => ".".into(),
        (false, false) => joined,
    }
}

/// Go's `path.Base`: the last element, with trailing slashes removed.
pub(crate) fn base(p: &str) -> String {
    if p.is_empty() {
        return ".".into();
    }
    let trimmed = p.trim_end_matches('/');
    if trimmed.is_empty() {
        return "/".into();
    }
    trimmed.rsplit('/').next().unwrap_or(trimmed).to_string()
}

/// Go's `path.Dir`: everything but the last element, cleaned.
pub(crate) fn dir(p: &str) -> String {
    let cut = p.rfind('/').map_or(0, |i| i + 1);
    clean(&p[..cut])
}

/// Go's `path.Join` of two elements.
pub(crate) fn join(a: &str, b: &str) -> String {
    clean(&format!("{a}/{b}"))
}

/// Maps a tar entry rooted at `base` to its name in the receiving directory,
/// rooted at `name` instead. Refuses absolute names, names that climb out
/// with `..`, and names outside `base`.
pub(crate) fn local_name(entry: &str, base: &str, name: &str) -> Result<String, String> {
    let shown = show(entry);
    if entry.starts_with('/') {
        return Err(format!("refused {shown:?}: it is an absolute path"));
    }
    let clean = clean(entry);
    if clean.split('/').any(|part| part == "..") {
        return Err(format!("refused {shown:?}: it climbs out of the copy"));
    }
    if base == "." {
        return Ok(if clean == "." {
            name.to_string()
        } else {
            join(name, &clean)
        });
    }
    match clean.strip_prefix(base) {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => Ok(format!("{name}{rest}")),
        _ => Err(format!("refused {shown:?}: it is outside {}", show(base))),
    }
}

/// Why a path component cannot be a file name on Windows, if it cannot.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn windows_name_problem(component: &str) -> Option<&'static str> {
    if component.contains('\\') || component.contains(':') {
        return Some("it holds `\\` or `:`");
    }
    if component.ends_with('.') || component.ends_with(' ') {
        return Some("it ends in a dot or a space");
    }
    let stem = component.split('.').next().unwrap_or(component);
    let upper = stem.trim_end_matches(' ').to_ascii_uppercase();
    let device = matches!(
        upper.as_str(),
        "CON" | "NUL" | "AUX" | "PRN" | "CONIN$" | "CONOUT$"
    ) || ((upper.starts_with("COM") || upper.starts_with("LPT"))
        && upper.chars().count() == 4
        && upper
            .chars()
            .nth(3)
            .is_some_and(|d| ('1'..='9').contains(&d) || matches!(d, '¹' | '²' | '³')));
    device.then_some("it names a device")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_matches_go() {
        for (input, want) in [
            ("", "."),
            ("a/b/../c", "a/c"),
            ("/../x", "/x"),
            ("../x", "../x"),
            ("a/./b/", "a/b"),
            ("/", "/"),
            ("./", "."),
        ] {
            assert_eq!(clean(input), want, "clean({input:?})");
        }
        assert_eq!(base("/work/tree/."), ".");
        assert_eq!(base("/work/tree/"), "tree");
        assert_eq!(dir("/work"), "/");
        assert_eq!(dir("data/copy"), "data");
        assert_eq!(dir("x"), ".");
    }

    #[test]
    fn local_name_keeps_entries_under_the_top() {
        assert_eq!(local_name("tree/a", "tree", "out").unwrap(), "out/a");
        assert_eq!(local_name("tree", "tree", "out").unwrap(), "out");
        assert_eq!(local_name("./a", ".", ".").unwrap(), "a");
        assert_eq!(local_name("./", ".", "x").unwrap(), "x");
        assert!(local_name("treex/a", "tree", "out").is_err());
        assert!(local_name(".bashrc", "results.csv", "results.csv").is_err());
        assert!(local_name("../evil", ".", ".").is_err());
        assert!(local_name("tree/../../evil", "tree", "out").is_err());
        assert!(local_name("/etc/passwd", ".", ".").is_err());
    }

    #[test]
    fn escape_controls_escapes_osc() {
        assert_eq!(escape_controls("a\x1b]52;b\x07"), "a\\x1b]52;b\\x07");
    }

    #[test]
    fn windows_names() {
        for bad in [
            "CON", "aux.py", "a:b", "x.", "x ", "COM1", "lpt9.txt", "COM¹", "conin$",
        ] {
            assert!(windows_name_problem(bad).is_some(), "{bad}");
        }
        for good in ["CONSOLE", "com10", "a.b", "auxiliary.py"] {
            assert!(windows_name_problem(good).is_none(), "{good}");
        }
    }
}
