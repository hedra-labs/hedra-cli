//! `cp` over a fake exec that plays the sandbox's half.

use std::fs;
use std::io::Read;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::mpsc;

use super::test_support::*;
use super::*;

fn mkfifo(p: &Path) {
    let ok = std::process::Command::new("mkfifo")
        .arg(p)
        .status()
        .unwrap()
        .success();
    assert!(ok, "mkfifo {}", p.display());
}

fn umask() -> u32 {
    extract::umask()
}

fn tree_archive() -> Vec<u8> {
    archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/empty/", b'5', 0o700, b"", ""),
        e("tree/nested/", b'5', 0o777, b"", ""),
        e("tree/nested/a.txt", b'0', 0o644, b"hello", ""),
        e("tree/run.sh", b'0', 0o4755, b"#!/bin/sh\n", ""),
        e("tree/link", b'2', 0o777, b"", "nested/a.txt"),
    ])
}

fn mode(p: &Path) -> u32 {
    fs::symlink_metadata(p).unwrap().permissions().mode() & 0o7777
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_out_writes_the_tree_where_scp_would() {
    let a = tree_archive();
    for (dst, root) in [(".", "tree"), ("checkpoints", "checkpoints")] {
        let base = tempfile::tempdir().unwrap();
        let r = copy_out_of(
            "/work/tree",
            &base.path().join(dst),
            a.clone(),
            gnu_totals(a.len()),
            0,
        )
        .await;
        assert!(matches!(r.result, Ok(0)), "{dst}: {:?}", r.result);
        let root = base.path().join(root);
        assert_eq!(fs::read(root.join("nested/a.txt")).unwrap(), b"hello");
        assert_eq!(mode(&root.join("empty")), 0o700 & !umask());
        assert_eq!(mode(&root.join("nested")), 0o777 & !umask());
        // 4755 loses its setuid bit and the umask's bits.
        assert_eq!(mode(&root.join("run.sh")), 0o755 & !umask());
        assert_eq!(
            fs::read_link(root.join("link")).unwrap(),
            Path::new("nested/a.txt")
        );
        let mtime = fs::metadata(root.join("nested"))
            .unwrap()
            .modified()
            .unwrap();
        assert_eq!(
            mtime,
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(MTIME)
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_out_creates_a_missing_parent() {
    let base = tempfile::tempdir().unwrap();
    let a = tree_archive();
    let dst = base.path().join("a/b/out");
    let r = copy_out_of("/work/tree", &dst, a.clone(), gnu_totals(a.len()), 0).await;
    assert!(matches!(r.result, Ok(0)), "{:?}", r.result);
    assert_eq!(fs::read(dst.join("nested/a.txt")).unwrap(), b"hello");
}

/// No entry is written outside the copy, however the archive names it and
/// whatever symlinks it, or an earlier copy, made.
/// A destination that cannot take the copy fails it before the call, and
/// the parents a failed copy created are removed.
#[tokio::test(flavor = "multi_thread")]
async fn copy_out_checks_the_destination_first() {
    let base = tempfile::tempdir().unwrap();
    let ro = base.path().join("ro");
    fs::create_dir(&ro).unwrap();
    fs::set_permissions(&ro, fs::Permissions::from_mode(0o555)).unwrap();
    for dst in [ro.join("out"), ro.join("a/b/out")] {
        let mut plan = plan_copy_out(&spec("/work/tree", &dst, Direction::Out)).unwrap();
        let e = plan.prepare().unwrap_err();
        assert!(e.contains("ro"), "{e}");
    }
    fs::set_permissions(&ro, fs::Permissions::from_mode(0o755)).unwrap();

    // Created for a copy that then failed: removed again.
    let dst = base.path().join("x/y/out");
    let r = copy_out_of("/work/tree", &dst, vec![], String::new(), 1).await;
    assert!(matches!(r.result, Ok(1)), "{:?}", r.result);
    assert!(!base.path().join("x").exists());
    // A directory that was already there stays.
    fs::create_dir(base.path().join("kept")).unwrap();
    let dst = base.path().join("kept/out");
    let _ = copy_out_of("/work/tree", &dst, vec![], String::new(), 1).await;
    assert!(base.path().join("kept").is_dir());
}

/// tar failing in the sandbox does not hide the entry hedra-cli refused.
#[tokio::test(flavor = "multi_thread")]
async fn copy_out_keeps_a_refusal_when_tar_fails_too() {
    let dst = tempfile::tempdir().unwrap();
    let a = archive(&[e("/etc/evil", b'0', 0o644, b"x", "")]);
    let r = copy_out_of("/work/tree", dst.path(), a, String::new(), 2).await;
    let msg = rejected(&r);
    assert!(
        msg.contains("absolute") && msg.contains("exited 2"),
        "{msg}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_out_refuses_an_entry_that_leaves_the_destination() {
    struct Case<'a> {
        name: &'a str,
        entries: Vec<E<'a>>,
        earlier: Option<&'a str>,
        base: &'a str,
    }
    let cases = vec![
        Case {
            name: "dot-dot",
            base: ".",
            earlier: None,
            entries: vec![
                e("./", b'5', 0o755, b"", ""),
                e("../evil", b'0', 0o644, b"x", ""),
            ],
        },
        Case {
            name: "absolute",
            base: ".",
            earlier: None,
            entries: vec![
                e("./", b'5', 0o755, b"", ""),
                e("/tmp/evil", b'0', 0o644, b"x", ""),
            ],
        },
        Case {
            name: "outside the source",
            base: "tree",
            earlier: None,
            entries: vec![
                e("other/", b'5', 0o755, b"", ""),
                e("other/evil", b'0', 0o644, b"x", ""),
            ],
        },
        Case {
            name: "a symlink as the top",
            base: "tree",
            earlier: None,
            entries: vec![e("tree", b'2', 0o777, b"", "../victim")],
        },
        Case {
            name: "beneath a symlink the archive makes",
            base: "tree",
            earlier: None,
            entries: vec![
                e("tree/", b'5', 0o755, b"", ""),
                e("tree/nested/", b'5', 0o755, b"", ""),
                e("tree/l", b'2', 0o777, b"", "nested"),
                e("tree/l/evil", b'0', 0o644, b"x", ""),
            ],
        },
        Case {
            name: "through a symlink out of the receiving directory",
            base: "tree",
            earlier: None,
            entries: vec![
                e("tree/", b'5', 0o755, b"", ""),
                e("tree/out", b'2', 0o777, b"", "../.."),
                e("tree/out/evil", b'0', 0o644, b"x", ""),
            ],
        },
        Case {
            name: "a chmod through a symlink",
            base: "tree",
            earlier: None,
            entries: vec![
                e("tree/", b'5', 0o755, b"", ""),
                e("tree/s", b'2', 0o777, b"", "../victim"),
                e("tree/s/", b'5', 0o777, b"", ""),
            ],
        },
        Case {
            name: "through a symlink an earlier copy left",
            base: "tree",
            earlier: Some(".."),
            entries: vec![
                e("tree/", b'5', 0o755, b"", ""),
                e("tree/l/evil", b'0', 0o644, b"x", ""),
            ],
        },
        Case {
            name: "a hard link out",
            base: "tree",
            earlier: None,
            entries: vec![
                e("tree/", b'5', 0o755, b"", ""),
                e("tree/evil", b'1', 0o644, b"", "../../etc/passwd"),
            ],
        },
    ];
    for c in cases {
        let outer = tempfile::tempdir().unwrap();
        let dst = outer.path().join("in/dst");
        let victim = outer.path().join("in/victim");
        fs::create_dir_all(&victim).unwrap();
        fs::set_permissions(&victim, fs::Permissions::from_mode(0o700)).unwrap();
        let mut target = dst.clone();
        if let Some(link) = c.earlier {
            fs::create_dir_all(dst.join("tree")).unwrap();
            symlink(link, dst.join("tree/l")).unwrap();
            target = dst.join("tree");
        }
        let a = archive(&c.entries);
        let r = copy_out_of(
            &format!("/work/{}", c.base),
            &dst,
            a.clone(),
            gnu_totals(a.len()),
            0,
        )
        .await;
        rejected(&r);
        for entry in walk(outer.path()) {
            if entry.starts_with(&target) {
                continue;
            }
            assert_ne!(
                entry.file_name().unwrap(),
                "evil",
                "{}: {} was written",
                c.name,
                entry.display()
            );
        }
        assert_eq!(mode(&victim), 0o700, "{}: victim changed", c.name);
    }
}

fn walk(p: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for ent in fs::read_dir(p).unwrap() {
        let path = ent.unwrap().path();
        out.push(path.clone());
        if fs::symlink_metadata(&path).unwrap().is_dir() {
            out.extend(walk(&path));
        }
    }
    out
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_out_refuses_an_entry_outside_the_requested_top() {
    let home = tempfile::tempdir().unwrap();
    let a = archive(&[
        e("results.csv", b'0', 0o644, b"1,2\n", ""),
        e(".bashrc", b'0', 0o644, b"evil", ""),
    ]);
    let r = copy_out_of(
        "/out/results.csv",
        home.path(),
        a.clone(),
        gnu_totals(a.len()),
        0,
    )
    .await;
    assert!(
        rejected(&r).contains("outside results.csv"),
        "{:?}",
        r.result
    );
    assert!(!home.path().join(".bashrc").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_out_refuses_a_hard_link_to_a_file_it_did_not_write() {
    let dst = tempfile::tempdir().unwrap();
    fs::create_dir(dst.path().join("tree")).unwrap();
    fs::write(dst.path().join("tree/mine"), b"the user's").unwrap();
    let a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/h", b'1', 0o644, b"", "tree/mine"),
    ]);
    let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
    assert!(rejected(&r).contains("did not write"), "{:?}", r.result);
    assert!(!dst.path().join("tree/h").exists());

    // A hard link to a file this copy wrote is made.
    let dst = tempfile::tempdir().unwrap();
    let a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/a", b'0', 0o644, b"body", ""),
        e("tree/h", b'1', 0o644, b"", "tree/a"),
    ]);
    let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
    assert!(matches!(r.result, Ok(0)), "{:?}", r.result);
    assert_eq!(fs::read(dst.path().join("tree/h")).unwrap(), b"body");
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_out_refuses_a_sparse_entry() {
    // A GNU sparse header with an empty map.
    let dst = tempfile::tempdir().unwrap();
    let a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/s", b'S', 0o644, b"", ""),
    ]);
    let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
    assert!(rejected(&r).contains("sparse"), "{:?}", r.result);

    // A PAX sparse file.
    let mut b = tar::Builder::new(Vec::new());
    b.append_pax_extensions([
        ("GNU.sparse.major", &b"1"[..]),
        ("GNU.sparse.minor", &b"0"[..]),
    ])
    .unwrap();
    let mut h = tar::Header::new_gnu();
    h.set_size(1);
    h.set_mode(0o644);
    b.append_data(&mut h, "tree", &b"x"[..]).unwrap();
    let a = b.into_inner().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
    assert!(rejected(&r).contains("sparse"), "{:?}", r.result);
}

/// A GNU sparse header followed by extension blocks without end.
struct EndlessSparse {
    name: Vec<u8>,
    sent: Arc<std::sync::atomic::AtomicU64>,
    limit: u64,
    block: Vec<u8>,
}

impl EndlessSparse {
    /// Stops after `limit` bytes; `name` comes before it as a GNU long name
    /// when set.
    fn new(long_name: Option<&str>, limit: u64) -> Self {
        let mut name = Vec::new();
        if let Some(n) = long_name {
            name = archive(&[e("././@LongLink", b'L', 0o644, n.as_bytes(), "")]);
            name.truncate(name.len() - 1024);
        }
        Self {
            name,
            sent: Arc::default(),
            limit,
            block: Vec::new(),
        }
    }

    fn next_block(&self, sent: u64) -> Vec<u8> {
        if sent == 0 {
            let mut h = tar::Header::new_gnu();
            let g = h.as_gnu_mut().unwrap();
            g.name[..6].copy_from_slice(b"tree/s");
            g.set_is_extended(true);
            h.set_entry_type(tar::EntryType::GNUSparse);
            h.set_mode(0o644);
            h.set_size(0);
            h.set_cksum();
            return h.as_bytes().to_vec();
        }
        let mut x = tar::GnuExtSparseHeader::new();
        x.set_is_extended(true);
        x.as_bytes().to_vec()
    }
}

impl Read for EndlessSparse {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::sync::atomic::Ordering;
        let sent = self.sent.load(Ordering::SeqCst);
        if sent >= self.limit {
            return Ok(0);
        }
        if self.block.is_empty() {
            let head = self.name.len() as u64;
            self.block = if sent < head {
                self.name[sent as usize..].to_vec()
            } else {
                self.next_block(sent - head)
            };
        }
        let n = buf.len().min(self.block.len());
        buf[..n].copy_from_slice(&self.block[..n]);
        self.block.drain(..n);
        self.sent.fetch_add(n as u64, Ordering::SeqCst);
        Ok(n)
    }
}

/// A GNU sparse header is refused before the `tar` crate reads its chain of
/// extension blocks, which it would read without limit.
#[test]
fn a_sparse_header_is_refused_before_its_extension_blocks() {
    for (long, want) in [
        (None, "\"tree/s\""),
        (Some("tree/long-name"), "\"tree/long-name\""),
        (Some("tree/\x1b]52;c;x\x07"), "\"tree/\\x1b]52;c;x\\x07\""),
    ] {
        let src = EndlessSparse::new(long, 4 << 20);
        let sent = src.sent.clone();
        let mut a = tar::Archive::new(capped::Capped::new(src));
        let err = match a.entries().unwrap().next() {
            Some(Err(e)) => e.to_string(),
            Some(Ok(_)) => panic!("an entry came back"),
            None => panic!("no entry"),
        };
        let read = sent.load(std::sync::atomic::Ordering::SeqCst);
        assert!(err.contains("it is a sparse file"), "{err}");
        assert!(err.contains(want), "{err}");
        assert!(read <= 4096, "read {read} bytes");
    }
}

/// Through the copy, a sparse header chained to more than a MiB of
/// extension blocks is refused at once.
#[tokio::test(flavor = "multi_thread")]
async fn copy_out_refuses_a_sparse_entry_with_endless_extensions() {
    let mut src = EndlessSparse::new(None, 2 << 20);
    let mut a = archive(&[e("tree/", b'5', 0o755, b"", "")]);
    a.truncate(a.len() - 1024);
    std::io::copy(&mut src, &mut a).unwrap();
    let dst = tempfile::tempdir().unwrap();
    let r = copy_out_of("/work/tree", dst.path(), a, String::new(), 0).await;
    assert!(
        rejected(&r).contains("\"tree/s\": it is a sparse file"),
        "{:?}",
        r.result
    );
}

/// A GNU long name or long link, or a PAX entry, over 1 MiB is refused
/// before it is read into memory.
#[tokio::test(flavor = "multi_thread")]
async fn copy_out_refuses_an_extension_entry_over_a_mib() {
    let big = vec![b'a'; (2 << 20) + 1];
    for kind in *b"LKxg" {
        let dst = tempfile::tempdir().unwrap();
        let a = archive(&[
            e("tree/", b'5', 0o755, b"", ""),
            e("././@LongLink", kind, 0o644, &big, ""),
            e("tree/a", b'0', 0o644, b"x", ""),
        ]);
        let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
        let msg = rejected(&r);
        assert!(
            msg.contains("over the 1048576-byte limit"),
            "{}: {}",
            kind as char,
            &msg[..msg.len().min(300)]
        );
        assert!(!dst.path().join("tree/a").exists());
    }
}

/// A refused entry's name is cut short in the message.
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_long_name_is_cut_short() {
    let dst = tempfile::tempdir().unwrap();
    let mut name = "other/".to_string();
    name.push_str(&"\x1b".repeat(5000));
    let a = archive(&[
        e("././@LongLink", b'L', 0o644, name.as_bytes(), ""),
        e("other", b'0', 0o644, b"x", ""),
    ]);
    let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
    let msg = rejected(&r);
    assert!(msg.contains("outside tree"), "{msg}");
    assert!(msg.len() < 1000, "{} bytes", msg.len());
    assert!(!msg.contains('\x1b'));
}

/// A PAX `size` moves the next header, so a file whose body looks like an
/// oversized extension header is not taken for one.
#[tokio::test(flavor = "multi_thread")]
async fn a_pax_size_is_followed_to_the_next_header() {
    let mut fake = tar::Header::new_gnu();
    fake.as_gnu_mut().unwrap().name[..13].copy_from_slice(b"././@LongLink");
    fake.set_entry_type(tar::EntryType::GNULongName);
    fake.set_size(4 << 30);
    fake.set_cksum();
    let body = fake.as_bytes().to_vec();

    let mut b = tar::Builder::new(Vec::new());
    let mut h = tar::Header::new_gnu();
    h.set_entry_type(tar::EntryType::Directory);
    h.set_mode(0o755);
    h.set_size(0);
    b.append_data(&mut h, "tree/", std::io::empty()).unwrap();
    let size = format!("{}", body.len());
    b.append_pax_extensions([("size", size.as_bytes())])
        .unwrap();
    let mut h = tar::Header::new_gnu();
    h.set_entry_type(tar::EntryType::Regular);
    h.set_mode(0o644);
    // The header's own size, which the PAX record overrides.
    h.set_size(0);
    h.set_path("tree/a").unwrap();
    h.set_cksum();
    b.append(&h, &body[..]).unwrap();
    let a = b.into_inner().unwrap();

    let dst = tempfile::tempdir().unwrap();
    let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
    assert!(matches!(r.result, Ok(0)), "{:?}", r.result);
    assert_eq!(fs::read(dst.path().join("tree/a")).unwrap(), body);
}

/// A file entry replaces a file or symlink already at its path, and never
/// writes through the link.
#[tokio::test(flavor = "multi_thread")]
async fn copy_out_replaces_a_file_or_symlink_at_the_entry() {
    let dst = tempfile::tempdir().unwrap();
    fs::create_dir(dst.path().join("tree")).unwrap();
    fs::write(dst.path().join("tree/f"), b"old").unwrap();
    fs::set_permissions(dst.path().join("tree/f"), fs::Permissions::from_mode(0o444)).unwrap();
    fs::write(dst.path().join("victim"), b"the user's").unwrap();
    symlink("../victim", dst.path().join("tree/g")).unwrap();
    let a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/f", b'0', 0o644, b"new", ""),
        e("tree/g", b'0', 0o644, b"new", ""),
    ]);
    let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
    assert!(matches!(r.result, Ok(0)), "{:?}", r.result);
    assert_eq!(fs::read(dst.path().join("tree/f")).unwrap(), b"new");
    let g = dst.path().join("tree/g");
    assert!(fs::symlink_metadata(&g).unwrap().is_file());
    assert_eq!(fs::read(&g).unwrap(), b"new");
    assert_eq!(fs::read(dst.path().join("victim")).unwrap(), b"the user's");
}

