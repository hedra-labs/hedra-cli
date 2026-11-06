//! `cp`: copies a file or a directory into or out of a sandbox as a tar
//! stream over an `Exec` call.
//!
//! The caller parses the arguments with [`parse_args`], plans the copy with
//! [`plan_copy_in`] or [`plan_copy_out`] before any request, opens and admits
//! an `Exec` call, and hands its two halves to [`copy_in`] or [`copy_out`],
//! which send `start` themselves.

mod capped;
mod extract;
mod names;
mod send;
mod stderr;
#[cfg(test)]
mod test_support;
#[cfg(all(test, unix))]
mod tests;
#[cfg(all(test, windows))]
mod windows_tests;

use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use tokio::sync::mpsc;

use crate::pb::hedra::sandbox::v1::{
    exec_client_frame, exec_server_frame, ExecClientFrame, ExecServerFrame, ExecStart, Stream,
};

pub(crate) use names::escape_controls;

/// The most stdin one frame carries.
const CHUNK: usize = 32 << 10;

/// GNU tar's default record; a stream sent to `tar -x` is padded to whole ones.
const TAR_RECORD: u64 = 20 * 512;

/// The longest line the copy-in script prints before `tar` runs.
const PROBE_MAX: usize = 4 << 10;

/// Chunks of a copy out held between the call and the extractor (8 MiB of
/// 32 KiB frames).
const DOWN_CHUNKS: usize = 256;

/// The text in the notice a supervisor writes into an exec's stderr when it
/// dropped output.
pub const LOSS_MARKER: &str = "dropped: the broker stopped reading this sandbox's stream";

/// The line both scripts print on stderr when the image has no `tar`.
pub const NO_TAR_MARKER: &str = "[hedra-cp] no tar";

/// The line the copy-out script prints on stderr before GNU tar runs.
const TOTALS_ANNOUNCED: &str = "[hedra-cp] totals";

/// Extracts a tar stream from stdin. `$1` is the destination and `$2` its
/// parent, created when missing. An empty `CDPATH` keeps `cd` quiet and
/// relative to the working directory. It first prints `dir` when the destination
/// is an existing directory, which receives the source under its own name,
/// or `new` when the source becomes the destination.
pub const UPLOAD_SCRIPT: &str = r#"CDPATH=
command -v tar >/dev/null 2>&1 || { echo "[hedra-cp] no tar" >&2; exit 127; }
if [ -d "$1" ]; then cd -- "$1" && echo dir; else mkdir -p -- "$2" && cd -- "$2" && echo new; fi && exec tar -xf -"#;

/// Writes `$2`, in the directory `$1`, as a tar stream on stdout. A
/// symlinked `$2` is followed. GNU tar reports the bytes it wrote with
/// `--totals`; the script announces GNU tar on stderr first, since busybox
/// tar has no `--totals`. `-b 64` writes 32 KiB records, one frame each.
pub const DOWNLOAD_SCRIPT: &str = r#"export LC_ALL=C
command -v tar >/dev/null 2>&1 || { echo "[hedra-cp] no tar" >&2; exit 127; }
h=
if [ -L "$1/$2" ]; then if [ -d "$1/$2" ]; then set -- "$1" "$2/."; else h=-h; fi; fi
case "$(tar --version 2>/dev/null)" in
*GNU*) echo "[hedra-cp] totals" >&2; exec tar -cf - -b 64 --totals $h -C "$1" -- "$2" ;;
*) exec tar -cf - $h -C "$1" -- "$2" ;;
esac"#;

/// The sending half of an admitted `Exec` call.
pub trait ExecSink: Send + 'static {
    /// Sends one frame; an error means the call ended, and the source says why.
    fn send(
        &mut self,
        frame: ExecClientFrame,
    ) -> impl Future<Output = Result<(), CallClosed>> + Send;
}

