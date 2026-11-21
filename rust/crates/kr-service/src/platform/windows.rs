//! Windows filesystem boundary. All pointer lifetimes and Win32 allocations
//! stay here; the service uses only safe Path-based operations.
use std::{
    ffi::c_void,
    fs::Metadata,
    io, mem,
    os::windows::{
        ffi::OsStrExt,
        fs::MetadataExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::Path,
    ptr::null_mut,
};
use windows_sys::Win32::{
    Foundation::{ERROR_INSUFFICIENT_BUFFER, LocalFree},
    Security::{
        ACE_HEADER, ACL_SIZE_INFORMATION, AclSizeInformation,
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
            GetNamedSecurityInfoW, SE_FILE_OBJECT,
        },
        CONTAINER_INHERIT_ACE, CreateWellKnownSid, DACL_SECURITY_INFORMATION, EqualSid, GetAce,
        GetAclInformation, GetTokenInformation, INHERIT_ONLY_ACE, NO_PROPAGATE_INHERIT_ACE,
        OBJECT_INHERIT_ACE, OWNER_SECURITY_INFORMATION, PSID, SECURITY_ATTRIBUTES, TOKEN_QUERY,
        TOKEN_USER, TokenUser, WinLocalSystemSid,
    },
    Storage::FileSystem::{
        CreateDirectoryW, FILE_ATTRIBUTE_REPARSE_POINT, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    },
    System::{
        SystemServices::{ACCESS_ALLOWED_ACE_TYPE, ACCESS_DENIED_ACE_TYPE},
        Threading::{GetCurrentProcess, OpenProcessToken},
    },
};

struct LocalAllocation(*mut c_void);
impl Drop for LocalAllocation {
    fn drop(&mut self) {
        // SAFETY: The pointer is null or comes from an API allocating via LocalAlloc.
        unsafe { LocalFree(self.0) };
    }
}

struct User(Vec<usize>);
impl User {
    fn current() -> io::Result<Self> {
        // SAFETY: OpenProcessToken initializes the handle on success. The buffer
        // is aligned for TOKEN_USER and remains allocated while its SID is used.
        unsafe {
            let mut raw = null_mut();
            check(OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw))?;
            let token = OwnedHandle::from_raw_handle(raw);
            let mut bytes = 0;
            if GetTokenInformation(token.as_raw_handle(), TokenUser, null_mut(), 0, &mut bytes) != 0
                || io::Error::last_os_error().raw_os_error()
                    != Some(ERROR_INSUFFICIENT_BUFFER as i32)
                || bytes < mem::size_of::<TOKEN_USER>() as u32
            {
                return Err(io::Error::other("could not size Windows process token"));
            }
            let mut buffer = vec![0usize; (bytes as usize).div_ceil(mem::size_of::<usize>())];
            check(GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                buffer.as_mut_ptr().cast(),
                bytes,
                &mut bytes,
            ))?;
            Ok(Self(buffer))
        }
    }

    fn sid(&self) -> PSID {
        // SAFETY: GetTokenInformation initialized a TOKEN_USER in this aligned buffer.
        unsafe { (*self.0.as_ptr().cast::<TOKEN_USER>()).User.Sid }
    }
}

fn check(success: i32) -> io::Result<()> {
    if success == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn wide_path(path: &Path) -> io::Result<Vec<u16>> {
    // Verbatim paths support state directories longer than MAX_PATH. The
    // parent may exist while the destination does not (create/publish).
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("path parent missing"))?;
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::other("path file name missing"))?;
    let path = parent.canonicalize()?.join(name);
    let mut wide: Vec<_> = path.as_os_str().encode_wide().collect();
    if wide.contains(&0) {
        return Err(io::Error::other("path contains NUL"));
    }
    wide.push(0);
    Ok(wide)
}

