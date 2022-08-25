#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, warnings)]

use {
    std::{
        convert::TryInto as _,
        ffi::OsStr,
        io,
        iter,
        os::windows::ffi::OsStrExt as _,
        path::Path,
        ptr::null_mut,
    },
    winapi::{
        ctypes::c_void,
        shared::minwindef::DWORD,
        um::winver::{
            GetFileVersionInfoSizeW,
            GetFileVersionInfoW,
            VerQueryValueW,
        },
    },
};

#[repr(C)]
#[allow(non_snake_case)]
struct VsFixedFileInfo {
    dwSignature: DWORD,
    dwStrucVersion: DWORD,
    dwFileVersionMS: DWORD,
    dwFileVersionLS: DWORD,
    dwProductVersionMS: DWORD,
    dwProductVersionLS: DWORD,
    dwFileFlagsMask: DWORD,
    dwFileFlags: DWORD,
    dwFileOS: DWORD,
    dwFileType: DWORD,
    dwFileSubtype: DWORD,
    dwFileDateMS: DWORD,
    dwFileDateLS: DWORD,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)] Io(#[from] io::Error),
    /// got invalid version-information resource from Windows
    #[error("got invalid version-information resource from Windows")]
    InvalidVersionInfo,
}

/// Returns the file version of the file at a given path.
///
/// This is the same field displayed by File Explorer as Properties → Details → File version.
pub fn get_file_version_info(path: impl AsRef<Path>) -> Result<[u16; 4], Error> {
    let path = path.as_ref().as_os_str().encode_wide().chain(iter::once(0)).collect::<Vec<_>>();
    let root = OsStr::new("\\").encode_wide().chain(iter::once(0)).collect::<Vec<_>>();
    unsafe {
        let mut unused = 0;
        let buf_size = GetFileVersionInfoSizeW(path.as_ptr(), &mut unused);
        if buf_size == 0 { return Err(io::Error::last_os_error().into()) }
        let mut buf = vec![0u8; buf_size.try_into().expect("buffer too long (16-bit Windows is not supported)")];
        if GetFileVersionInfoW(path.as_ptr(), unused, buf_size, buf.as_mut_ptr() as *mut c_void) == 0 { return Err(io::Error::last_os_error().into()) }
        let mut ver_data = null_mut();
        let mut data_size = 0;
        if VerQueryValueW(buf.as_ptr() as *const c_void, root.as_ptr(), &mut ver_data, &mut data_size) == 0 { return Err(Error::InvalidVersionInfo) }
        let ver_data = &*(ver_data as *const VsFixedFileInfo);
        if ver_data.dwSignature != 0xfeef04bd { panic!("wrong magic numbers in VS_FIXEDFILEINFO: expected 0xfeef04bd, got 0x{:08x}", ver_data.dwSignature) }
        Ok([(ver_data.dwFileVersionMS >> 16) as u16, ver_data.dwFileVersionMS as u16, (ver_data.dwFileVersionLS >> 16) as u16, ver_data.dwFileVersionLS as u16])
    }
}
