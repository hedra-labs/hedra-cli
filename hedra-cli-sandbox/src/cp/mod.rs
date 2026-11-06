//! `cp`: copies a file or a directory into or out of a sandbox as a tar
//! stream over an `Exec` call.
//!
//! The caller parses the arguments with [`parse_args`], plans the copy with
//! [`plan_copy_in`] or [`plan_copy_out`] before any request, opens and admits
//! an `Exec` call, and hands its two halves to [`copy_in`] or [`copy_out`],
//! which send `start` themselves.

mod capped;
mod extract;
mod left;
mod names;
mod send;
mod stderr;
#[cfg(test)]
mod test_support;
#[cfg(all(test, unix))]
mod tests;
#[cfg(all(test, windows))]
mod windows_tests;

use std::ffi::{OsStr, OsString};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use tokio::sync::mpsc;

pub use left::Left;

use crate::pb::hedra::sandbox::v1::{
    exec_client_frame, exec_server_frame, ExecClientFrame, ExecServerFrame, ExecStart, Stream,
};

use crate::stream::calls::CallClosed;
use crate::text::escape_controls;

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

/// What the supervisor writes on stderr, before exit 2, when it cannot start
/// a command, such as `sh` in an image without one; the reason follows.
pub const COULD_NOT_START: &str = "[hedra] the command could not start:";

/// The line the copy-out script prints on stderr before GNU tar runs.
const TOTALS_ANNOUNCED: &str = "[hedra-cp] totals";

/// Extracts a tar stream from stdin. `$1` is the destination and `$2` its
/// parent, created when missing. An empty `CDPATH` keeps `cd` quiet and
/// relative to the working directory. It first prints `dir` when the destination
/// is an existing directory, which receives the source under its own name,
/// `new` when the source becomes the destination, or `notdir`, creating
/// nothing, when a destination ending in `/` is not a directory.
pub const UPLOAD_SCRIPT: &str = r#"CDPATH=
command -v tar >/dev/null 2>&1 || { echo "[hedra-cp] no tar" >&2; exit 127; }
if [ -d "$1" ]; then cd -- "$1" && echo dir; else case $1 in */) echo notdir; exit 0;; esac; mkdir -p -- "$2" && cd -- "$2" && echo new; fi && exec tar -xf -"#;

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

/// The entry a copy began last, which a failure names, whether the copy was
/// cancelled, and whether a copy out began or finished writing anything to
/// the destination.
#[derive(Debug, Default)]
pub struct Progress {
    entry: Mutex<Begun>,
    /// Set with `Begun::cancelled`, for the extractor's reads to check
    /// without taking the lock.
    stop: AtomicBool,
    /// A copy out's extractor was started, and whether it has ended, with
    /// the signal its end wakes.
    extracting: AtomicBool,
    extracted: AtomicBool,
    ended: tokio::sync::Notify,
}

#[derive(Debug, Default)]
struct Begun {
    name: String,
    cancelled: bool,
    /// A write was let through, and may have landed.
    writing: bool,
    /// A write left something in the destination.
    wrote: bool,
}

impl Progress {
    fn lock(&self) -> std::sync::MutexGuard<'_, Begun> {
        self.entry.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The entry begun last, escaped; empty before the first.
    pub fn entry(&self) -> String {
        names::show(&self.lock().name)
    }

    /// Stops a copy out beginning another entry or writing anything more,
    /// and says whether it let a write through: when it did not, it has
    /// written nothing and never will. A write it let through may still be
    /// landing, or may have failed.
    pub fn cancel(&self) -> bool {
        let mut e = self.lock();
        e.cancelled = true;
        self.stop.store(true, Ordering::SeqCst);
        e.writing
    }

    /// Waits for a copy out's extractor, once started, to end, as it does
    /// soon after [`Progress::cancel`] once the copy is dropped, having put
    /// back the modes of directories it opened up.
    pub async fn extracted(&self) {
        loop {
            let notified = self.ended.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if !self.extracting.load(Ordering::SeqCst) || self.extracted.load(Ordering::SeqCst) {
                return;
            }
            notified.await;
        }
    }

