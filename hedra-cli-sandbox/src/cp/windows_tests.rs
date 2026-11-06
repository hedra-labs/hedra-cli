//! `cp`'s Windows rules: drive letters, symlink entries, reserved names and
//! names that differ only in case, and the copy-in walk over `NtCreateFile`.

use std::path::{Path, PathBuf};
use std::process::Command;

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

/// A junction at `link` to the directory `target`; `mklink /J` needs no
/// privilege.
fn junction(link: &Path, target: &Path) {
    let ok = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()
        .unwrap()
        .status
        .success();
    assert!(ok, "mklink /J {}", link.display());
}

/// One entry of a tar stream: its name, type, mode, body and link target.
fn entries(b: &[u8]) -> Vec<(String, u8, u32, Vec<u8>, String)> {
    let mut ar = tar::Archive::new(b);
    ar.entries()
        .unwrap()
        .map(|ent| {
            let mut ent = ent.unwrap();
            let h = ent.header();
            let kind = h.entry_type().as_byte();
            let mode = h.mode().unwrap() & 0o7777;
            let link = ent
                .link_name_bytes()
                .map(|l| String::from_utf8_lossy(&l).into_owned())
                .unwrap_or_default();
            let name = String::from_utf8_lossy(&ent.path_bytes()).into_owned();
            let mut body = Vec::new();
            std::io::Read::read_to_end(&mut ent, &mut body).unwrap();
            (name, kind, mode, body, link)
        })
        .collect()
}

/// A sandbox taking a copy in of `local` at `/work`: it answers `dir`,
/// reads stdin to its end, and exits 0.
async fn copy_in_to_work(local: &Path) -> (CopyReport<FakeErr>, Vec<u8>) {
    let plan = plan_copy_in(&spec("/work", local, Direction::In)).unwrap();
    let (sink, source, mut fake) = call();
    let sandbox = tokio::spawn(async move {
        fake.argv().await;
        fake.data(Stream::Stdout, b"dir\n").await;
        let (stdin, _) = fake.stdin().await;
        fake.exit(0).await;
        stdin
    });
    let r = within(copy_in(plan, sink, source, Arc::new(Progress::default()))).await;
    (r, sandbox.await.unwrap())
}

/// A tree with a file, an empty directory, a nested file, a junction to a
/// directory outside it, and, where symlinks can be made, a symlink.
fn fixture() -> (tempfile::TempDir, PathBuf, bool) {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("tree");
    std::fs::create_dir_all(root.join("empty")).unwrap();
    std::fs::create_dir_all(root.join("sub")).unwrap();
    std::fs::write(root.join("a.txt"), b"hello").unwrap();
    std::fs::write(root.join("sub").join("b.txt"), b"nested").unwrap();
    let outside = t.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("secret.txt"), b"credentials").unwrap();
    junction(&root.join("j"), &outside);
    // Creating a symlink takes a privilege or developer mode.
    let linked = std::os::windows::fs::symlink_file("a.txt", root.join("s")).is_ok();
    (t, root, linked)
}

/// The walk sends files 0644 and directories 0755, sends a junction or a
/// symlink beneath the top as a symlink entry, and never reads through one.
#[tokio::test(flavor = "multi_thread")]
async fn copy_in_walks_the_tree_and_sends_links_as_links() {
    let (_t, root, linked) = fixture();
    let (r, stdin) = copy_in_to_work(&root).await;
    assert!(matches!(r.result, Ok(0)), "{:?}", r.result);
    let got = entries(&stdin);
    let find = |name: &str| {
        got.iter()
            .find(|e| e.0.trim_end_matches('/') == name)
            .unwrap_or_else(|| panic!("no {name} in {got:?}"))
    };
    assert_eq!(find("tree").1, b'5');
    assert_eq!(find("tree").2, 0o755);
    assert_eq!(find("tree/empty").1, b'5');
    assert_eq!(find("tree/empty").2, 0o755);
    let a = find("tree/a.txt");
    assert_eq!((a.1, a.2, a.3.as_slice()), (b'0', 0o644, &b"hello"[..]));
    let b = find("tree/sub/b.txt");
    assert_eq!((b.1, b.2, b.3.as_slice()), (b'0', 0o644, &b"nested"[..]));
    let j = find("tree/j");
    assert_eq!(j.1, b'2', "{j:?}");
    assert!(j.4.ends_with("outside"), "{j:?}");
    if linked {
        let s = find("tree/s");
        assert_eq!((s.1, s.4.as_str()), (b'2', "a.txt"), "{s:?}");
    }
    assert!(got.iter().all(|e| e.3 != b"credentials"), "{got:?}");
    assert!(got.iter().all(|e| !e.0.contains("secret")), "{got:?}");
}

/// A file swapped for a junction between the listing and the open fails the
/// copy rather than reading through the junction.
#[test]
fn a_file_swapped_for_a_junction_fails_the_walk() {
    let t = tempfile::tempdir().unwrap();
    let outside = t.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("secret.txt"), b"credentials").unwrap();
    // The listing saw a file; a junction is there now.
    junction(&t.path().join("data"), &outside);
    let mut out = Vec::new();
    let mut b = tar::Builder::new(&mut out);
    let mut walked = send::Walked::default();
    let e = send::test_entry(&mut b, t.path(), "data", false, &mut walked).unwrap_err();
    assert!(e.contains("changed while it was being copied"), "{e}");
    // So did a directory the listing saw.
    let e = send::test_entry(&mut b, t.path(), "data", true, &mut walked).unwrap_err();
    assert!(e.contains("changed while it was being copied"), "{e}");
}

/// A top swapped for a junction between the plan and the walk is not
/// followed.
#[tokio::test(flavor = "multi_thread")]
async fn a_top_swapped_for_a_junction_after_the_plan_fails_the_copy() {
    let (t, root, _) = fixture();
    let plan = plan_copy_in(&spec("/work", &root, Direction::In)).unwrap();
    std::fs::rename(&root, t.path().join("moved")).unwrap();
    junction(&root, &t.path().join("moved"));
    let (sink, source, mut fake) = call();
    let sandbox = tokio::spawn(async move {
        fake.argv().await;
        fake.data(Stream::Stdout, b"dir\n").await;
        let (stdin, eof) = fake.stdin().await;
        (stdin, eof)
    });
    let r = within(copy_in(plan, sink, source, Arc::new(Progress::default()))).await;
    assert!(
        rejected(&r).contains("changed while it was being copied"),
        "{:?}",
        r.result
    );
    let (stdin, eof) = sandbox.await.unwrap();
    assert!(!eof && stdin.is_empty());
}
