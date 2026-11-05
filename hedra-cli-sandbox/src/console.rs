//! Windows consoles: raw input, UTF-8 output and the terminal check.

use std::io;

use windows_sys::Win32::Foundation::{
    DuplicateHandle, GetLastError, DUPLICATE_SAME_ACCESS, HANDLE, INVALID_HANDLE_VALUE,
    WAIT_TIMEOUT,
};
use windows_sys::Win32::Storage::FileSystem::WriteFile;
use windows_sys::Win32::System::Console::{
    GetConsoleMode, GetStdHandle, ReadConsoleW, SetConsoleMode, WriteConsoleW, CONSOLE_MODE,
    ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT, ENABLE_VIRTUAL_TERMINAL_INPUT,
    ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_ERROR_HANDLE, STD_HANDLE, STD_INPUT_HANDLE,
    STD_OUTPUT_HANDLE,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, WaitForSingleObject};

/// A standard handle a stream writes or reads.
#[derive(Clone, Copy)]
pub enum Std {
    In,
    Out,
    Err,
}

fn std_handle(which: Std) -> HANDLE {
    let id: STD_HANDLE = match which {
        Std::In => STD_INPUT_HANDLE,
        Std::Out => STD_OUTPUT_HANDLE,
        Std::Err => STD_ERROR_HANDLE,
    };
    // SAFETY: GetStdHandle has no preconditions.
    unsafe { GetStdHandle(id) }
}

fn mode(handle: HANDLE) -> Option<CONSOLE_MODE> {
    let mut mode: CONSOLE_MODE = 0;
    // SAFETY: `mode` is a valid out pointer.
    (unsafe { GetConsoleMode(handle, &mut mode) } != 0).then_some(mode)
}

/// Whether the handle is a console. An msys pipe under mintty is not, though
/// `IsTerminal` says it is.
pub fn is_console(which: Std) -> bool {
    mode(std_handle(which)).is_some()
}

/// A duplicate of a standard output handle, so writes never take std's lock.
pub struct Output {
    handle: usize,
    console: bool,
    /// An incomplete UTF-8 sequence carried to the next write.
    pending: Vec<u8>,
}

// SAFETY: the handle is owned by this value and used from one task at a time.
unsafe impl Send for Output {}

impl Output {
    pub fn dup(which: Std) -> io::Result<Self> {
        let source = std_handle(which);
        let mut target: HANDLE = std::ptr::null_mut();
        // SAFETY: duplicating a handle of this process into this process.
        let ok = unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                source,
                GetCurrentProcess(),
                &mut target,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        };
        if ok == 0 || target == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            handle: target as usize,
            console: mode(target).is_some(),
            pending: Vec::new(),
        })
    }

    /// Whether a failed write means the console is gone: a broken pipe or no
    /// data on a console handle, or the handle no longer a console.
    pub fn console_gone(&self, e: &io::Error) -> bool {
        // ERROR_BROKEN_PIPE, ERROR_NO_DATA, ERROR_PIPE_NOT_CONNECTED.
        self.console
            && (matches!(e.raw_os_error(), Some(109) | Some(232) | Some(233))
                || mode(self.handle as HANDLE).is_none())
    }

    /// Writes all of `bytes` on a blocking thread.
    pub async fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        let handle = self.handle;
        if self.console {
            let mut data = std::mem::take(&mut self.pending);
            data.extend_from_slice(bytes);
            let cut = complete_prefix(&data);
            self.pending = data.split_off(cut);
            let text: Vec<u16> = String::from_utf8_lossy(&data).encode_utf16().collect();
            tokio::task::spawn_blocking(move || write_console(handle as HANDLE, &text))
                .await
                .map_err(io::Error::other)?
        } else {
            let data = bytes.to_vec();
            tokio::task::spawn_blocking(move || write_file(handle as HANDLE, &data))
                .await
                .map_err(io::Error::other)?
        }
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value.
        unsafe { windows_sys::Win32::Foundation::CloseHandle(self.handle as HANDLE) };
    }
}