/// A symlink at the copy's top is not followed, even to a directory inside
/// the destination.
#[tokio::test(flavor = "multi_thread")]
async fn copy_out_does_not_follow_a_symlink_at_the_top() {
    let dst = tempfile::tempdir().unwrap();
    fs::create_dir(dst.path().join("real")).unwrap();
    symlink("real", dst.path().join("tree")).unwrap();
    let a = archive(&[e("tree/evil", b'0', 0o644, b"x", "")]);
    let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
    rejected(&r);
    assert!(!dst.path().join("real/evil").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_out_fails_a_fifo_top_and_leaves_out_one_beneath() {
    let dst = tempfile::tempdir().unwrap();
    let a = archive(&[e("tree", b'6', 0o644, b"", "")]);
    let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
    assert!(
        rejected(&r).contains("top is not a file or a directory"),
        "{:?}",
        r.result
    );

    let dst = tempfile::tempdir().unwrap();
    let a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/f\x1b]52;", b'6', 0o644, b"", ""),
        e("tree/a", b'0', 0o644, b"x", ""),
    ]);
    let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
    let msg = rejected(&r).to_string();
    assert!(msg.contains("left out tree/f\\x1b]52;"), "{msg}");
    assert!(
        r.warnings.iter().any(|w| w.contains("not a file")),
        "{:?}",
        r.warnings
    );
    assert!(!msg.contains('\x1b') && r.warnings.iter().all(|w| !w.contains('\x1b')));
    assert_eq!(fs::read(dst.path().join("tree/a")).unwrap(), b"x");
}

