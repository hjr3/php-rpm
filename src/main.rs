#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::default::Default;
use std::fs::File;
use std::mem::transmute;
use std::os::unix::io::AsRawFd;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::ptr;
use std::ffi::CString;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    let argc = 0 as c_int;
    let mut argv: Vec<*mut c_char> = vec![ptr::null_mut()];

    let filename = std::env::args().nth(1).unwrap();
    let file = File::open(filename.clone()).unwrap();
    let fd = file.as_raw_fd();

    let filename = CString::new(filename).unwrap();
    let mut handle = _zend_file_handle__bindgen_ty_1::default();
    unsafe {
        *handle.fd.as_mut() = fd;
        let _script = zend_file_handle {
            handle: handle,
            filename: filename.as_ptr(),
            opened_path: ptr::null_mut(),
            type_: zend_stream_type::ZEND_HANDLE_FD,
            free_filename: 0,
        };
        let script: *mut zend_file_handle = transmute(Box::new(_script));
        php_embed_init(argc, argv.as_mut_ptr());
        php_execute_script(script);
        php_embed_shutdown();
    }
}
