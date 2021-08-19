#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, warnings)]

windows::include_bindings!();

use {
    std::{
        convert::TryInto as _,
        ffi::c_void,
        path::Path,
        ptr::null_mut,
    },
    windows::HRESULT,
    crate::Windows::Win32::{
        Storage::FileSystem,
        System::Diagnostics::Debug::GetLastError,
    },
};

unsafe fn get_last_error<T>() -> windows::Result<T> {
    Err(HRESULT::from_win32(GetLastError().0).into())
}

/// Returns the file version of the file at a given path.
///
/// This is the same field displayed by File Explorer as Properties → Details → File version.
pub fn get_file_version_info(path: impl AsRef<Path>) -> windows::Result<[u16; 4]> {
    let path = path.as_ref().to_str().expect("non-Unicode path (should be impossible on Windows)");
    unsafe {
        let mut unused = 0;
        let buf_size = FileSystem::GetFileVersionInfoSizeA(path, &mut unused);
        if buf_size == 0 { return get_last_error() }
        let mut buf = vec![0; buf_size.try_into().expect("buffer too long (16-bit Windows is not supported)")].into_boxed_slice();
        if !FileSystem::GetFileVersionInfoA(path, unused, buf_size, buf.as_mut_ptr() as *mut c_void).as_bool() { return get_last_error() }
        let mut ver_data = null_mut();
        let mut data_size = 0;
        FileSystem::VerQueryValueA(buf.as_ptr() as *const c_void, "\\", &mut ver_data, &mut data_size).expect("got invalid version-information resource from Windows");
        let ver_data = &*(ver_data as *const FileSystem::VS_FIXEDFILEINFO);
        if ver_data.dwSignature != 0xfeef04bd { panic!("wrong magic numbers in VS_FIXEDFILEINFO: expected 0xfeef04bd, got 0x{:08x}", ver_data.dwSignature) }
        Ok([(ver_data.dwFileVersionMS >> 16) as u16, ver_data.dwFileVersionMS as u16, (ver_data.dwFileVersionLS >> 16) as u16, ver_data.dwFileVersionLS as u16])
    }
}
