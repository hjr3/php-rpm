#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::default::Default;
use std::ffi::{CString, CStr};
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::os::raw::{c_char, c_uchar, c_int, c_void};
use std::ptr;
use std::slice;

use hyper::header::ContentLength;
use hyper::{Request, Response};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

struct ServerContext {
    buffer: Vec<u8>,
}

impl ServerContext {
    pub fn new() -> ServerContext {
        ServerContext {
            buffer: Vec::new(),
        }
    }
}

pub fn bootstrap() {

    unsafe { php_startup(); }
}

pub fn execute(_request: Request, filename: &str) -> Response {
    let file = File::open(filename.clone()).unwrap();
    let fd = file.as_raw_fd();
    let filename = CString::new(filename).unwrap();

    let body = unsafe  {
        let mut handle = _zend_file_handle__bindgen_ty_1::default();
        *handle.fd.as_mut() = fd;
        let mut script = Box::new(zend_file_handle {
            handle: handle,
            filename: filename.as_ptr(),
            opened_path: ptr::null_mut(),
            type_: zend_stream_type::ZEND_HANDLE_FD,
            free_filename: 0,
        });
        let script_ptr: *mut zend_file_handle = &mut *script;

        sapi_globals.sapi_headers.http_response_code = 200;
        let mut context = Box::new(ServerContext::new());
        let context_ptr: *mut ServerContext = &mut *context;
        sapi_globals.server_context = context_ptr as *mut c_void;
        php_request_startup();

        php_execute_script(script_ptr);

        let context = sapi_globals.server_context as *mut ServerContext;
        let buffer: &Vec<u8> = &(*context).buffer;

        sapi_globals.server_context = ptr::null_mut();
        php_request_shutdown(ptr::null_mut());

        buffer
    };


    Response::new()
        .with_header(ContentLength(body.len() as u64))
        .with_body(body.clone())
}

pub fn teardown() {
    unsafe {
        php_module_shutdown();
        sapi_shutdown();
    }
}

unsafe fn php_startup() {

    let mut module = Box::new(sapi_module_struct::default());
    let name = CString::new("php-rpm").unwrap();
    let pretty_name = CString::new("PHP Rust Process Manager").unwrap();

    module.name = name.as_ptr() as *mut c_char;
    module.pretty_name = pretty_name.as_ptr() as *mut c_char;
    module.startup = Some(sapi_server_startup);
    module.shutdown = Some(sapi_server_shutdown);
    module.ub_write = Some(sapi_server_ub_write);
    module.flush = Some(sapi_server_flush);
    //module.sapi_error = Some(php_error);
    module.send_headers = Some(sapi_server_send_headers);
    module.read_post = Some(sapi_server_read_post);
    module.read_cookies = Some(sapi_server_read_cookies);
    module.register_server_variables = Some(sapi_server_register_variables);
    module.log_message = Some(sapi_server_log_message);

    let module_ptr: *mut sapi_module_struct = &mut *module;

    // TODO error check
    sapi_startup(module_ptr);

    let mut request_info = sapi_request_info::default();
    let request_method = CString::new("GET").unwrap();
    let path_translated = CString::new("/home/herman/projects/php-rpm/tests/index.php").unwrap();
    let content_type = CString::new("text/html").unwrap();
    request_info.request_method = request_method.as_ptr();
    request_info.content_length = 0;
    request_info.path_translated = path_translated.as_ptr() as *mut c_char;
    request_info.content_type = content_type.as_ptr();
    sapi_globals.request_info = request_info;

    php_module_startup(module_ptr, ptr::null_mut(), 0);
}

unsafe extern "C" fn sapi_server_startup(_module: *mut sapi_module_struct) -> c_int {
    trace!("sapi_server_startup");
    ZEND_RESULT_CODE::SUCCESS as c_int
}

unsafe extern "C" fn sapi_server_shutdown(_module: *mut sapi_module_struct) -> c_int {
    trace!("sapi_server_shutdown");
    ZEND_RESULT_CODE::SUCCESS as c_int
}

unsafe extern "C" fn sapi_server_ub_write(s: *const c_char, s_len: usize) -> usize {
    trace!("sapi_server_ub_write");
    let _s: *const c_uchar = s as *const c_uchar;
    let rs = slice::from_raw_parts(_s, s_len);
    //io::stdout().write(&rs).unwrap();
    let context = sapi_globals.server_context as *mut ServerContext;
    (*context).buffer.extend_from_slice(&rs);

    s_len
}

unsafe extern "C" fn sapi_server_flush(_server_context: *mut c_void) {
    trace!("sapi_server_flush");
    if sapi_globals.headers_sent == 1 {
        return;
    }

    sapi_send_headers();
}

//unsafe extern "C" fn php_error(type_: c_int, error_msg: *const c_char) {
//}

unsafe extern "C" fn sapi_server_send_headers(_sapi_headers: *mut sapi_headers_struct) -> c_int {
    //trace!("sapi_server_send_headers");
    //let h: &sapi_headers_struct = sapi_headers.as_ref().unwrap();
    ////println!("send_headers: {:?}", h);
    ////pub headers: zend_llist,
    //println!("http_response_code: {}", h.http_response_code);
    //println!("send_default_content_type: {}", h.send_default_content_type);
    //println!("mimetype: {}", CStr::from_ptr(h.mimetype).to_string_lossy());

    //// set by userland using the `header()` function
    //if !h.http_status_line.is_null() {
    //    println!("http_status_line: {}", CStr::from_ptr(h.http_status_line).to_string_lossy());
    //} else {
    //    // create a status line from information. note: this will require us to track things like
    //    // the protocol of the request and the response code
    //}

    // used so flush can try and send headers prior to output
    sapi_globals.headers_sent = 1;

    // bindgen treats this as a `c_uint` type but this function requires a c_int
    SAPI_HEADER_SENT_SUCCESSFULLY as c_int
}

unsafe extern "C" fn sapi_server_read_post(_buffer: *mut c_char, _bytes: usize) -> usize {
    println!("read_post");
    0
}

unsafe extern "C" fn sapi_server_read_cookies() -> *mut c_char {
    trace!("read_cookies");
    ptr::null_mut()
}

unsafe extern "C" fn sapi_server_register_variables(_track_vars_array: *mut zval) {
    trace!("sapi_server_register_varibles");
    // DOCUMENT_ROOT
    // REMOTE_ADDR
    // REMOTE_PORT
    // SERVER_SOFTWARE
    // SERVER_PROTOCOL
    // SERVER_NAME
    // SERVER_PORT
    // REQUEST_URI
    // REQUEST_METHOD
    // SCRIPT_NAME
    // SCRIPT_FILENAME
    // SCRIPT_FILENAME
    // PATH_INFO
    // PHP_SELF
    // PHP_SELF
    // QUERY_STRING
    // CONTENT_TYPE
    // CONTENT_LENGTH
    // every header gets HTTP_*
}

unsafe extern "C" fn sapi_server_log_message(message: *mut c_char, _syslog_type_int: c_int) {
    let m = CStr::from_ptr(message);
    // TODO map syslog levels to log crate macros
    warn!("{}", m.to_string_lossy());
}