/// The receiving half of an admitted `Exec` call. A copy cancels the call by
/// dropping the source, then the sink; dropping the sink resets the call
/// rather than ending stdin.
pub trait ExecSource: Send + 'static {
    /// The call's failure, which the caller classes.
    type Error: Send + 'static;
    /// The next frame, or `None` when the call ended without a status.
    fn recv(&mut self)
        -> impl Future<Output = Result<Option<ExecServerFrame>, Self::Error>> + Send;
}

/// A send on a call that has already ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallClosed;

impl ExecSink for mpsc::Sender<ExecClientFrame> {
    async fn send(&mut self, frame: ExecClientFrame) -> Result<(), CallClosed> {
        mpsc::Sender::send(self, frame)
            .await
            .map_err(|_| CallClosed)
    }
}

impl ExecSink for crate::stream::calls::Sender<ExecClientFrame> {
    async fn send(&mut self, frame: ExecClientFrame) -> Result<(), CallClosed> {
        mpsc::Sender::send(self, frame)
            .await
            .map_err(|_| CallClosed)
    }
}

impl ExecSource for tonic::Streaming<ExecServerFrame> {
    type Error = tonic::Status;
    async fn recv(&mut self) -> Result<Option<ExecServerFrame>, tonic::Status> {
        self.message().await
    }
}

/// How far a copy has got, for a progress line.
#[derive(Debug, Default)]
pub struct Progress {
    bytes: AtomicU64,
    entry: Mutex<String>,
}

impl Progress {
    /// Bytes moved so far.
    pub fn bytes(&self) -> u64 {
        self.bytes.load(Ordering::Relaxed)
    }

    /// The entry begun last, escaped; empty before the first.
    pub fn entry(&self) -> String {
        self.entry
            .lock()
            .map(|e| names::show(&e))
            .unwrap_or_default()
    }

    fn add(&self, n: u64) {
        self.bytes.fetch_add(n, Ordering::Relaxed);
    }

    fn began(&self, name: &str) {
        if let Ok(mut e) = self.entry.lock() {
            name.clone_into(&mut e);
        }
    }
}

/// Which way a copy goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// From the laptop into the sandbox.
    In,
    /// From the sandbox to the laptop.
    Out,
}

/// A parsed `cp <src> <dst>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopySpec {
    /// The sandbox name or id, as given.
    pub sandbox: String,
    /// The path in the sandbox.
    pub remote: String,
    /// The local path.
    pub local: PathBuf,
    /// Which way the copy goes.
    pub direction: Direction,
}

/// Splits `<sandbox>:<path>`: a sandbox path when the part before the first
/// `:` passes `is_sandbox`; on Windows a single letter there is a drive.
fn remote_arg<'a>(arg: &'a str, is_sandbox: &dyn Fn(&str) -> bool) -> Option<(&'a str, &'a str)> {
    let (name, path) = arg.split_once(':')?;
    if cfg!(windows) && name.len() == 1 && name.as_bytes()[0].is_ascii_alphabetic() {
        return None;
    }
    is_sandbox(name).then_some((name, path))
}

/// Parses `cp <src> <dst>`, exactly one side of which names a sandbox.
pub fn parse_args(
    src: &str,
    dst: &str,
    is_sandbox: impl Fn(&str) -> bool,
) -> Result<CopySpec, String> {
    match (remote_arg(src, &is_sandbox), remote_arg(dst, &is_sandbox)) {
        (Some(_), Some(_)) => {
            Err("both sides name a sandbox; copy between a sandbox and a local path".into())
        }
        (None, None) => Err("neither side names a sandbox; one side is <sandbox>:<path>".into()),
        (Some((name, path)), None) => Ok(CopySpec {
            sandbox: name.into(),
            remote: path.into(),
            local: PathBuf::from(dst),
            direction: Direction::Out,
        }),
        (None, Some((name, path))) => Ok(CopySpec {
            sandbox: name.into(),
            remote: path.into(),
            local: PathBuf::from(src),
            direction: Direction::In,
        }),
    }
}

/// A copy in, planned before any request.
#[derive(Debug, Clone)]
pub struct CopyIn {
    sandbox: String,
    remote: String,
    src: PathBuf,
    name: String,
    argv: Vec<String>,
}

