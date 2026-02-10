#![allow(dead_code)]

use std::{ffi::CStr, fs::File, io::Write};

pub fn mrbc_compile(fname: &'static str, code: &'static str) -> Vec<u8> {
    let mut src = std::env::temp_dir();
    src.push(format!("{}.{}.rb", fname, std::process::id()));
    let mut f = File::create(&src).expect("cannot open src file");
    f.write_all(code.as_bytes())
        .expect("cannot create src file");
    f.flush().unwrap();

    let mut src0 = src.as_os_str().to_string_lossy().into_owned();
    src0.push('\0');

    let mut dest = std::env::temp_dir();
    dest.push(format!("{}.{}.mrb", fname, std::process::id()));
    let mut dest0 = dest.as_os_str().to_string_lossy().into_owned();
    dest0.push('\0');

    let args = [
        CStr::from_bytes_with_nul(b"mrbc\0").unwrap().as_ptr(),
        CStr::from_bytes_with_nul(b"-o\0").unwrap().as_ptr(),
        CStr::from_bytes_with_nul(dest0.as_bytes())
            .unwrap()
            .as_ptr(),
        CStr::from_bytes_with_nul(src0.as_bytes()).unwrap().as_ptr(),
    ];
    unsafe {
        mec_mrbc_sys::mrbc_main(args.len() as i32, args.as_ptr() as *mut *mut i8);
    }

    std::fs::read(dest).unwrap()
}
