//! Use the standard Windows shell for the pinned CA.exe if CreateProcess
//! reports ERROR_ELEVATION_REQUIRED. The OS shows its ordinary UAC prompt.
use std::{
    io,
    mem::size_of,
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::ExitStatusExt,
    },
    path::Path,
    process::ExitStatus,
};
use windows_sys::Win32::{
    Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{GetExitCodeProcess, INFINITE, TerminateProcess, WaitForSingleObject},
    UI::{
        Shell::{SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW},
        WindowsAndMessaging::SW_SHOWNORMAL,
    },
};

pub(crate) struct Game(OwnedHandle);
impl Game {
    pub(crate) fn open(executable: &Path, directory: &Path) -> io::Result<Self> {
        fn wide(path: &Path) -> Vec<u16> {
            path.as_os_str().encode_wide().chain([0]).collect()
        }
        let executable = wide(executable);
        let directory = wide(directory);
        let verb = wide(Path::new("runas"));
        let arguments = wide(Path::new("multiserveridx:0"));
        let mut entry = SHELLEXECUTEINFOW {
            cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
            lpVerb: verb.as_ptr(),
            lpFile: executable.as_ptr(),
            lpParameters: arguments.as_ptr(),
            lpDirectory: directory.as_ptr(),
            nShow: SW_SHOWNORMAL,
            ..Default::default()
        };
        // SAFETY: This is a fully initialized shell request. All UTF-16 buffers
        // are nul terminated and remain live through synchronous dispatch.
        if unsafe { ShellExecuteExW(&mut entry) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if entry.hProcess.is_null() {
            return Err(io::Error::other("Windows game process handle missing"));
        }
        // SAFETY: NOCLOSEPROCESS returns ownership of this non-null handle.
        Ok(Self(unsafe {
            OwnedHandle::from_raw_handle(entry.hProcess)
        }))
    }

    pub(crate) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.wait_for(0)
    }

    pub(crate) fn wait(&mut self) -> io::Result<ExitStatus> {
        self.wait_for(INFINITE)?
            .ok_or_else(|| io::Error::other("Windows game wait timed out"))
    }

    fn wait_for(&self, timeout: u32) -> io::Result<Option<ExitStatus>> {
        let handle = self.0.as_raw_handle();
        // SAFETY: The owned process handle is live; GetExitCodeProcess writes
        // into a valid u32 only once the process is observed to have exited.
        unsafe {
            match WaitForSingleObject(handle, timeout) {
                WAIT_TIMEOUT => Ok(None),
                WAIT_OBJECT_0 => {
                    let mut code = 0;
                    if GetExitCodeProcess(handle, &mut code) == 0 {
                        return Err(io::Error::last_os_error());
                    }
                    Ok(Some(ExitStatus::from_raw(code)))
                }
                _ => Err(io::Error::last_os_error()),
            }
        }
    }