/// A copy out, planned before any request.
#[derive(Debug, Clone)]
pub struct CopyOut {
    dir: PathBuf,
    base: String,
    name: String,
    argv: Vec<String>,
    /// Directories [`CopyOut::prepare`] created, deepest first.
    created: Vec<PathBuf>,
}

impl CopyOut {
    /// Creates the receiving directory and its missing parents, and checks
    /// it can be written, so a destination that cannot take the copy fails
    /// it before the call opens.
    pub fn prepare(&mut self) -> Result<(), String> {
        let shown = self.dir.display().to_string();
        let mut missing = Vec::new();
        let mut at = self.dir.as_path();
        loop {
            match std::fs::symlink_metadata(at) {
                Ok(_) => break,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    missing.push(at.to_path_buf());
                    match at.parent() {
                        Some(p) if !p.as_os_str().is_empty() => at = p,
                        _ => break,
                    }
                }
                Err(e) => return Err(format!("cannot create {shown}: {e}")),
            }
        }
        if let Err(e) = std::fs::create_dir_all(&self.dir) {
            self.created = missing;
            self.discard();
            return Err(format!("cannot create {shown}: {e}"));
        }
        self.created.extend(missing);
        let writable = std::fs::metadata(&self.dir)
            .map_err(|e| e.to_string())
            .and_then(|m| {
                if !m.is_dir() {
                    return Err("it is not a directory".to_string());
                }
                writable(&self.dir).map_err(|e| e.to_string())
            });
        if let Err(e) = writable {
            self.discard();
            return Err(format!("cannot write to {shown}: {e}"));
        }
        Ok(())
    }

    /// Removes the directories [`CopyOut::prepare`] created that are still
    /// empty.
    pub fn discard(&mut self) {
        for d in self.created.drain(..) {
            let _ = std::fs::remove_dir(d);
        }
    }
}

#[cfg(unix)]
fn writable(dir: &Path) -> std::io::Result<()> {
    use rustix::fs::{access, Access};
    access(dir, Access::WRITE_OK | Access::EXEC_OK).map_err(std::io::Error::from)
}

