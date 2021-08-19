fn main() {
    windows::build!(
        Windows::Win32::{
            Storage::FileSystem::{
                GetFileVersionInfoA,
                GetFileVersionInfoSizeA,
                VS_FIXEDFILEINFO,
                VerQueryValueA,
            },
            System::Diagnostics::Debug::GetLastError,
        },
    );
}