    pub(crate) fn kill(&mut self) -> io::Result<()> {
        // SAFETY: The handle refers only to the specific game instance.
        if unsafe { TerminateProcess(self.0.as_raw_handle(), 1) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

// Keep privileged commands fixed: the only inputs are the installed program
// paths, used as netsh arguments. No script from the archive is elevated.
pub(crate) fn allow_lan(launcher: &Path, companion: &Path) -> io::Result<()> {
    use windows_sys::Win32::{
        System::SystemInformation::GetSystemDirectoryW, UI::WindowsAndMessaging::SW_HIDE,
    };
    let mut system = [0u16; 260];
    // SAFETY: The buffer is writable for the supplied capacity.
    let n = unsafe { GetSystemDirectoryW(system.as_mut_ptr(), system.len() as u32) } as usize;
    if n == 0 || n >= system.len() {
        return Err(io::Error::other("Windows system directory unavailable"));
    }
    use std::os::windows::ffi::OsStringExt;
    let executable =
        std::path::PathBuf::from(std::ffi::OsString::from_wide(&system[..n])).join("netsh.exe");
    if [launcher, companion].iter().any(|path| {
        path.as_os_str()
            .encode_wide()
            .any(|ch| matches!(ch, 0..=31 | 34))
    }) {
        return Err(io::Error::other("invalid Windows program path"));
    }
    let names = [launcher, companion].map(|path| {
        let mut p: Vec<_> = path.as_os_str().encode_wide().collect();
        if p.starts_with(&[92, 92, 63, 92]) {
            p.drain(..4);
        }
        String::from_utf16_lossy(&p)
    });
    // An elevated built-in PowerShell runs fixed netsh commands in one UAC
    // prompt. Encode paths as UTF-16 literals with single quotes escaped.
    let mut script = String::from("$ErrorActionPreference='Stop';");
    for (kind, name) in ["Launcher", "Companion"].into_iter().zip(names) {
        use sha2::{Digest, Sha256};
        let hash: String = Sha256::digest(name.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let rule = format!("LANBoom-{kind}-{hash}");
        let name = name.replace('\'', "''");
        use std::fmt::Write;
        let _ = write!(
            script,
            "& ([IO.Path]::Combine([Environment]::SystemDirectory,'netsh.exe')) advfirewall firewall delete rule 'name={rule}' | Out-Null; & ([IO.Path]::Combine([Environment]::SystemDirectory,'netsh.exe')) advfirewall firewall add rule 'name={rule}' dir=in action=allow 'program={name}' protocol=UDP remoteip=LocalSubnet profile=private,domain enable=yes | Out-Null; if($LASTEXITCODE -ne 0){{exit 1}};"
        );
    }
    // Quote complete PowerShell script for Windows' command-line parser:
    // paths cannot contain double quotes, and the script ends in ';'.
    let args = format!("-NoProfile -NonInteractive -Command \"{script}\"");
    let powershell = executable
        .with_file_name("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");
    let mut child = shell_run(&powershell, &args, SW_HIDE)?;
    if child.wait()?.success() {
        Ok(())
    } else {
        Err(io::Error::other("netsh failed"))
    }
}

fn shell_run(program: &Path, arguments: &str, show: i32) -> io::Result<Game> {
    let executable: Vec<_> = program.as_os_str().encode_wide().chain([0]).collect();
    let verb: Vec<_> = "runas".encode_utf16().chain([0]).collect();
    let arguments: Vec<_> = arguments.encode_utf16().chain([0]).collect();
    let mut entry = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpVerb: verb.as_ptr(),
        lpFile: executable.as_ptr(),
        lpParameters: arguments.as_ptr(),
        nShow: show,
        ..Default::default()
    };
    // SAFETY: Buffers are terminated and live through synchronous dispatch.
    if unsafe { ShellExecuteExW(&mut entry) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if entry.hProcess.is_null() {
        return Err(io::Error::other("Windows process handle missing"));
    }
    // SAFETY: NOCLOSEPROCESS transfers ownership of this non-null handle.
    Ok(Game(unsafe {
        OwnedHandle::from_raw_handle(entry.hProcess)
    }))
}

pub(crate) fn save_text_path() -> io::Result<Option<std::path::PathBuf>> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::UI::Controls::Dialogs::{
        CommDlgExtendedError, GetSaveFileNameW, OFN_NOCHANGEDIR, OFN_PATHMUSTEXIST, OPENFILENAMEW,
    };
    let mut file = [0u16; 32768];
    for (dst, src) in file.iter_mut().zip("LANBoom-recovery.txt".encode_utf16()) {
        *dst = src;
    }
    let filter: Vec<_> = "Text (*.txt)\0*.txt\0\0".encode_utf16().collect();
    let extension: Vec<_> = "txt\0".encode_utf16().collect();
    let mut dialog = OPENFILENAMEW {
        lStructSize: size_of::<OPENFILENAMEW>() as u32,
        lpstrFilter: filter.as_ptr(),
        lpstrFile: file.as_mut_ptr(),
        nMaxFile: file.len() as u32,
        lpstrDefExt: extension.as_ptr(),
        Flags: OFN_NOCHANGEDIR | OFN_PATHMUSTEXIST,
        ..Default::default()
    };
    // SAFETY: The dialog writes to the live buffer, bounded by nMaxFile.
    if unsafe { GetSaveFileNameW(&mut dialog) } == 0 {
        // SAFETY: Reads the error from the immediately preceding dialog.
        return if unsafe { CommDlgExtendedError() } == 0 {
            Ok(None)
        } else {
            Err(io::Error::other("Windows save dialog failed"))
        };
    }
    let n = file.iter().position(|ch| *ch == 0).unwrap_or(file.len());
    Ok(Some(std::ffi::OsString::from_wide(&file[..n]).into()))
}
