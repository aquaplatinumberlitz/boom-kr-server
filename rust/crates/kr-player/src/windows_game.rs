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