/// A copy out is held to what tar says it wrote, and to the supervisor's
/// notice of anything it dropped.
#[tokio::test(flavor = "multi_thread")]
async fn copy_out_judges_the_stream() {
    let a = tree_archive();
    let marker = format!("\r\n[hedra] 4096 bytes of output (1 frames) {LOSS_MARKER}\r\n");
    let cases: Vec<(&str, String, i32, Option<&str>)> = vec![
        ("whole", gnu_totals(a.len()), 0, None),
        (
            "totals short",
            gnu_totals(a.len() + 4096),
            0,
            Some("arrived"),
        ),
        (
            "no totals from GNU tar",
            format!("{TOTALS_ANNOUNCED}\n"),
            0,
            Some("did not report"),
        ),
        (
            "the marker",
            gnu_totals(a.len()) + &marker,
            0,
            Some("dropped part of the stream"),
        ),
        (
            "the marker without GNU tar",
            marker.clone(),
            0,
            Some("not GNU tar"),
        ),
        ("busybox, whole", String::new(), 0, None),
        (
            "no tar",
            format!("{NO_TAR_MARKER}\n"),
            127,
            Some("has no `tar`"),
        ),
        ("no sh", String::new(), 2, None),
    ];
    for (name, stderr, code, want) in cases {
        let dst = tempfile::tempdir().unwrap();
        let archive = if name == "no tar" || name == "no sh" {
            vec![]
        } else {
            a.clone()
        };
        let r = copy_out_of("/work/tree", &dst.path().join("out"), archive, stderr, code).await;
        match (name, want) {
            ("no sh", _) => assert!(rejected(&r).contains("has no `tar`"), "{name}"),
            (_, Some(w)) => assert!(rejected(&r).contains(w), "{name}: {:?}", r.result),
            (_, None) => assert!(matches!(r.result, Ok(0)), "{name}: {:?}", r.result),
        }
        let kept = String::from_utf8_lossy(&r.remote_stderr);
        assert!(
            !kept.contains("Total bytes written") && !kept.contains(TOTALS_ANNOUNCED),
            "{name}: {kept}"
        );
    }

    // A busybox tar that fails is judged by its exit code.
    let dst = tempfile::tempdir().unwrap();
    let r = copy_out_of(
        "/work/tree",
        &dst.path().join("out"),
        a.clone(),
        "tar: error\n".into(),
        1,
    )
    .await;
    assert!(matches!(r.result, Ok(1)), "{:?}", r.result);
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_out_reports_a_call_cut_off_part_way() {
    let dst = tempfile::tempdir().unwrap();
    let big = vec![b'x'; 64 << 10];
    let a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/big.bin", b'0', 0o644, &big, ""),
    ]);
    let plan = plan_copy_out(&spec("/work/tree", &dst.path().join("out"), Direction::Out)).unwrap();
    let (sink, source, mut fake) = call();
    tokio::spawn(async move {
        fake.argv().await;
        fake.data(Stream::Stdout, &a[..2 * 512 + 10_000]).await;
        let _ = fake.to_client.send(Err(FakeErr("cut"))).await;
    });
    let r = copy_out(plan, sink, source, Arc::new(Progress::default())).await;
    assert!(
        matches!(r.result, Err(CopyError::Call(FakeErr("cut")))),
        "{:?}",
        r.result
    );
    assert_eq!(r.stopped_at.as_deref(), Some("out/big.bin"));
}