pub use crate::console_text::complete_prefix;

fn write_console(handle: HANDLE, text: &[u16]) -> io::Result<()> {
    let mut rest = text;
    while !rest.is_empty() {
        let mut written = 0u32;
        // SAFETY: `rest` is a valid buffer of the given length.
        let ok = unsafe {
            WriteConsoleW(
                handle,
                rest.as_ptr().cast(),
                rest.len().min(u32::MAX as usize) as u32,
                &mut written,
                std::ptr::null(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        rest = &rest[written as usize..];
    }
    Ok(())
}

fn write_file(handle: HANDLE, data: &[u8]) -> io::Result<()> {
    let mut rest = data;
    while !rest.is_empty() {
        let mut written = 0u32;
        // SAFETY: `rest` is a valid buffer of the given length.
        let ok = unsafe {
            WriteFile(
                handle,
                rest.as_ptr(),
                rest.len().min(u32::MAX as usize) as u32,
                &mut written,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        rest = &rest[written as usize..];
    }
    Ok(())
}

/// The console modes of stdin and stdout, set raw and written back verbatim
/// when dropped.
pub struct RawConsole {
    input: (usize, CONSOLE_MODE),
    output: (usize, CONSOLE_MODE),
}

impl RawConsole {
    pub fn enter() -> io::Result<Self> {
        let input = std_handle(Std::In);
        let output = std_handle(Std::Out);
        let (Some(in_mode), Some(out_mode)) = (mode(input), mode(output)) else {
            return Err(io::Error::other("stdin and stdout are not a console"));
        };
        let raw_in = (in_mode | ENABLE_VIRTUAL_TERMINAL_INPUT)
            & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
        // SAFETY: setting modes on this process's console handles.
        unsafe {
            if SetConsoleMode(input, raw_in) == 0
                || SetConsoleMode(output, out_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) == 0
            {
                SetConsoleMode(input, in_mode);
                return Err(io::Error::last_os_error());
            }
        }
        Ok(Self {
            input: (input as usize, in_mode),
            output: (output as usize, out_mode),
        })
    }
}

impl Drop for RawConsole {
    fn drop(&mut self) {
        // SAFETY: restoring the modes read in `enter`.
        unsafe {
            SetConsoleMode(self.input.0 as HANDLE, self.input.1);
            SetConsoleMode(self.output.0 as HANDLE, self.output.1);
        }
    }
}

/// Reads console input as UTF-8, with no input control, so Ctrl-Z arrives as
/// 0x1A rather than ending input. A surrogate pair split across reads is
/// carried over. Returns `Ok(0)` only when the console is gone.
pub struct ConsoleReader {
    carry: Option<u16>,
}

impl Default for ConsoleReader {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleReader {
    pub fn new() -> Self {
        Self { carry: None }
    }

    /// Waits up to `timeout` for console input, and says whether to read.
    pub fn wait(&self, timeout: std::time::Duration) -> bool {
        let ms = timeout.as_millis().min(u128::from(u32::MAX - 1)) as u32;
        // SAFETY: waits on this process's console input handle.
        unsafe { WaitForSingleObject(std_handle(Std::In), ms) != WAIT_TIMEOUT }
    }

    pub fn read(&mut self) -> io::Result<Vec<u8>> {
        let handle = std_handle(Std::In);
        let mut units = [0u16; 4096];
        let mut read = 0u32;
        // SAFETY: `units` is a valid buffer of the given length.
        let ok = unsafe {
            ReadConsoleW(
                handle,
                units.as_mut_ptr().cast(),
                units.len() as u32,
                &mut read,
                std::ptr::null(),
            )
        };
        if ok == 0 {
            // SAFETY: reads the calling thread's last error.
            let _ = unsafe { GetLastError() };
            return Err(io::Error::last_os_error());
        }
        Ok(crate::console_text::decode_units(
            &mut self.carry,
            &units[..read as usize],
        ))
    }
}
