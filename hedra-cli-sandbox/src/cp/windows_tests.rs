//! `cp`'s Windows rules: drive letters, symlink entries, reserved names and
//! names that differ only in case.

use std::path::Path;

use super::test_support::*;
use super::*;

#[test]
fn a_drive_letter_is_a_local_path() {
    let valid = |n: &str| !n.is_empty() && !n.contains('/') && !n.contains('\\');
    let s = parse_args("c:\\data", "train:/work", valid).unwrap();
    assert_eq!((s.direction, s.sandbox.as_str()), (Direction::In, "train"));
    assert_eq!(s.local, Path::new("c:\\data"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_symlink_entry_is_left_out_and_fails_the_copy() {
    let a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/a.txt", b'0', 0o644, b"hello", ""),
        e("tree/link", b'2', 0o777, b"", "a.txt"),
    ]);
    let base = tempfile::tempdir().unwrap();
    let r = copy_out_of("/work/tree", base.path(), a.clone(), gnu_totals(a.len()), 0).await;
    let m = rejected(&r);
    assert!(m.contains("left out"), "{m}");
    assert!(
        r.warnings
            .iter()
            .any(|w| w.contains("symlinks are not created on Windows")),
        "{:?}",
        r.warnings
    );
    assert_eq!(
        std::fs::read(base.path().join("tree").join("a.txt")).unwrap(),
        b"hello"
    );
    assert!(!base.path().join("tree").join("link").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn reserved_and_unwritable_names_are_left_out() {
    for bad in ["CON", "aux.py", "a:b", "x."] {
        let name = format!("tree/{bad}");
        let a = archive(&[
            e("tree/", b'5', 0o755, b"", ""),
            e(&name, b'0', 0o644, b"x", ""),
        ]);
        let base = tempfile::tempdir().unwrap();
        let r = copy_out_of("/work/tree", base.path(), a.clone(), gnu_totals(a.len()), 0).await;
        let m = rejected(&r);
        assert!(m.contains("left out"), "{bad}: {m}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn names_differing_only_in_case_are_reported() {
    let a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/Readme", b'0', 0o644, b"one", ""),
        e("tree/README", b'0', 0o644, b"two", ""),
    ]);
    let base = tempfile::tempdir().unwrap();
    let r = copy_out_of("/work/tree", base.path(), a.clone(), gnu_totals(a.len()), 0).await;
    let m = rejected(&r);
    assert!(m.contains("left out"), "{m}");
    assert!(
        r.warnings
            .iter()
            .any(|w| w.contains("does not tell its name from")),
        "{:?}",
        r.warnings
    );
}
