#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::c_char;
use std::os::raw::c_int;
use std::ptr;
use std::ffi::CString;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    //let args = std::env::args().map(|arg| CString::new(arg).unwrap()).collect::<Vec<CString>>();
    //let mut c_args = args.iter().map(|arg| arg.as_ptr()).collect::<Vec<*const c_char>>();
    //c_args.push(ptr::null());

    //let argc: c_int = c_args.len() as c_int;
    //let argv: *mut *mut c_char = c_args.as_ptr() as *mut *mut c_char;
    let argc = 0 as c_int;
    let mut argv: Vec<*mut c_char> = vec![ptr::null_mut()];
    let eval_str = CString::new("echo 1 + 1, PHP_EOL;").unwrap();
    let retval_ptr = ptr::null_mut();
    let string_name = CString::new("php-rpm eval'd string").unwrap();
    unsafe {
        php_embed_init(argc, argv.as_mut_ptr());
        zend_eval_string(eval_str.as_ptr() as *mut c_char, retval_ptr, string_name.as_ptr() as *mut c_char);
        php_embed_shutdown();
    }
}
