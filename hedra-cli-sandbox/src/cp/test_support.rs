//! The fake exec and the archive builders the `cp` tests share on every OS.

#![allow(dead_code)] // each OS uses a different subset

use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FakeErr(pub(super) &'static str);

/// The receiving half the copy reads.
pub(super) struct FakeSource(pub(super) mpsc::Receiver<Result<ExecServerFrame, FakeErr>>);

impl ExecSource for FakeSource {
    type Error = FakeErr;
    async fn recv(&mut self) -> Result<Option<ExecServerFrame>, FakeErr> {
        match self.0.recv().await {
            Some(Ok(f)) => Ok(Some(f)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

/// The sandbox's half of the call.
pub(super) struct Fake {
    pub(super) from_client: mpsc::Receiver<ExecClientFrame>,
    pub(super) to_client: mpsc::Sender<Result<ExecServerFrame, FakeErr>>,
}

pub(super) fn call() -> (mpsc::Sender<ExecClientFrame>, FakeSource, Fake) {
    let (ctx, crx) = mpsc::channel(64);
    let (stx, srx) = mpsc::channel(4);
    (
        ctx,
        FakeSource(srx),
        Fake {
            from_client: crx,
            to_client: stx,
        },
    )
}

impl Fake {
    pub(super) async fn argv(&mut self) -> Vec<String> {
        match self.from_client.recv().await.and_then(|f| f.frame) {
            Some(exec_client_frame::Frame::Start(s)) => s.argv,
            other => panic!("the first frame is not start: {other:?}"),
        }
    }

    pub(super) async fn data(&self, stream: Stream, mut b: &[u8]) {
        while !b.is_empty() {
            let n = b.len().min(4096);
            let f = ExecServerFrame {
                frame: Some(exec_server_frame::Frame::Data(
                    crate::pb::hedra::sandbox::v1::ExecData {
                        stream: stream as i32,
                        chunk: b[..n].to_vec(),
                    },
                )),
            };
            if self.to_client.send(Ok(f)).await.is_err() {
                return;
            }
            b = &b[n..];
        }
    }

    pub(super) async fn exit(&self, code: i32) {
        let f = ExecServerFrame {
            frame: Some(exec_server_frame::Frame::ExitCode(code)),
        };
        let _ = self.to_client.send(Ok(f)).await;
    }

    /// Reads stdin to `stdin_eof`; false when the call ended first.
    pub(super) async fn stdin(&mut self) -> (Vec<u8>, bool) {
        let mut got = Vec::new();
        while let Some(f) = self.from_client.recv().await {
            match f.frame {
                Some(exec_client_frame::Frame::Stdin(b)) => got.extend(b),
                Some(exec_client_frame::Frame::StdinEof(_)) => return (got, true),
                _ => {}
            }
        }
        (got, false)
    }
}

/// One raw entry, written without the `tar` crate's path checks.
#[derive(Clone, Copy)]
pub(super) struct E<'a> {
    pub(super) name: &'a str,
    pub(super) kind: u8,
    pub(super) mode: u32,
    pub(super) body: &'a [u8],
    pub(super) link: &'a str,
}

pub(super) fn e<'a>(name: &'a str, kind: u8, mode: u32, body: &'a [u8], link: &'a str) -> E<'a> {
    E {
        name,
        kind,
        mode,
        body,
        link,
    }
}

pub(super) const MTIME: u64 = 1_700_000_000;

pub(super) fn archive(entries: &[E<'_>]) -> Vec<u8> {
    let mut out = Vec::new();
    for en in entries {
        let mut h = tar::Header::new_gnu();
        {
            let g = h.as_gnu_mut().unwrap();
            g.name[..en.name.len()].copy_from_slice(en.name.as_bytes());
            g.linkname[..en.link.len()].copy_from_slice(en.link.as_bytes());
            g.set_real_size(en.body.len() as u64);
        }
        h.set_entry_type(tar::EntryType::new(en.kind));
        h.set_mode(en.mode);
        h.set_size(en.body.len() as u64);
        h.set_mtime(MTIME);
        h.set_cksum();
        out.extend_from_slice(h.as_bytes());
        out.extend_from_slice(en.body);
        out.resize(out.len().div_ceil(512) * 512, 0);
    }
    out.resize(out.len() + 1024, 0);
    out
}

pub(super) fn gnu_totals(n: usize) -> String {
    format!("{TOTALS_ANNOUNCED}\nTotal bytes written: {n} (10KiB, 1.0MiB/s)\n")
}

pub(super) fn spec(remote: &str, local: &Path, direction: Direction) -> CopySpec {
    CopySpec {
        sandbox: "qwen".into(),
        remote: remote.into(),
        local: local.to_path_buf(),
        direction,
    }
}

/// `task` with a limit, so a copy, or a fake sandbox waiting on one, that
/// never ends fails its test instead of hanging the suite.
pub(super) async fn within<T>(task: impl Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(30), task)
        .await
        .expect("the copy or the fake sandbox did not end within 30 s")
}

/// A copy of `qwen:<remote>` to `local` from a sandbox sending `archive`,
/// then `stderr`, then `code`.
pub(super) async fn copy_out_of(
    remote: &str,
    local: &Path,
    archive: Vec<u8>,
    stderr: String,
    code: i32,
) -> CopyReport<FakeErr> {
    let plan = plan_copy_out(&spec(remote, local, Direction::Out)).unwrap();
    let (sink, source, mut fake) = call();
    let sandbox = tokio::spawn(async move {
        fake.argv().await;
        fake.data(Stream::Stdout, &archive).await;
        fake.data(Stream::Stderr, stderr.as_bytes()).await;
        fake.exit(code).await;
        fake
    });
    let r = within(copy_out(plan, sink, source, Arc::new(Progress::default()))).await;
    let _ = within(sandbox).await;
    r
}

pub(super) fn rejected<E: std::fmt::Debug>(r: &CopyReport<E>) -> &str {
    match &r.result {
        Err(CopyError::Rejected(m)) => m,
        other => panic!("want a rejected copy, got {other:?}"),
    }
}