/// A sandbox taking a copy in: it answers `answer`, reads stdin, exits 0.
async fn copy_into(
    local: &Path,
    remote: &str,
    answer: &'static str,
) -> (CopyReport<FakeErr>, Vec<String>, Vec<u8>) {
    let plan = plan_copy_in(&spec(remote, local, Direction::In)).unwrap();
    let (sink, source, mut fake) = call();
    let sandbox = tokio::spawn(async move {
        let argv = fake.argv().await;
        fake.data(Stream::Stdout, format!("{answer}\n").as_bytes())
            .await;
        let (stdin, eof) = fake.stdin().await;
        assert!(eof, "stdin did not end with stdin_eof");
        fake.exit(0).await;
        (argv, stdin)
    });
    let r = copy_in(plan, sink, source, Arc::new(Progress::default())).await;
    let (argv, stdin) = sandbox.await.unwrap();
    (r, argv, stdin)
}

#[derive(Debug, PartialEq)]
struct Got {
    name: String,
    kind: u8,
    mode: u32,
    body: Vec<u8>,
    link: String,
}

fn read_archive(b: &[u8]) -> Vec<Got> {
    assert_eq!(b.len() % TAR_RECORD as usize, 0, "not whole records");
    let mut out = Vec::new();
    let mut ar = tar::Archive::new(b);
    for ent in ar.entries().unwrap() {
        let mut ent = ent.unwrap();
        let h = ent.header();
        assert_eq!((h.uid().unwrap(), h.gid().unwrap()), (0, 0));
        assert!(h.username_bytes().unwrap_or_default().is_empty());
        assert!(h.groupname_bytes().unwrap_or_default().is_empty());
        let kind = h.entry_type().as_byte();
        let mode = if kind == b'2' {
            0
        } else {
            h.mode().unwrap() & 0o7777
        };
        let link = ent
            .link_name_bytes()
            .map(|l| String::from_utf8_lossy(&l).into_owned())
            .unwrap_or_default();
        let name = String::from_utf8_lossy(&ent.path_bytes()).into_owned();
        let mut body = Vec::new();
        ent.read_to_end(&mut body).unwrap();
        out.push(Got {
            name,
            kind,
            mode,
            body,
            link,
        });
    }
    out
}

