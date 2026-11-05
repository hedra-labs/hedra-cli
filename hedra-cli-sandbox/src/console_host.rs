//! A console of its own for the Windows console tests: `cargo test`'s handles
//! are pipes and every test in one process shares one console, so a test
//! re-runs itself as a host process, which calls this module to allocate a
//! fresh console with code page 437, run the built binary on it, type keys
//! into its input buffer and read its screen buffer back.

use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::io::AsRawHandle;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Console::{
    AllocConsole, FreeConsole, GetConsoleMode, GetConsoleScreenBufferInfo,
    ReadConsoleOutputCharacterW, SetConsoleCP, SetConsoleOutputCP, WriteConsoleInputW,
    CONSOLE_MODE, CONSOLE_SCREEN_BUFFER_INFO, COORD, INPUT_RECORD, KEY_EVENT, LEFT_CTRL_PRESSED,
};

/// The code page the tests set, so a console write that went through the
/// code page instead of `WriteConsoleW` would garble every non-ASCII byte.
pub const CODE_PAGE: u32 = 437;

/// One key typed into the console's input buffer.
#[derive(Clone, Copy, Debug)]
pub enum Key {
    /// A character, typed with no modifier.
    Char(char),
    /// Ctrl and a letter, which a raw console reads as its control byte.
    Ctrl(char),
    /// A key with no character, by its virtual-key code, such as 0x26 for Up.
    Virtual(u16),
}

/// A console this process allocated, and its input and output handles.
pub struct ConsoleHost {
    input: File,
    output: File,
}

impl ConsoleHost {
    /// Detaches from the inherited console, allocates a new one and sets its
    /// code pages to [`CODE_PAGE`].
    pub fn allocate() -> io::Result<Self> {
        // SAFETY: plain console calls on this process.
        unsafe {
            FreeConsole();
            if AllocConsole() == 0
                || SetConsoleOutputCP(CODE_PAGE) == 0
                || SetConsoleCP(CODE_PAGE) == 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        let open = |name: &str| OpenOptions::new().read(true).write(true).open(name);
        Ok(Self {
            input: open("CONIN$")?,
            output: open("CONOUT$")?,
        })
    }

    fn input_handle(&self) -> HANDLE {
        self.input.as_raw_handle() as HANDLE
    }

    fn output_handle(&self) -> HANDLE {
        self.output.as_raw_handle() as HANDLE
    }

    /// Spawns `cmd` with this console as its stdin, stdout and stderr.
    pub fn spawn(&self, cmd: &mut Command) -> io::Result<Child> {
        cmd.stdin(Stdio::from(self.input.try_clone()?))
            .stdout(Stdio::from(self.output.try_clone()?))
            .stderr(Stdio::from(self.output.try_clone()?))
            .spawn()
    }

    /// The input and output handles' console modes.
    pub fn modes(&self) -> io::Result<(CONSOLE_MODE, CONSOLE_MODE)> {
        let read = |handle: HANDLE| {
            let mut mode: CONSOLE_MODE = 0;
            // SAFETY: `mode` is a valid out pointer.
            if unsafe { GetConsoleMode(handle, &mut mode) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(mode)
        };
        Ok((read(self.input_handle())?, read(self.output_handle())?))
    }

    /// Writes a key-down and a key-up record for each key.
    pub fn type_keys(&self, keys: &[Key]) -> io::Result<()> {
        let mut records = Vec::new();
        for key in keys {
            let (unit, vk, state) = match *key {
                Key::Char(c) => (c as u16, 0, 0),
                Key::Ctrl(c) => (
                    (c.to_ascii_uppercase() as u16) & 0x1f,
                    c.to_ascii_uppercase() as u16,
                    LEFT_CTRL_PRESSED,
                ),
                Key::Virtual(vk) => (0, vk, 0),
            };
            for down in [1, 0] {
                let mut record = INPUT_RECORD {
                    EventType: KEY_EVENT as u16,
                    ..Default::default()
                };
                // SAFETY: a KEY_EVENT record's payload is its KeyEvent.
                let event = unsafe { &mut record.Event.KeyEvent };
                event.bKeyDown = down;
                event.wRepeatCount = 1;
                event.wVirtualKeyCode = vk;
                event.uChar.UnicodeChar = unit;
                event.dwControlKeyState = state;
                records.push(record);
            }
        }
        let mut written = 0u32;
        // SAFETY: `records` is a valid buffer of the given length.
        let ok = unsafe {
            WriteConsoleInputW(
                self.input_handle(),
                records.as_ptr(),
                records.len() as u32,
                &mut written,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// The screen buffer's characters, one line per row, trailing spaces
    /// trimmed.
    pub fn screen(&self) -> io::Result<String> {
        // SAFETY: an all-zero CONSOLE_SCREEN_BUFFER_INFO is a valid value.
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
        // SAFETY: `info` is a valid out pointer.
        if unsafe { GetConsoleScreenBufferInfo(self.output_handle(), &mut info) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let width = info.dwSize.X.max(1) as usize;
        let rows = (info.dwCursorPosition.Y as usize + 1).min(info.dwSize.Y.max(1) as usize);
        let mut units = vec![0u16; width * rows];
        let mut read = 0u32;
        // SAFETY: `units` holds `width * rows` units.
        let ok = unsafe {
            ReadConsoleOutputCharacterW(
                self.output_handle(),
                units.as_mut_ptr(),
                units.len() as u32,
                COORD { X: 0, Y: 0 },
                &mut read,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        units.truncate(read as usize);
        Ok(units
            .chunks(width)
            .map(|row| String::from_utf16_lossy(row).trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

/// Waits for `child` up to `timeout`, killing it on expiry, and returns its
/// exit code, or `None` when it had to be killed.
pub fn wait(child: &mut Child, timeout: Duration) -> io::Result<Option<i32>> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status.code());
        }
        if Instant::now() > deadline {
            child.kill()?;
            let _ = child.wait()?;
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