#[cfg(not(unix))]
fn writable(_: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Whether `p`, as given, ends in a `.` component (`src/.`).
fn ends_in_dot(p: &Path) -> bool {
    let s = p.as_os_str().to_string_lossy();
    s == "." || s.ends_with("/.") || (cfg!(windows) && s.ends_with("\\."))
}

/// Plans a copy in: resolves a symlinked source, checks it is a file or a
/// directory, and builds the command.
pub fn plan_copy_in(spec: &CopySpec) -> Result<CopyIn, String> {
    let shown = spec.local.display();
    let abs = std::path::absolute(&spec.local).map_err(|e| format!("cannot read {shown}: {e}"))?;
    let default_name = abs.components().next_back().and_then(|c| match c {
        std::path::Component::Normal(n) => Some(n.to_string_lossy().into_owned()),
        _ => None,
    });
    let src = std::fs::canonicalize(&abs).map_err(|e| format!("cannot read {shown}: {e}"))?;
    if src.parent().is_none() {
        return Err("copy what is under the root, not the root itself".into());
    }
    let meta = std::fs::metadata(&src).map_err(|e| format!("cannot read {shown}: {e}"))?;
    if !meta.is_file() && !meta.is_dir() {
        return Err(format!("{shown} is not a file or a directory"));
    }
    let name = if ends_in_dot(&spec.local) {
        ".".to_string()
    } else {
        match default_name {
            Some(n) => n,
            None => src
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .ok_or_else(|| format!("{shown} names no file or directory"))?,
        }
    };
    let remote = if spec.remote.is_empty() {
        ".".to_string()
    } else {
        spec.remote.clone()
    };
    let argv = vec![
        "sh".into(),
        "-c".into(),
        UPLOAD_SCRIPT.into(),
        "cp".into(),
        remote.clone(),
        names::dir(&names::clean(&remote)),
    ];
    Ok(CopyIn {
        sandbox: spec.sandbox.clone(),
        remote,
        src,
        name,
        argv,
    })
}

/// Plans a copy out: where it lands locally, and the command.
pub fn plan_copy_out(spec: &CopySpec) -> Result<CopyOut, String> {
    let remote = if spec.remote.is_empty() {
        ".".to_string()
    } else {
        names::clean(&spec.remote)
    };
    let (mut parent, mut base) = (names::dir(&remote), names::base(&remote));
    // `a/..` cleans to `.`, which would copy the whole working directory.
    let ends_in_dot_dot = names::base(&spec.remote) == "..";
    match base.as_str() {
        "/" => return Err("copy what is under the sandbox's root, not the root itself".into()),
        _ if ends_in_dot_dot => {
            return Err(format!(
                "{:?} ends in ..; name the directory itself",
                escape_controls(&spec.remote)
            ))
        }
        ".." => {
            return Err(format!(
                "{:?} ends in ..; name the directory itself",
                escape_controls(&spec.remote)
            ))
        }
        _ => {}
    }
    // A source ending in `/.` is the directory's contents.
    if base != "." && names::base(&spec.remote) == "." {
        parent = remote.clone();
        base = ".".into();
    }
    let (dir, name) = local_target(&spec.local, &base)?;
    let argv = vec![
        "sh".into(),
        "-c".into(),
        DOWNLOAD_SCRIPT.into(),
        "cp".into(),
        parent,
        base.clone(),
    ];
    Ok(CopyOut {
        dir,
        base,
        name,
        argv,
        created: Vec::new(),
    })
}

/// Where a copy out lands: an existing directory receives the source under
/// its own name; anything else is the new name, in a parent created when
/// missing.
fn local_target(local: &Path, base: &str) -> Result<(PathBuf, String), String> {
    let shown = local.display();
    match std::fs::metadata(local) {
        Ok(m) if m.is_dir() => return Ok((local.to_path_buf(), base.to_string())),
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(format!("{shown}: {e}")),
        _ => {}
    }
    let s = local.as_os_str().to_string_lossy();
    if s.ends_with('/') || (cfg!(windows) && s.ends_with('\\')) {
        return Err(format!("{shown} is not a directory"));
    }
    let name = local
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| format!("{shown} names no file or directory"))?;
    let parent = match local.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };
    Ok((parent, name))
}

/// How a copy failed.
#[derive(Debug)]
pub enum CopyError<E> {
    /// hedra-cli refused or could not finish the copy itself: a local file,
    /// a tar entry, no `tar`, a short `--totals` or the loss marker. The
    /// message is escaped and names what failed.
    Rejected(String),
    /// The call failed; the caller classes the error.
    Call(E),
    /// The call ended without an exit code.
    Ended,
}

/// What a copy did.
#[derive(Debug)]
pub struct CopyReport<E> {
    /// The remote command's exit code, 0 for a whole copy, or why it failed.
    pub result: Result<i32, CopyError<E>>,
    /// Lines to print on stderr, escaped.
    pub warnings: Vec<String>,
    /// The remote command's stderr less the copy's bookkeeping, up to 64 KiB.
    pub remote_stderr: Vec<u8>,
    /// The entry the copy began last, escaped, when it stopped part way.
    pub stopped_at: Option<String>,
}

fn start(argv: &[String]) -> ExecClientFrame {
    ExecClientFrame {
        frame: Some(exec_client_frame::Frame::Start(ExecStart {
            argv: argv.to_vec(),
            ..Default::default()
        })),
    }
}

/// Whether an exit says the image cannot run the copy: the marker, 127 from
/// `sh` finding no `tar`, or 2 with no output at all (no `sh`).
fn no_tar(code: i32, marker: bool, output: bool) -> bool {
    marker || code == 127 || (code == 2 && !output)
}

const NO_TAR: &str = "the image has no `tar`; `cp` needs `sh` and `tar` in the image";

/// One frame of the copy's command.
enum Got {
    Stdout(Bytes),
    Stderr(Bytes),
    Exit(i32),
}