/// A tree with a nested directory, an empty one, a symlink, an executable,
/// and a symlink to a file outside it.
fn fixture() -> (tempfile::TempDir, PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("tree");
    fs::create_dir_all(root.join("nested")).unwrap();
    fs::create_dir_all(root.join("empty")).unwrap();
    fs::set_permissions(root.join("empty"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(root.join("nested"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(root.join("nested/a.txt"), b"hello").unwrap();
    fs::set_permissions(root.join("nested/a.txt"), fs::Permissions::from_mode(0o644)).unwrap();
    fs::write(root.join("run.sh"), b"#!/bin/sh\n").unwrap();
    fs::set_permissions(root.join("run.sh"), fs::Permissions::from_mode(0o755)).unwrap();
    symlink("nested/a.txt", root.join("link")).unwrap();
    fs::write(t.path().join("secret"), b"credentials").unwrap();
    symlink("../secret", root.join("outside")).unwrap();
    (t, root)
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_in_streams_the_tree_as_tar() {
    let (_t, src) = fixture();
    let s = src.to_str().unwrap().to_string();
    let cases = [
        (s.clone(), "/work", "dir", "tree", vec!["/work", "/"]),
        (
            s.clone(),
            "data/copy",
            "new",
            "copy",
            vec!["data/copy", "data"],
        ),
        (s.clone(), "", "dir", "tree", vec![".", "."]),
        (format!("{s}/."), "/work", "dir", ".", vec!["/work", "/"]),
    ];
    for (local, remote, answer, root, want_args) in cases {
        let (r, argv, stdin) = copy_into(Path::new(&local), remote, answer).await;
        assert!(matches!(r.result, Ok(0)), "{local}: {:?}", r.result);
        assert_eq!(argv[..4], ["sh", "-c", UPLOAD_SCRIPT, "cp"]);
        assert_eq!(argv[4..], want_args);
        let p = |rel: &str| {
            if root == "." {
                rel.to_string()
            } else {
                format!("{root}/{rel}")
            }
        };
        let mut want = Vec::new();
        if root != "." {
            want.push(Got {
                name: format!("{root}/"),
                kind: b'5',
                mode: 0o755,
                body: vec![],
                link: String::new(),
            });
        }
        want.extend([
            Got {
                name: p("empty/"),
                kind: b'5',
                mode: 0o755,
                body: vec![],
                link: String::new(),
            },
            Got {
                name: p("link"),
                kind: b'2',
                mode: 0,
                body: vec![],
                link: "nested/a.txt".into(),
            },
            Got {
                name: p("nested/"),
                kind: b'5',
                mode: 0o755,
                body: vec![],
                link: String::new(),
            },
            Got {
                name: p("nested/a.txt"),
                kind: b'0',
                mode: 0o644,
                body: b"hello".to_vec(),
                link: String::new(),
            },
            // A symlink to a file outside the tree is sent as the link.
            Got {
                name: p("outside"),
                kind: b'2',
                mode: 0,
                body: vec![],
                link: "../secret".into(),
            },
            Got {
                name: p("run.sh"),
                kind: b'0',
                mode: 0o755,
                body: b"#!/bin/sh\n".to_vec(),
                link: String::new(),
            },
        ]);
        let got = read_archive(&stdin);
        let norm = |g: &Got| {
            (
                g.name.trim_end_matches('/').to_string(),
                g.kind,
                g.mode,
                g.body.clone(),
                g.link.clone(),
            )
        };
        assert_eq!(
            got.iter().map(norm).collect::<Vec<_>>(),
            want.iter().map(norm).collect::<Vec<_>>(),
            "{local}"
        );
    }

    // A single file is one entry, named for where it lands.
    let (r, _, stdin) = copy_into(&src.join("run.sh"), "/work/start.sh", "new").await;
    assert!(matches!(r.result, Ok(0)), "{:?}", r.result);
    let got = read_archive(&stdin);
    assert_eq!(got.len(), 1);
    assert_eq!(
        (got[0].name.as_str(), got[0].body.as_slice()),
        ("start.sh", &b"#!/bin/sh\n"[..])
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_in_follows_a_symlinked_top_only() {
    let (t, src) = fixture();
    let link = t.path().join("via");
    symlink(&src, &link).unwrap();
    let (r, _, stdin) = copy_into(&link, "/work", "dir").await;
    assert!(matches!(r.result, Ok(0)), "{:?}", r.result);
    let got = read_archive(&stdin);
    // Its entries keep the name given on the command line.
    assert!(got
        .iter()
        .any(|g| g.name == "via/nested/a.txt" && g.body == b"hello"));
    assert!(got.iter().all(|g| g.body != b"credentials"));
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_in_refuses_a_fifo_top_and_leaves_out_one_beneath() {
    let t = tempfile::tempdir().unwrap();
    let fifo = t.path().join("pipe");
    mkfifo(&fifo);
    let err = plan_copy_in(&spec("/work", &fifo, Direction::In)).unwrap_err();
    assert!(err.contains("not a file or a directory"), "{err}");

    let (_t, src) = fixture();
    mkfifo(&src.join("pipe"));
    let (r, _, stdin) = copy_into(&src, "/work", "dir").await;
    assert!(
        rejected(&r).contains("left out tree/pipe"),
        "{:?}",
        r.result
    );
    assert!(r
        .warnings
        .iter()
        .any(|w| w.contains("not a file, a directory or a symlink")));
    assert!(read_archive(&stdin)
        .iter()
        .all(|g| !g.name.contains("pipe")));
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_in_cancels_the_call_on_an_unreadable_file() {
    let (_t, src) = fixture();
    let locked = src.join("nested/locked");
    fs::write(&locked, b"x").unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::File::open(&locked).is_ok() {
        return; // running as root
    }
    let plan = plan_copy_in(&spec("/work", &src, Direction::In)).unwrap();
    let (sink, source, mut fake) = call();
    let sandbox = tokio::spawn(async move {
        fake.argv().await;
        fake.data(Stream::Stdout, b"dir\n").await;
        let (_, eof) = fake.stdin().await;
        // The call is cancelled: stdin never ends cleanly and the copy
        // stops reading.
        fake.to_client.closed().await;
        eof
    });
    let r = copy_in(plan, sink, source, Arc::new(Progress::default())).await;
    let msg = rejected(&r).to_string();
    assert!(msg.contains("nested/locked"), "{msg}");
    assert!(!sandbox.await.unwrap(), "stdin_eof was sent");
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_in_reports_tar_failures_and_a_missing_tar() {
    let (_t, src) = fixture();
    let run = |answer: Option<&'static str>, stderr: &'static str, code: i32| {
        let src = src.clone();
        async move {
            let plan = plan_copy_in(&spec("/work", &src, Direction::In)).unwrap();
            let (sink, source, mut fake) = call();
            tokio::spawn(async move {
                fake.argv().await;
                if let Some(a) = answer {
                    fake.data(Stream::Stdout, format!("{a}\n").as_bytes()).await;
                    let _ = fake.from_client.recv().await;
                }
                fake.data(Stream::Stderr, stderr.as_bytes()).await;
                fake.exit(code).await;
            });
            copy_in(plan, sink, source, Arc::new(Progress::default())).await
        }
    };
    let r = run(Some("dir"), "tar: Unexpected EOF in archive\n", 2).await;
    assert!(matches!(r.result, Ok(2)), "{:?}", r.result);
    assert_eq!(r.remote_stderr, b"tar: Unexpected EOF in archive\n");

    let r = run(None, "[hedra-cp] no tar\n", 127).await;
    assert!(rejected(&r).contains("has no `tar`"));
    assert!(r.remote_stderr.is_empty());

    let r = run(None, "mkdir: can't create directory '/nope'\n", 1).await;
    assert!(
        rejected(&r).contains("nothing was copied"),
        "{:?}",
        r.result
    );

    // A trailing slash names a directory that has to exist.
    let plan = plan_copy_in(&spec("/work/new/", &src, Direction::In)).unwrap();
    let (sink, source, mut fake) = call();
    tokio::spawn(async move {
        fake.argv().await;
        fake.data(Stream::Stdout, b"new\n").await;
        fake.to_client.closed().await;
    });
    let r = copy_in(plan, sink, source, Arc::new(Progress::default())).await;
    assert!(
        rejected(&r).contains("qwen:/work/new/ is not a directory"),
        "{:?}",
        r.result
    );
}

#[cfg(unix)]
#[test]
fn a_file_swapped_between_the_walk_and_its_open_fails() {
    use rustix::fs::{openat, statat, AtFlags, Mode, OFlags, CWD};
    for swap in ["symlink", "fifo", "hard link"] {
        let t = tempfile::tempdir().unwrap();
        fs::write(t.path().join("f"), b"real").unwrap();
        fs::write(t.path().join("other"), b"what the link points to").unwrap();
        let dirfd = openat(
            CWD,
            t.path(),
            OFlags::RDONLY | OFlags::DIRECTORY,
            Mode::empty(),
        )
        .unwrap();
        let seen = statat(&dirfd, "f", AtFlags::SYMLINK_NOFOLLOW).unwrap();
        fs::remove_file(t.path().join("f")).unwrap();
        match swap {
            "symlink" => symlink("other", t.path().join("f")).unwrap(),
            // Opens without following anything; only its identity tells.
            "hard link" => fs::hard_link(t.path().join("other"), t.path().join("f")).unwrap(),
            _ => mkfifo(&t.path().join("f")),
        }
        let mut b = tar::Builder::new(Vec::new());
        let mut walked = send::Walked::default();
        let err = send::test_entry(&mut b, &dirfd, "f", &seen, &t.path().join("f"), &mut walked)
            .unwrap_err();
        assert!(
            err.contains("changed while it was being copied"),
            "{swap}: {err}"
        );
        let out = b.into_inner().unwrap();
        assert!(
            !out.windows(23).any(|w| w == b"what the link points to"),
            "{swap}"
        );
    }
}

/// A file swapped for a symlink is refused without opening what it points
/// to: a FIFO's writer stays blocked, as it would not had the FIFO been
/// opened for reading.
#[test]
fn a_symlink_swapped_in_is_not_followed_to_its_target() {
    use rustix::fs::{openat, statat, AtFlags, Mode, OFlags, CWD};
    use std::sync::atomic::{AtomicBool, Ordering};
    let t = tempfile::tempdir().unwrap();
    let fifo = t.path().join("fifo");
    mkfifo(&fifo);
    fs::write(t.path().join("f"), b"real").unwrap();
    let dirfd = openat(
        CWD,
        t.path(),
        OFlags::RDONLY | OFlags::DIRECTORY,
        Mode::empty(),
    )
    .unwrap();
    let seen = statat(&dirfd, "f", AtFlags::SYMLINK_NOFOLLOW).unwrap();
    fs::remove_file(t.path().join("f")).unwrap();
    symlink("fifo", t.path().join("f")).unwrap();
    let opened = Arc::new(AtomicBool::new(false));
    let writer = {
        let (fifo, opened) = (fifo.clone(), opened.clone());
        std::thread::spawn(move || {
            let _w = fs::OpenOptions::new().write(true).open(&fifo).unwrap();
            opened.store(true, Ordering::SeqCst);
        })
    };
    let mut b = tar::Builder::new(Vec::new());
    let mut walked = send::Walked::default();
    let err =
        send::test_entry(&mut b, &dirfd, "f", &seen, &t.path().join("f"), &mut walked).unwrap_err();
    assert!(err.contains("changed while it was being copied"), "{err}");
    std::thread::sleep(std::time::Duration::from_millis(300));
    let followed = opened.load(Ordering::SeqCst);
    // Lets the writer's open return.
    let _r = rustix::fs::open(&fifo, OFlags::RDONLY | OFlags::NONBLOCK, Mode::empty()).unwrap();
    writer.join().unwrap();
    assert!(!followed, "the copy opened the FIFO the symlink points to");
}

#[test]
fn a_file_that_shrank_fails_naming_it_and_one_that_grew_is_cut() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("data.bin");
    fs::write(&p, b"12345").unwrap();
    let mut b = tar::Builder::new(Vec::new());
    let err = send::test_append(&mut b, fs::File::open(&p).unwrap(), 10, &p).unwrap_err();
    assert!(
        err.contains("data.bin") && err.contains("ended before the 10 bytes"),
        "{err}"
    );

    let mut b = tar::Builder::new(Vec::new());
    send::test_append(&mut b, fs::File::open(&p).unwrap(), 3, &p).unwrap();
    let out = b.into_inner().unwrap();
    let got = read_archive(&{
        let mut o = out.clone();
        o.resize(
            o.len().div_ceil(TAR_RECORD as usize) * TAR_RECORD as usize,
            0,
        );
        o
    });
    assert_eq!(got[0].body, b"123");
}

/// A tree copied in and then out again lands byte- and mode-identical.
#[tokio::test(flavor = "multi_thread")]
async fn a_tree_round_trips() {
    let (_t, src) = fixture();
    fs::remove_file(src.join("outside")).unwrap();
    let (r, _, stdin) = copy_into(&src, "/work", "dir").await;
    assert!(matches!(r.result, Ok(0)), "{:?}", r.result);
    let dst = tempfile::tempdir().unwrap();
    let out = dst.path().join("back");
    let r = copy_out_of(
        "/work/tree",
        &out,
        stdin.clone(),
        gnu_totals(stdin.len()),
        0,
    )
    .await;
    assert!(matches!(r.result, Ok(0)), "{:?}", r.result);
    let rels = |root: &Path| {
        let mut v: Vec<_> = walk(root)
            .into_iter()
            .map(|p| p.strip_prefix(root).unwrap().to_path_buf())
            .collect();
        v.sort();
        v
    };
    assert_eq!(rels(&src), rels(&out));
    for rel in rels(&src) {
        let (a, b) = (src.join(&rel), out.join(&rel));
        let (ma, mb) = (
            fs::symlink_metadata(&a).unwrap(),
            fs::symlink_metadata(&b).unwrap(),
        );
        assert_eq!(ma.file_type(), mb.file_type(), "{}", rel.display());
        if ma.is_symlink() {
            assert_eq!(fs::read_link(&a).unwrap(), fs::read_link(&b).unwrap());
            continue;
        }
        assert_eq!(mode(&a), mode(&b), "{}", rel.display());
        if ma.is_file() {
            assert_eq!(fs::read(&a).unwrap(), fs::read(&b).unwrap());
        }
    }
    assert_eq!(mode(&src), mode(&out));
}

#[test]
fn parse_args_finds_the_sandbox_side() {
    let valid = |n: &str| !n.is_empty() && !n.contains('/');
    let s = parse_args("./a.txt", "train:/work", valid).unwrap();
    assert_eq!(
        (s.direction, s.sandbox.as_str(), s.remote.as_str()),
        (Direction::In, "train", "/work")
    );
    let s = parse_args("train:/out", "here", valid).unwrap();
    assert_eq!(
        (s.direction, s.local.as_path()),
        (Direction::Out, Path::new("here"))
    );
    assert!(parse_args("a:/x", "b:/y", valid)
        .unwrap_err()
        .contains("both sides"));
    assert!(parse_args("./a:b", "c", valid)
        .unwrap_err()
        .contains("neither side"));
    assert!(parse_args("x", ":/work", valid)
        .unwrap_err()
        .contains("neither side"));
}

#[test]
fn plan_copy_out_reads_the_source_as_scp_does() {
    let t = tempfile::tempdir().unwrap();
    let p = plan_copy_out(&spec("/work/tree/.", t.path(), Direction::Out)).unwrap();
    assert_eq!(
        (p.argv[4].as_str(), p.base.as_str(), p.name.as_str()),
        ("/work/tree", ".", ".")
    );
    let p = plan_copy_out(&spec("", &t.path().join("new"), Direction::Out)).unwrap();
    assert_eq!(
        (p.argv[4].as_str(), p.argv[5].as_str(), p.name.as_str()),
        (".", ".", "new")
    );
    assert!(plan_copy_out(&spec("/", t.path(), Direction::Out))
        .unwrap_err()
        .contains("root"));
    assert!(plan_copy_out(&spec("/work/..", t.path(), Direction::Out)).is_err());
    // `a/..` would clean to `.`, the whole working directory.
    for climbs in ["a/..", "a/b/../", ".."] {
        let e = plan_copy_out(&spec(climbs, t.path(), Direction::Out)).unwrap_err();
        assert!(e.contains("ends in .."), "{climbs}: {e}");
    }
    let slash = format!("{}/missing/", t.path().display());
    assert!(
        plan_copy_out(&spec("/x", Path::new(&slash), Direction::Out))
            .unwrap_err()
            .contains("not a directory")
    );
}

/// The scripts, run by the local `sh` and `tar`.
mod scripts {
    use super::*;
    use std::process::{Command, Stdio};

    fn sh(script: &str, args: &[&str], path: Option<&str>, stdin: &[u8]) -> (Vec<u8>, String, i32) {
        let mut cmd = Command::new("/bin/sh");
        cmd.arg("-c").arg(script).arg("cp").args(args);
        // macOS's tar adds AppleDouble entries without it.
        cmd.env("COPYFILE_DISABLE", "1");
        if let Some(p) = path {
            cmd.env("PATH", p);
        }
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        std::io::Write::write_all(child.stdin.as_mut().unwrap(), stdin).unwrap();
        drop(child.stdin.take());
        let out = child.wait_with_output().unwrap();
        (
            out.stdout,
            String::from_utf8_lossy(&out.stderr).into_owned(),
            out.status.code().unwrap_or(-1),
        )
    }

    #[test]
    fn the_upload_script_creates_a_missing_parent() {
        let t = tempfile::tempdir().unwrap();
        let dst = t.path().join("work/new");
        let parent = t.path().join("work");
        let a = archive(&[e("new", b'0', 0o644, b"body", "")]);
        let (out, err, code) = sh(
            UPLOAD_SCRIPT,
            &[dst.to_str().unwrap(), parent.to_str().unwrap()],
            None,
            &a,
        );
        assert_eq!((out.as_slice(), code), (&b"new\n"[..], 0), "{err}");
        assert_eq!(fs::read(&dst).unwrap(), b"body");
    }

    /// A `CDPATH` in the sandbox's environment neither moves the copy nor
    /// adds to the line the script prints.
    #[test]
    fn the_upload_script_ignores_cdpath() {
        let t = tempfile::tempdir().unwrap();
        let (cwd, other) = (t.path().join("cwd"), t.path().join("other"));
        fs::create_dir_all(cwd.join("data")).unwrap();
        fs::create_dir_all(other.join("data")).unwrap();
        let a = archive(&[e("f", b'0', 0o644, b"body", "")]);
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg(UPLOAD_SCRIPT)
            .args(["cp", "data", "."])
            .current_dir(&cwd)
            .env("CDPATH", &other)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        std::io::Write::write_all(child.stdin.as_mut().unwrap(), &a).unwrap();
        drop(child.stdin.take());
        let out = child.wait_with_output().unwrap();
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.stdout, b"dir\n", "{err}");
        assert_eq!(fs::read(cwd.join("data/f")).unwrap(), b"body");
        assert!(!other.join("data/f").exists());
    }

    #[test]
    fn both_scripts_print_the_marker_without_tar() {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().to_str().unwrap();
        for script in [UPLOAD_SCRIPT, DOWNLOAD_SCRIPT] {
            let (_, err, code) = sh(script, &[d, d], Some("/nonexistent"), b"");
            assert_eq!(code, 127, "{err}");
            let mut s = stderr::CopyStderr::new();
            s.write(err.as_bytes());
            s.flush();
            assert!(s.no_tar && s.kept.is_empty(), "{err}");
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_download_script_feeds_a_copy_out() {
        let (_t, src) = fixture();
        fs::remove_file(src.join("outside")).unwrap();
        let parent = src.parent().unwrap().to_str().unwrap().to_string();
        let (out, err, code) = sh(DOWNLOAD_SCRIPT, &[&parent, "tree"], None, b"");
        assert_eq!(code, 0, "{err}");
        let dst = tempfile::tempdir().unwrap();
        let r = copy_out_of(
            &format!("{parent}/tree"),
            &dst.path().join("got"),
            out,
            err,
            code,
        )
        .await;
        assert!(matches!(r.result, Ok(0)), "{:?}", r.result);
        assert_eq!(
            fs::read(dst.path().join("got/nested/a.txt")).unwrap(),
            b"hello"
        );
    }
}

/// A walk that fails right after a whole chunk's worth of entries sends
/// nothing more: the end blocks the builder would write on drop fill that
/// chunk exactly, and must not reach the sandbox before the reset.
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_walk_sends_no_end_of_archive() {
    let t = tempfile::tempdir().unwrap();
    let src = t.path().join("tree");
    fs::create_dir_all(&src).unwrap();
    // `tree/`'s header, `tree/a`'s header and body, and two end blocks
    // make exactly one chunk.
    fs::write(src.join("a"), vec![7u8; CHUNK - 4 * 512]).unwrap();
    fs::write(src.join("b"), b"x").unwrap();
    fs::set_permissions(src.join("b"), fs::Permissions::from_mode(0o000)).unwrap();
    if fs::File::open(src.join("b")).is_ok() {
        return; // running as root
    }
    let plan = plan_copy_in(&spec("/work", &src, Direction::In)).unwrap();
    let (sink, source, mut fake) = call();
    let sandbox = tokio::spawn(async move {
        fake.argv().await;
        fake.data(Stream::Stdout, b"dir\n").await;
        fake.stdin().await
    });
    let r = copy_in(plan, sink, source, Arc::new(Progress::default())).await;
    assert!(rejected(&r).contains("tree/b"), "{:?}", r.result);
    let (got, eof) = sandbox.await.unwrap();
    assert!(!eof, "stdin_eof was sent");
    assert!(
        got.len() < 1024 || got[got.len() - 1024..].iter().any(|b| *b != 0),
        "the sandbox got the archive's end blocks ({} bytes)",
        got.len()
    );
}

/// Dropping a copy in part way, as a signal does, resets the call rather
/// than letting its sender finish the stream.
#[tokio::test(flavor = "multi_thread")]
async fn dropping_a_copy_in_resets_its_call() {
    let t = tempfile::tempdir().unwrap();
    let src = t.path().join("big.bin");
    fs::write(&src, vec![1u8; 4 << 20]).unwrap();
    let plan = plan_copy_in(&spec("/work", &src, Direction::In)).unwrap();
    let (sink, source, mut fake) = call();
    let (some_tx, some_rx) = tokio::sync::oneshot::channel();
    let sandbox = tokio::spawn(async move {
        fake.argv().await;
        fake.data(Stream::Stdout, b"dir\n").await;
        // One chunk of stdin, then the copy is dropped.
        while let Some(f) = fake.from_client.recv().await {
            if matches!(f.frame, Some(exec_client_frame::Frame::Stdin(_))) {
                break;
            }
        }
        let _ = some_tx.send(());
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let mut eof = false;
        while let Some(f) = fake.from_client.recv().await {
            eof |= matches!(f.frame, Some(exec_client_frame::Frame::StdinEof(_)));
        }
        eof
    });
    tokio::select! {
        _ = copy_in(plan, sink, source, Arc::new(Progress::default())) => panic!("the copy finished"),
        _ = some_rx => {}
    }
    let eof = tokio::time::timeout(std::time::Duration::from_secs(20), sandbox)
        .await
        .expect("the call was not closed")
        .unwrap();
    assert!(!eof, "the dropped copy still ended stdin");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_case_or_normalization_collision_fails_the_copy_where_the_destination_folds() {
    let dst = tempfile::tempdir().unwrap();
    let folding = extract::probe(
        &cap_std::fs::Dir::open_ambient_dir(dst.path(), cap_std::ambient_authority()).unwrap(),
    );
    for (first, second, folds) in [
        ("tree/Makefile", "tree/makefile", folding.case),
        ("tree/caf\u{e9}", "tree/cafe\u{301}", folding.normalization),
    ] {
        let a = archive(&[
            e("tree/", b'5', 0o755, b"", ""),
            e(first, b'0', 0o644, b"first", ""),
            e(second, b'0', 0o644, b"second", ""),
        ]);
        let dst = tempfile::tempdir().unwrap();
        let out = dst.path().join("out");
        let r = copy_out_of("/work/tree", &out, a.clone(), gnu_totals(a.len()), 0).await;
        if folds {
            let msg = rejected(&r);
            assert!(msg.contains("left out"), "{msg}");
            assert!(
                r.warnings
                    .iter()
                    .any(|w| w.contains("does not tell its name")),
                "{:?}",
                r.warnings
            );
            assert_eq!(fs::read_dir(&out).unwrap().count(), 1);
            assert_eq!(fs::read(out.join(&first[5..])).unwrap(), b"first");
        } else {
            assert!(matches!(r.result, Ok(0)), "{second}: {:?}", r.result);
            assert_eq!(fs::read_dir(&out).unwrap().count(), 2);
        }
    }
}

#[test]
fn collisions_fold_as_the_destination_does() {
    use extract::{Collisions, Folding};
    let both = Folding {
        case: true,
        normalization: true,
    };
    let mut c = Collisions::new(both);
    assert_eq!(c.check("t/Makefile"), None);
    assert_eq!(
        c.check("t/Makefile"),
        None,
        "the same name again is not a collision"
    );
    assert_eq!(c.check("t/makefile").as_deref(), Some("t/Makefile"));
    assert_eq!(c.check("t/caf\u{e9}"), None);
    assert_eq!(c.check("t/cafe\u{301}").as_deref(), Some("t/caf\u{e9}"));
    assert_eq!(c.check("t/CAFE\u{301}").as_deref(), Some("t/caf\u{e9}"));

    let mut c = Collisions::new(Folding {
        case: true,
        normalization: false,
    });
    assert_eq!(c.check("t/caf\u{e9}"), None);
    assert_eq!(c.check("t/cafe\u{301}"), None);
    assert_eq!(c.check("t/CAF\u{c9}").as_deref(), Some("t/caf\u{e9}"));

    let mut c = Collisions::new(Folding {
        case: false,
        normalization: true,
    });
    assert_eq!(c.check("t/Makefile"), None);
    assert_eq!(c.check("t/makefile"), None);
    assert_eq!(c.check("t/cafe\u{301}"), None);
    assert_eq!(c.check("t/caf\u{e9}").as_deref(), Some("t/cafe\u{301}"));

    let mut c = Collisions::new(Folding {
        case: false,
        normalization: false,
    });
    assert_eq!(c.check("t/Makefile"), None);
    assert_eq!(c.check("t/makefile"), None);
}

/// A stream cut at a header's boundary, with no end blocks, is not a whole
/// copy even when tar exits 0 and reports no totals.
#[tokio::test(flavor = "multi_thread")]
async fn a_stream_cut_at_a_header_fails_the_copy() {
    let mut a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/a", b'0', 0o644, b"aaaa", ""),
        e("tree/b", b'0', 0o644, b"bbbb", ""),
    ]);
    a.truncate(512 * 3);
    let dst = tempfile::tempdir().unwrap();
    let r = copy_out_of("/work/tree", &dst.path().join("out"), a, String::new(), 0).await;
    assert!(
        rejected(&r).contains("ended before the archive did"),
        "{:?}",
        r.result
    );
}

/// Padding past the end blocks, more than the channel holds, is read on to
/// the exit code.
#[tokio::test(flavor = "multi_thread")]
async fn padding_past_the_end_blocks_is_read_to_the_exit_code() {
    let mut a = tree_archive();
    a.resize(a.len() + (DOWN_CHUNKS + 64) * 4096, 0);
    let dst = tempfile::tempdir().unwrap();
    let out = dst.path().join("out");
    let r = copy_out_of("/work/tree", &out, a.clone(), gnu_totals(a.len()), 0).await;
    assert!(matches!(r.result, Ok(0)), "{:?}", r.result);
    assert_eq!(fs::read(out.join("nested/a.txt")).unwrap(), b"hello");
}

#[test]
fn a_directory_entry_does_not_keep_a_symlink_at_its_path() {
    let t = tempfile::tempdir().unwrap();
    fs::create_dir(t.path().join("real")).unwrap();
    symlink("real", t.path().join("d")).unwrap();
    let dir = cap_std::fs::Dir::open_ambient_dir(t.path(), cap_std::ambient_authority()).unwrap();
    let err = extract::mkdir_over(&dir, "d").unwrap_err();
    assert!(err.to_string().contains("not a directory"), "{err}");
    extract::mkdir_over(&dir, "real").unwrap();
    extract::mkdir_over(&dir, "new").unwrap();
    assert!(t.path().join("new").is_dir());
}

fn cap_dir(p: &Path) -> cap_std::fs::Dir {
    cap_std::fs::Dir::open_ambient_dir(p, cap_std::ambient_authority()).unwrap()
}

/// A directory's mode and mtime are never set through a symlink at its path.
#[test]
fn directory_attributes_are_not_set_through_a_symlink() {
    let t = tempfile::tempdir().unwrap();
    fs::create_dir(t.path().join("real")).unwrap();
    fs::set_permissions(t.path().join("real"), fs::Permissions::from_mode(0o755)).unwrap();
    symlink("real", t.path().join("d")).unwrap();
    let when = std::time::UNIX_EPOCH + std::time::Duration::from_secs(MTIME);
    assert!(extract::set_dir_attrs(&cap_dir(t.path()), "d", 0o700, when).is_err());
    assert_eq!(mode(&t.path().join("real")), 0o755);
    extract::set_dir_attrs(&cap_dir(t.path()), "real", 0o700, when).unwrap();
    assert_eq!(mode(&t.path().join("real")), 0o700);
}

/// The folding probe creates its file anew: a file or a symlink already at
/// the probe's name, and what the symlink points to, are left as they were.
#[test]
fn the_folding_probe_never_opens_a_file_already_there() {
    let t = tempfile::tempdir().unwrap();
    let id = std::process::id();
    let probe = t.path().join(format!(".hedra-cp-probe-{id}"));
    fs::write(&probe, b"mine").unwrap();
    fs::write(t.path().join("victim"), b"keep").unwrap();
    let nfc = t.path().join(format!(".hedra-cp-probe-\u{e9}-{id}"));
    symlink("victim", &nfc).unwrap();
    let f = extract::probe(&cap_dir(t.path()));
    assert!(f.case && f.normalization, "{f:?}");
    assert_eq!(fs::read(&probe).unwrap(), b"mine");
    assert_eq!(fs::read(t.path().join("victim")).unwrap(), b"keep");
    assert!(fs::symlink_metadata(&nfc).unwrap().file_type().is_symlink());
}

/// Entries beneath an existing top are checked against what the top's own
/// directory folds, which can differ from its parent's, as on another mount.
#[test]
fn collisions_beneath_an_existing_top_follow_the_top_s_folding() {
    let both = extract::Folding {
        case: true,
        normalization: true,
    };
    let none = extract::Folding {
        case: false,
        normalization: false,
    };
    // A directory holding `.folds` folds; any other does not.
    let fake = |d: &cap_std::fs::Dir| if d.exists(".folds") { both } else { none };
    let a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/Makefile", b'0', 0o644, b"first", ""),
        e("tree/makefile", b'0', 0o644, b"second", ""),
    ]);
    for (outer_folds, inner_folds, want_left_out) in [
        (false, true, true),
        (true, false, false),
        (false, false, false),
    ] {
        let t = tempfile::tempdir().unwrap();
        let out = t.path().join("out");
        fs::create_dir_all(out.join("tree")).unwrap();
        if outer_folds {
            fs::write(out.join(".folds"), b"").unwrap();
        }
        if inner_folds {
            fs::write(out.join("tree/.folds"), b"").unwrap();
        }
        let (tx, rx) = mpsc::channel(4);
        tx.blocking_send(extract::Down::Data(a.clone().into()))
            .unwrap();
        tx.blocking_send(extract::Down::End).unwrap();
        let got = extract::extract_probing(rx, &out, "tree", "tree", &Progress::default(), &fake);
        assert_eq!(
            got.skipped.len(),
            usize::from(want_left_out),
            "outer {outer_folds}, inner {inner_folds}: {:?} {:?}",
            got.warnings,
            got.error
        );
    }
}

/// No symlink is created through another symlink the stream made.
#[tokio::test(flavor = "multi_thread")]
async fn copy_out_refuses_a_symlink_beneath_a_symlink_in_the_stream() {
    let dst = tempfile::tempdir().unwrap();
    let a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/l", b'2', 0o777, b"", "."),
        e("tree/l/m", b'2', 0o777, b"", "elsewhere"),
    ]);
    let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
    assert!(
        rejected(&r).contains("a symlink in the same stream"),
        "{:?}",
        r.result
    );
    assert!(fs::symlink_metadata(dst.path().join("tree/m")).is_err());
}

/// Directory modes are set deepest first by path, whatever the stream's
/// order, and a path's later entry wins.
#[tokio::test(flavor = "multi_thread")]
async fn directory_modes_go_deepest_first_by_path() {
    let dst = tempfile::tempdir().unwrap();
    let a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/a/", b'5', 0o755, b"", ""),
        e("tree/a/b/", b'5', 0o750, b"", ""),
        // Again, closed, after its child: set first in stream order
        // reversed, it would lock its child out.
        e("tree/a/", b'5', 0o000, b"", ""),
    ]);
    let r = copy_out_of("/work/tree", dst.path(), a.clone(), gnu_totals(a.len()), 0).await;
    let a_dir = dst.path().join("tree/a");
    let a_mode = fs::symlink_metadata(&a_dir).map(|m| m.permissions().mode() & 0o777);
    let _ = fs::set_permissions(&a_dir, fs::Permissions::from_mode(0o755));
    assert!(matches!(r.result, Ok(0)), "{:?}", r.result);
    assert_eq!(a_mode.unwrap(), 0);
    assert_eq!(mode(&a_dir.join("b")), 0o750 & !umask());
}

/// A failed tar does not hide an entry hedra-cli left out.
#[tokio::test(flavor = "multi_thread")]
async fn a_left_out_entry_fails_the_copy_when_tar_fails_too() {
    let dst = tempfile::tempdir().unwrap();
    let a = archive(&[
        e("tree/", b'5', 0o755, b"", ""),
        e("tree/pipe", b'6', 0o644, b"", ""),
    ]);
    let r = copy_out_of("/work/tree", dst.path(), a, String::new(), 2).await;
    let msg = rejected(&r);
    assert!(
        msg.contains("left out tree/pipe") && msg.contains("exited 2"),
        "{msg}"
    );

    let (_t, src) = fixture();
    mkfifo(&src.join("pipe"));
    let plan = plan_copy_in(&spec("/work", &src, Direction::In)).unwrap();
    let (sink, source, mut fake) = call();
    tokio::spawn(async move {
        fake.argv().await;
        fake.data(Stream::Stdout, b"dir\n").await;
        let _ = fake.stdin().await;
        fake.exit(2).await;
    });
    let r = copy_in(plan, sink, source, Arc::new(Progress::default())).await;
    let msg = rejected(&r);
    assert!(
        msg.contains("left out tree/pipe") && msg.contains("exited 2"),
        "{msg}"
    );
}