pub(super) fn create_private_dir(path: &Path) -> io::Result<()> {
    let user = User::current()?;
    let path = wide_path(path)?;
    // SAFETY: SID and descriptor buffers are alive across the calls. Both
    // Windows-allocated results are released with LocalFree.
    unsafe {
        let mut sid = null_mut();
        check(ConvertSidToStringSidW(user.sid(), &mut sid))?;
        let _sid = LocalAllocation(sid.cast());
        let length = (0..).find(|n| *sid.add(*n) == 0).unwrap();
        let sid = String::from_utf16_lossy(std::slice::from_raw_parts(sid, length));
        let sddl: Vec<_> = format!("D:P(A;OICI;FA;;;{sid})(A;OICI;FA;;;SY)\0")
            .encode_utf16()
            .collect();
        let mut descriptor = LocalAllocation(null_mut());
        check(ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor.0,
            null_mut(),
        ))?;
        let attributes = SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor.0,
            bInheritHandle: 0,
        };
        check(CreateDirectoryW(path.as_ptr(), &attributes))
    }
}

pub(super) fn check_private(path: &Path, meta: &Metadata, directory: bool) -> io::Result<()> {
    let invalid =
        || io::Error::other("state must have a private Windows ACL owned by the current user");
    if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(invalid());
    }
    let path = wide_path(path)?;
    let user = User::current()?;
    // SAFETY: GetNamedSecurityInfo owns the descriptor and points into it.
    // GetAce returns a valid ACE from its DACL; we look at the SID only for
    // conventional ACCESS_ALLOWED_ACE entries.
    unsafe {
        let mut acl = null_mut();
        let mut owner = null_mut();
        let mut descriptor = LocalAllocation(null_mut());
        let status = GetNamedSecurityInfoW(
            path.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            null_mut(),
            &mut acl,
            null_mut(),
            &mut descriptor.0,
        );
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        if acl.is_null() || owner.is_null() || EqualSid(owner, user.sid()) == 0 {
            return Err(invalid());
        }
        let mut system_sid = [0u32; 17]; // SECURITY_MAX_SID_SIZE (68 bytes)
        let mut size = mem::size_of_val(&system_sid) as u32;
        let system: PSID = system_sid.as_mut_ptr().cast();
        check(CreateWellKnownSid(
            WinLocalSystemSid,
            null_mut(),
            system,
            &mut size,
        ))?;
        let mut info = ACL_SIZE_INFORMATION::default();
        check(GetAclInformation(
            acl,
            (&mut info as *mut ACL_SIZE_INFORMATION).cast(),
            mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
            AclSizeInformation,
        ))?;
        let inherit = OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE;
        let mut inherited_user_access = false;
        for index in 0..info.AceCount {
            let mut entry = null_mut();
            check(GetAce(acl, index, &mut entry))?;
            let header = &*entry.cast::<ACE_HEADER>();
            match u32::from(header.AceType) {
                ACCESS_DENIED_ACE_TYPE => continue,
                ACCESS_ALLOWED_ACE_TYPE => {}
                _ => return Err(invalid()),
            }
            // ACCESS_ALLOWED_ACE layout: header, access mask, then SID.
            let sid: PSID = entry.cast::<u32>().add(2).cast();
            if EqualSid(sid, user.sid()) != 0 {
                let flags = u32::from(header.AceFlags);
                inherited_user_access |=
                    flags & (inherit | NO_PROPAGATE_INHERIT_ACE | INHERIT_ONLY_ACE) == inherit;
            } else if EqualSid(sid, system) == 0 {
                return Err(invalid());
            }
        }
        if directory && !inherited_user_access {
            return Err(invalid());
        }
        Ok(())
    }
}

pub(super) fn publish_snapshot(source: &Path, destination: &Path) -> io::Result<()> {
    let source = wide_path(source)?;
    let destination = wide_path(destination)?;
    // SAFETY: Both paths are live NUL-terminated strings. No overwrite flag.
    check(unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_WRITE_THROUGH,
        )
    })
}