fn got(frame: ExecServerFrame) -> Option<Got> {
    match frame.frame? {
        exec_server_frame::Frame::Data(d) => {
            let chunk = Bytes::from(d.chunk);
            Some(if d.stream == Stream::Stderr as i32 {
                Got::Stderr(chunk)
            } else {
                Got::Stdout(chunk)
            })
        }
        exec_server_frame::Frame::ExitCode(c) => Some(Got::Exit(c)),
    }
}

/// How the receive loop ended.
enum Ending<E> {
    Exit(i32),
    Call(E),
    Ended,
}

fn report<E>(
    result: Result<i32, CopyError<E>>,
    mut warnings: Vec<String>,
    err: stderr::CopyStderr,
    progress: &Progress,
) -> CopyReport<E> {
    let stopped_at = match &result {
        Ok(0) => None,
        _ => Some(progress.entry()).filter(|e| !e.is_empty()),
    };
    warnings.shrink_to_fit();
    CopyReport {
        result,
        warnings,
        remote_stderr: err.kept,
        stopped_at,
    }
}

/// How the task sending the tar stream ended; it hands the sink back so the
/// copy can drop it after the source.
enum Sent<S> {
    Whole(S),
    BuilderFailed(S),
    CallClosed(S),
}

/// Copies a local file or directory into the sandbox, as the stdin of
/// `tar -x` there. Sends `start` first; the call must be admitted.
pub async fn copy_in<S: ExecSink, R: ExecSource>(
    plan: CopyIn,
    mut sink: S,
    mut source: R,
    progress: Arc<Progress>,
) -> CopyReport<R::Error> {
    let mut err = stderr::CopyStderr::new();
    let _ = sink.send(start(&plan.argv)).await;

    // The script's line first: `dir` or `new` decides the entries' top name.
    let mut line: Vec<u8> = Vec::new();
    loop {
        let frame = match source.recv().await {
            Ok(Some(f)) => f,
            Ok(None) => return report(Err(CopyError::Ended), vec![], err, &progress),
            Err(e) => return report(Err(CopyError::Call(e)), vec![], err, &progress),
        };
        match got(frame) {
            Some(Got::Stderr(b)) => err.write(&b),
            Some(Got::Stdout(b)) => {
                let (chunk, done) = match b.iter().position(|c| *c == b'\n') {
                    Some(i) => (&b[..i], true),
                    None => (&b[..], false),
                };
                line.extend_from_slice(chunk);
                if line.len() > PROBE_MAX {
                    let msg = "nothing was copied: the sandbox's sh printed more than the one line it was asked for";
                    drop(source);
                    drop(sink);
                    return report(Err(CopyError::Rejected(msg.into())), vec![], err, &progress);
                }
                if done {
                    break;
                }
            }
            Some(Got::Exit(code)) => {
                err.flush();
                let msg = if no_tar(code, err.no_tar, !line.is_empty() || err.any) {
                    NO_TAR.to_string()
                } else {
                    format!(
                        "nothing was copied: {}:{} is not a directory and could not be created in the sandbox (exit {code})",
                        escape_controls(&plan.sandbox),
                        escape_controls(&plan.remote)
                    )
                };
                return report(Err(CopyError::Rejected(msg)), vec![], err, &progress);
            }
            None => {}
        }
    }
    let mut name = plan.name.clone();
    if String::from_utf8_lossy(&line).trim() == "new" {
        if plan.remote.ends_with('/') {
            let msg = format!(
                "nothing was copied: {}:{} is not a directory in the sandbox",
                escape_controls(&plan.sandbox),
                escape_controls(&plan.remote)
            );
            // tar there is waiting on stdin; cancel rather than end it.
            drop(source);
            drop(sink);
            return report(Err(CopyError::Rejected(msg)), vec![], err, &progress);
        }
        name = names::base(&names::clean(&plan.remote));
    }

    let (tx, mut rx) = mpsc::channel::<send::Up>(8);
    let src = plan.src.clone();
    let builder = {
        let progress = progress.clone();
        tokio::task::spawn_blocking(move || send::send_tar(&src, &name, tx, progress))
    };
    let sender = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Some(send::Up::Data(b)) => {
                    let frame = ExecClientFrame {
                        frame: Some(exec_client_frame::Frame::Stdin(b.to_vec())),
                    };
                    if sink.send(frame).await.is_err() {
                        return Sent::CallClosed(sink);
                    }
                }
                Some(send::Up::Done) => {
                    let eof = ExecClientFrame {
                        frame: Some(exec_client_frame::Frame::StdinEof(true)),
                    };
                    return match sink.send(eof).await {
                        Ok(()) => Sent::Whole(sink),
                        Err(_) => Sent::CallClosed(sink),
                    };
                }
                None => return Sent::BuilderFailed(sink),
            }
        }
    });
    // Dropping the copy drops the sink with it, which resets the call.
    let mut sender = AbortOnDrop(sender);

    let mut sent: Option<Sent<S>> = None;
    let ending = loop {
        tokio::select! {
            res = &mut sender.0, if sent.is_none() => {
                match res {
                    Ok(Sent::BuilderFailed(sink)) => {
                        let msg = match builder.await {
                            Ok(Err(m)) => m,
                            Ok(Ok(_)) => "the tar stream ended early".to_string(),
                            Err(e) => std::panic::resume_unwind(e.into_panic()),
                        };
                        // A local failure cancels the call, so tar in the
                        // sandbox never sees a clean end.
                        drop(source);
                        drop(sink);
                        return report(Err(CopyError::Rejected(msg)), vec![], err, &progress);
                    }
                    Ok(s) => sent = Some(s),
                    Err(e) => std::panic::resume_unwind(e.into_panic()),
                }
            }
            frame = source.recv() => match frame {
                Ok(Some(f)) => match got(f) {
                    Some(Got::Stderr(b)) => err.write(&b),
                    Some(Got::Exit(c)) => break Ending::Exit(c),
                    _ => {}
                },
                Ok(None) => break Ending::Ended,
                Err(e) => break Ending::Call(e),
            }
        }
    };
    err.flush();
    // Once tar has exited, at most padding is left to send.
    drop(source);
    drop(sent);
    drop(sender);

    let code = match ending {
        Ending::Call(e) => return report(Err(CopyError::Call(e)), vec![], err, &progress),
        Ending::Ended => return report(Err(CopyError::Ended), vec![], err, &progress),
        Ending::Exit(c) => c,
    };
    if no_tar(code, err.no_tar, true) {
        return report(
            Err(CopyError::Rejected(NO_TAR.into())),
            vec![],
            err,
            &progress,
        );
    }
    let walked = match builder.await {
        Ok(Ok(w)) => w,
        // tar stopped reading: its code says why.
        Ok(Err(_)) if code != 0 => return report(Ok(code), vec![], err, &progress),
        Ok(Err(m)) => return report(Err(CopyError::Rejected(m)), vec![], err, &progress),
        Err(e) => std::panic::resume_unwind(e.into_panic()),
    };
    if !walked.skipped.is_empty() {
        let mut msg = format!("the copy left out {}", walked.skipped.join(", "));
        if code != 0 {
            msg = format!("{msg}; tar in the sandbox exited {code}");
        }
        return report(
            Err(CopyError::Rejected(msg)),
            walked.warnings,
            err,
            &progress,
        );
    }
    report(Ok(code), walked.warnings, err, &progress)
}