    fn extraction_starts(&self) {
        self.extracting.store(true, Ordering::SeqCst);
    }

    fn extraction_ended(&self) {
        self.extracted.store(true, Ordering::SeqCst);
        self.ended.notify_waiters();
    }

    /// Whether the copy was cancelled.
    fn cancelled(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    /// Whether a write of a copy out left something in the destination.
    fn wrote(&self) -> bool {
        self.lock().wrote
    }

    /// Records that the copy begins `name`; false once it was cancelled.
    fn began(&self, name: &str) -> bool {
        let mut e = self.lock();
        if e.cancelled {
            return false;
        }
        name.clone_into(&mut e.name);
        true
    }

    /// Records that a copy out is about to change the destination; false
    /// once it was cancelled, when it must not.
    fn writes(&self) -> bool {
        let mut e = self.lock();
        if e.cancelled {
            return false;
        }
        e.writing = true;
        true
    }

    /// Records that a write of a copy out left something in the destination.
    fn landed(&self) {
        self.lock().wrote = true;
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
/// Anything else is a local path, which need not be UTF-8; a sandbox path
/// must be, since the command that copies it is.
fn remote_arg(
    arg: &OsStr,
    is_sandbox: &dyn Fn(&str) -> bool,
) -> Result<Option<(String, String)>, String> {
    let text = arg.to_string_lossy();
    let Some((name, path)) = text.split_once(':') else {
        return Ok(None);
    };
    if cfg!(windows) && name.len() == 1 && name.as_bytes()[0].is_ascii_alphabetic() {
        return Ok(None);
    }
    if !is_sandbox(name) {
        return Ok(None);
    }
    if arg.to_str().is_none() {
        return Err(format!(
            "{}:{} is not UTF-8; name the sandbox's path in UTF-8",
            escape_controls(name),
            escape_controls(path)
        ));
    }
    Ok(Some((name.to_string(), path.to_string())))
}

/// Parses `cp <src> <dst>`, exactly one side of which names a sandbox.
pub fn parse_args(
    src: impl AsRef<OsStr>,
    dst: impl AsRef<OsStr>,
    is_sandbox: impl Fn(&str) -> bool,
) -> Result<CopySpec, String> {
    let (src, dst) = (src.as_ref(), dst.as_ref());
    match (remote_arg(src, &is_sandbox)?, remote_arg(dst, &is_sandbox)?) {
        (Some(_), Some(_)) => {
            Err("both sides name a sandbox; copy between a sandbox and a local path".into())
        }
        (None, None) => Err("neither side names a sandbox; one side is <sandbox>:<path>".into()),
        (Some((sandbox, remote)), None) => Ok(CopySpec {
            sandbox,
            remote,
            local: PathBuf::from(dst),
            direction: Direction::Out,
        }),
        (None, Some((sandbox, remote))) => Ok(CopySpec {
            sandbox,
            remote,
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
    /// The top entry's name, the local file's own unless the sandbox side
    /// names a new one; as the local file system gives it, UTF-8 or not.
    name: OsString,
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
    /// Whether the target was a directory before the first
    /// [`CopyOut::prepare`], which may create it; unknown before that.
    existed_dir: Option<bool>,
}

impl CopyOut {
    /// Where the copy lands: the source's new name in the receiving
    /// directory, or the directory itself for a source's contents.
    pub fn target(&self) -> PathBuf {
        if self.name == "." {
            self.dir.clone()
        } else {
            self.dir.join(&self.name)
        }
    }

    /// Creates the receiving directory and its missing parents, and checks
    /// it can be written, so a destination that cannot take the copy fails
    /// it before the call opens.
    pub fn prepare(&mut self) -> Result<(), String> {
        if self.existed_dir.is_none() {
            let m = std::fs::symlink_metadata(self.target());
            self.existed_dir = Some(m.is_ok_and(|m| m.is_dir()));
        }
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

    /// What a copy out that wrote something and failed left at its target:
    /// part of the copy in a directory that was already there, or a partial
    /// copy in one this copy made or a file it replaced.
    pub fn left(&self) -> Left {
        let path = escape_controls(&self.target().display().to_string());
        match self.existed_dir {
            Some(true) => Left::Into(path),
            _ => Left::Partial(path),
        }
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

/// Whether `p`, as given, ends in a `.` component (`src/.`, `./`).
fn ends_in_dot(p: &Path) -> bool {
    let s = p.as_os_str().to_string_lossy();
    let s = s.trim_end_matches(|c| c == '/' || (cfg!(windows) && c == '\\'));
    s == "." || s.ends_with("/.") || (cfg!(windows) && s.ends_with("\\."))
}

/// Plans a copy in: resolves a symlinked source, checks it is a file or a
/// directory, and builds the command.
pub fn plan_copy_in(spec: &CopySpec) -> Result<CopyIn, String> {
    let shown = spec.local.display();
    let abs = std::path::absolute(&spec.local).map_err(|e| format!("cannot read {shown}: {e}"))?;
    let default_name = abs.components().next_back().and_then(|c| match c {
        std::path::Component::Normal(n) => Some(n.to_os_string()),
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
        OsString::from(".")
    } else {
        match default_name {
            Some(n) => n,
            None => src
                .file_name()
                .map(OsStr::to_os_string)
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
    if base == "/" {
        return Err("copy what is under the sandbox's root, not the root itself".into());
    }
    if ends_in_dot_dot || base == ".." {
        return Err(format!(
            "{:?} ends in ..; name the directory itself",
            escape_controls(&spec.remote)
        ));
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
        existed_dir: None,
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
        .ok_or_else(|| format!("{shown} names no file or directory"))?;
    // The copy's entries are named under it, and are UTF-8.
    let name = name
        .to_str()
        .ok_or_else(|| {
            format!("{shown} is not UTF-8; copy into a directory, or name the copy in UTF-8")
        })?
        .to_string();
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
    /// a tar entry, an image that cannot run the copy, a short `--totals`,
    /// the loss marker, or a copy out whose `tar` failed. The message is
    /// escaped and names what failed.
    Rejected(String),
    /// The call failed; the caller classes the error.
    Call(E),
    /// The call ended without an exit code.
    Ended,
}

/// What a copy did.
#[derive(Debug)]
pub struct CopyReport<E> {
    /// 0 for a whole copy, the remote `tar`'s code for a copy in it failed,
    /// or why the copy failed.
    pub result: Result<i32, CopyError<E>>,
    /// Lines to print on stderr, escaped.
    pub warnings: Vec<String>,
    /// The remote command's stderr less the copy's bookkeeping, up to 64 KiB.
    pub remote_stderr: Vec<u8>,
    /// The entry the copy began last, escaped, when it stopped part way.
    pub stopped_at: Option<String>,
    /// Where a failed copy out left what it wrote; it is not removed, as
    /// `scp` leaves its own.
    pub partial: Option<Left>,
}

fn start(argv: &[String]) -> ExecClientFrame {
    ExecClientFrame {
        frame: Some(exec_client_frame::Frame::Start(ExecStart {
            argv: argv.to_vec(),
            ..Default::default()
        })),
    }
}

/// Why an exit says the image cannot run the copy, if it does: the no-`tar`
/// marker or 127 from `sh` finding no `tar`; or 2 with the supervisor's line
/// that the command could not start, or with no output at all from a
/// supervisor that predates that line, which is `sh` failing to start.
fn unrunnable(code: i32, err: &stderr::CopyStderr, output: bool) -> Option<String> {
    if err.no_tar || code == 127 {
        return Some(NO_TAR.into());
    }
    if code != 2 {
        return None;
    }
    match &err.could_not_start {
        Some(why) => Some(format!(
            "the sandbox could not start `sh` ({}); `cp` needs `sh` and `tar` in the image",
            escape_controls(why)
        )),
        None if !output => {
            Some("the sandbox could not start `sh`; `cp` needs `sh` and `tar` in the image".into())
        }
        None => None,
    }
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
        partial: None,
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
                let msg = if let Some(m) = unrunnable(code, &err, !line.is_empty() || err.any) {
                    m
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
    match String::from_utf8_lossy(&line).trim() {
        "new" => name = names::base(&names::clean(&plan.remote)).into(),
        // The script exits having created nothing.
        "notdir" => {
            let msg = format!(
                "nothing was copied: {}:{} is not a directory in the sandbox",
                escape_controls(&plan.sandbox),
                escape_controls(&plan.remote)
            );
            return report(Err(CopyError::Rejected(msg)), vec![], err, &progress);
        }
        _ => {}
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
    if let Some(m) = unrunnable(code, &err, true) {
        return report(Err(CopyError::Rejected(m)), vec![], err, &progress);
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
/// first; the call must be admitted. A copy that fails before writing
/// anything removes the parents it created; one that wrote something leaves
/// it in place and names it in [`CopyReport::partial`], unless every entry
/// tar sent landed and only `tar`'s exit 1 failed the copy.
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
    let (mut r, whole) = copy_out_prepared(&plan, sink, source, progress.clone()).await;
    if matches!(r.result, Ok(0)) || whole {
        return r;
    }
    // The extractor has ended, so what it wrote is settled.
    if progress.wrote() && std::fs::symlink_metadata(plan.target()).is_ok() {
        r.partial = Some(plan.left());
    } else {
        plan.discard();
    }
    r
}

/// [`copy_out`] once the destination is prepared, and whether the copy
/// failed though every entry tar sent landed.
async fn copy_out_prepared<S: ExecSink, R: ExecSource>(
    plan: &CopyOut,
    mut sink: S,
    mut source: R,
    progress: Arc<Progress>,
) -> (CopyReport<R::Error>, bool) {
    let mut err = stderr::CopyStderr::new();
    let _ = sink.send(start(&plan.argv)).await;

    let (tx, rx) = mpsc::channel::<extract::Down>(DOWN_CHUNKS);
    let extractor = {
        let progress = progress.clone();
        let (dir, base, name) = (plan.dir.clone(), plan.base.clone(), plan.name.clone());
        progress.extraction_starts();
        tokio::task::spawn_blocking(move || {
            let x = extract::extract(rx, &dir, &base, &name, &progress);
            progress.extraction_ended();
            x
        })
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
        let r = report(Err(CopyError::Rejected(msg)), warnings, err, &progress);
        return (r, false);
    }
    let code = match ending {
        Ending::Call(e) => {
            let result = match ex.error {
                Some(m) if !ex.cut => Err(CopyError::Rejected(m)),
                _ => Err(CopyError::Call(e)),
            };
            return (report(result, warnings, err, &progress), false);
        }
        Ending::Ended => {
            return (
                report(Err(CopyError::Ended), warnings, err, &progress),
                false,
            );
        }
        Ending::Exit(c) => c,
    };
    // Every entry tar sent landed and GNU tar's totals match what arrived.
    // A `tar` exiting 1 (a file changed, shrank or went away as it read it)
    // still fails the copy, but the line says every entry arrived and names
    // no partial copy.
    let whole = ex.error.is_none()
        && ex.skipped.is_empty()
        && ex.whole
        && !err.dropped
        && err.gnu
        && err.totals == Some(received);
    let fail = |msg: String| Err(CopyError::Rejected(msg));
    // A copy out whose `tar` failed is hedra-cli's failure, whatever the
    // code: only a copy in exits with the remote `tar`'s.
    let unrun = unrunnable(code, &err, received > 0 || err.any);
    let whole = whole && code == 1 && unrun.is_none();
    let result = if let Some(m) = unrun {
        fail(m)
    } else if code != 0 {
        match ex.error {
            Some(e) => fail(format!("{e}; tar in the sandbox exited {code}")),
            None if !ex.skipped.is_empty() => fail(format!(
                "the copy left out {}; tar in the sandbox exited {code}",
                ex.skipped.join(", ")
            )),
            None if whole => fail(format!(
                "tar in the sandbox exited 1 (its message is above); every entry it sent arrived at {}",
                escape_controls(&plan.target().display().to_string())
            )),
            None => fail(format!("tar in the sandbox exited {code}")),
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
    (report(result, warnings, err, &progress), whole)
}