/// Aborts a task when dropped.
struct AbortOnDrop<T>(tokio::task::JoinHandle<T>);

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Copies a file or a directory out of the sandbox, from the stdout of
/// `tar -c` there, and holds it to what GNU tar says it wrote. Sends `start`
/// first; the call must be admitted.
pub async fn copy_out<S: ExecSink, R: ExecSource>(
    mut plan: CopyOut,
    sink: S,
    source: R,
    progress: Arc<Progress>,
) -> CopyReport<R::Error> {
    if let Err(m) = plan.prepare() {
        let err = stderr::CopyStderr::new();
        return report(Err(CopyError::Rejected(m)), vec![], err, &progress);
    }
    let r = copy_out_prepared(&plan, sink, source, progress).await;
    if !matches!(r.result, Ok(0)) {
        plan.discard();
    }
    r
}

async fn copy_out_prepared<S: ExecSink, R: ExecSource>(
    plan: &CopyOut,
    mut sink: S,
    mut source: R,
    progress: Arc<Progress>,
) -> CopyReport<R::Error> {
    let mut err = stderr::CopyStderr::new();
    let _ = sink.send(start(&plan.argv)).await;

    let (tx, rx) = mpsc::channel::<extract::Down>(DOWN_CHUNKS);
    let extractor = {
        let progress = progress.clone();
        let (dir, base, name) = (plan.dir.clone(), plan.base.clone(), plan.name.clone());
        tokio::task::spawn_blocking(move || extract::extract(rx, &dir, &base, &name, &progress))
    };
    let mut received: u64 = 0;
    let mut extractor_stopped = false;
    let ending = loop {
        let frame = match source.recv().await {
            Ok(Some(f)) => f,
            Ok(None) => break Ending::Ended,
            Err(e) => break Ending::Call(e),
        };
        match got(frame) {
            Some(Got::Stdout(b)) => {
                received += b.len() as u64;
                progress.add(b.len() as u64);
                if tx.send(extract::Down::Data(b)).await.is_err() {
                    extractor_stopped = true;
                    break Ending::Ended;
                }
            }
            Some(Got::Stderr(b)) => err.write(&b),
            Some(Got::Exit(c)) => {
                let _ = tx.send(extract::Down::End).await;
                break Ending::Exit(c);
            }
            None => {}
        }
    };
    drop(tx);
    // The extractor stopping first cancels the call: nothing more of the
    // stream can be written.
    drop(source);
    drop(sink);
    let ex = match extractor.await {
        Ok(x) => x,
        Err(e) => std::panic::resume_unwind(e.into_panic()),
    };
    err.flush();
    let warnings = ex.warnings.clone();
    if extractor_stopped {
        let msg = ex.error.unwrap_or_else(|| "the copy stopped early".into());
        return report(Err(CopyError::Rejected(msg)), warnings, err, &progress);
    }
    let code = match ending {
        Ending::Call(e) => {
            if ex.error.is_some() && !ex.cut {
                return report(
                    Err(CopyError::Rejected(ex.error.unwrap_or_default())),
                    warnings,
                    err,
                    &progress,
                );
            }
            return report(Err(CopyError::Call(e)), warnings, err, &progress);
        }
        Ending::Ended => return report(Err(CopyError::Ended), warnings, err, &progress),
        Ending::Exit(c) => c,
    };
    let fail = |msg: String| Err(CopyError::Rejected(msg));
    let result = if no_tar(code, err.no_tar, received > 0 || err.any) {
        fail(NO_TAR.into())
    } else if code != 0 {
        match ex.error {
            Some(e) => fail(format!("{e}; tar in the sandbox exited {code}")),
            None if !ex.skipped.is_empty() => fail(format!(
                "the copy left out {}; tar in the sandbox exited {code}",
                ex.skipped.join(", ")
            )),
            None => Ok(code),
        }
    } else if err.dropped && !err.gnu {
        fail("the platform dropped part of the stream; the image's tar is not GNU tar, which reports its totals, so only the platform's notice caught it".into())
    } else if err.dropped {
        fail("the platform dropped part of the stream".into())
    } else if let Some(e) = ex.error {
        fail(e)
    } else if !ex.whole {
        fail("the stream ended before the archive did".into())
    } else if !ex.skipped.is_empty() {
        fail(format!("the copy left out {}", ex.skipped.join(", ")))
    } else if err.gnu && err.totals.is_none() {
        fail("tar in the sandbox did not report how many bytes it wrote".into())
    } else if let Some(t) = err.totals.filter(|t| err.gnu && *t != received) {
        fail(format!(
            "tar in the sandbox wrote {t} bytes and {received} arrived"
        ))
    } else {
        Ok(0)
    };
    report(result, warnings, err, &progress)
}
