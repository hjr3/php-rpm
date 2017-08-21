use std::default::Default;
use std::ffi::{CString, CStr};
use std::fs::File;
use std::net::SocketAddr;
use std::os::unix::io::AsRawFd;
use std::os::raw::{c_char, c_uchar, c_int, c_void};
use std::path::{PathBuf, Path};
use std::ptr;
use std::slice;
use std::str;

use hyper::header::{Headers, ContentLength, ContentType, Cookie};
use hyper::{self, Response};

use php;

struct ServerContext {
    buffer: Vec<u8>,
    response: Response,
    document_root: PathBuf,
    script: String,
    addr: SocketAddr,
    method: hyper::Method,
    uri: hyper::Uri,
    http_version: hyper::HttpVersion,
    headers: hyper::Headers,
    body: Box<[u8]>,
}

impl ServerContext {
    pub fn new(method: hyper::Method, uri: hyper::Uri, http_version: hyper::HttpVersion, headers: hyper::Headers, body: Box<[u8]>, document_root: PathBuf, script: String, addr: SocketAddr) -> ServerContext {
        ServerContext {
            buffer: Vec::new(),
            response: Response::new(),
            document_root: document_root,
            script: script,
            addr: addr,
            method: method,
            uri: uri,
            http_version: http_version,
            headers: headers,
            body: body,
        }
    }
}

pub fn bootstrap() {

    unsafe { php_startup(); }
}

pub fn execute(method: hyper::Method, uri: hyper::Uri, http_version: hyper::HttpVersion, headers: hyper::Headers, chunk: &[u8], document_root: &Path, filename: &str, addr: &SocketAddr) -> Response {
    let doc_root = document_root.to_path_buf();
    let mut abs_filename = doc_root.clone();
    abs_filename.push(filename);
    let file = File::open(abs_filename).unwrap();
    let fd = file.as_raw_fd();
    let handle_filename = CString::new(filename).unwrap();

    let (headers, body) = unsafe  {
        let mut handle = php::_zend_file_handle__bindgen_ty_1::default();
        *handle.fd.as_mut() = fd;
        let mut script = Box::new(php::zend_file_handle {
            handle: handle,
            filename: handle_filename.as_ptr(),
            opened_path: ptr::null_mut(),
            type_: php::zend_stream_type::ZEND_HANDLE_FD,
            free_filename: 0,
        });
        let script_ptr: *mut php::zend_file_handle = &mut *script;

        php::sapi_globals.sapi_headers.http_response_code = 200;
        let mut context = Box::new(ServerContext::new(method, uri, http_version, headers, chunk.to_owned().into_boxed_slice(), doc_root, filename.to_string(), addr.clone()));
        let context_ptr: *mut ServerContext = &mut *context;
        php::sapi_globals.server_context = context_ptr as *mut c_void;
        php::php_request_startup();

        php::php_execute_script(script_ptr);

        let context = php::sapi_globals.server_context as *mut ServerContext;
        let buffer: &Vec<u8> = &(*context).buffer;
        let headers = (*context).response.headers().clone();

        // TODO move this into the request shutdown block
        // note: strangely enough, php_request_shutdown will not call our request shutdown callback
        if !php::sapi_globals.request_info.cookie_data.is_null() {
            drop(CString::from_raw(php::sapi_globals.request_info.cookie_data));
        }
        php::sapi_globals.request_info.cookie_data = ptr::null_mut();
        php::sapi_globals.server_context = ptr::null_mut();

        php::php_request_shutdown(ptr::null_mut());

        (headers, buffer)
    };

    Response::new()
        .with_headers(headers)
        .with_header(ContentLength(body.len() as u64))
        .with_body(body.clone())
}

pub fn teardown() {
    unsafe {
        php::php_module_shutdown();
        php::sapi_shutdown();

        drop(CString::from_raw(php::sapi_globals.request_info.path_translated));
        drop(CString::from_raw(php::sapi_module.name));
        drop(CString::from_raw(php::sapi_module.pretty_name));

        // PHP is a bit funny in that we create an `php::sapi_module_struct`, create a pointer to that
        // and then pass it to some core PHP functions. When it is passed, those core functions
        // dereference the pointer and store the memory in a global. As we used `Box::into_raw` to
        // create the pointer, we need to use `Box::from_raw` to free the underlying memory. In order
        // to use `Box::from_raw`, we must first get a reference to the global.
        // TODO check that this is the right way to do this
        drop(Box::from_raw(&mut php::sapi_module));
    }
}

unsafe fn php_startup() {

    let mut module = Box::new(php::sapi_module_struct::default());
    let name = CString::new("php-rpm").unwrap();
    let pretty_name = CString::new("PHP Rust Process Manager").unwrap();

    module.name = name.into_raw();
    module.pretty_name = pretty_name.into_raw();
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

    let module_ptr = Box::into_raw(module);

    // TODO error check
    // this function assigns the module pointer to the `php::sapi_module` global variable
    php::sapi_startup(module_ptr);

    let mut request_info = php::sapi_request_info::default();
    let request_method = CString::new("GET").unwrap();
    let path_translated = CString::new("/home/herman/projects/php-rpm/tests/index.php").unwrap();
    let content_type = CString::new("text/html").unwrap();
    request_info.request_method = request_method.as_ptr();
    request_info.content_length = 0;
    request_info.path_translated = path_translated.into_raw();
    request_info.content_type = content_type.as_ptr();
    php::sapi_globals.request_info = request_info;

    // this function also assigns the module pointer to the `php::sapi_module` global variable
    php::php_module_startup(module_ptr, ptr::null_mut(), 0);
}

unsafe extern "C" fn sapi_server_startup(_module: *mut php::sapi_module_struct) -> c_int {
    trace!("sapi_server_startup");
    php::ZEND_RESULT_CODE::SUCCESS as c_int
}

unsafe extern "C" fn sapi_server_shutdown(_module: *mut php::sapi_module_struct) -> c_int {
    trace!("sapi_server_shutdown");
    php::ZEND_RESULT_CODE::SUCCESS as c_int
}

unsafe extern "C" fn sapi_server_ub_write(s: *const c_char, s_len: usize) -> usize {
    trace!("sapi_server_ub_write");
    let s_: *const c_uchar = s as *const c_uchar;
    let rs = slice::from_raw_parts(s_, s_len);
    //io::stdout().write(&rs).unwrap();
    let context = php::sapi_globals.server_context as *mut ServerContext;
    (*context).buffer.extend_from_slice(&rs);

    s_len
}

unsafe extern "C" fn sapi_server_flush(_server_context: *mut c_void) {
    trace!("sapi_server_flush");
    if php::sapi_globals.headers_sent == 1 {
        return;
    }

    php::sapi_send_headers();
}

//unsafe extern "C" fn php_error(type_: c_int, error_msg: *const c_char) {
//}

// TODO hyper has no way to set the HTTP version. this should change when the http crate is stabilized
fn set_response_status(response: &mut Response, _request_http_version: hyper::HttpVersion, http_status_line: *mut c_char, sapi_http_response_code: c_int) {
    if http_status_line.is_null() {
        response.set_status(hyper::StatusCode::try_from(sapi_http_response_code as u16).unwrap());
    } else {
        // TODO use a better parser for this
        let status_line = unsafe { CStr::from_ptr(http_status_line).to_string_lossy() };
        let (_, status) = status_line.split_at("HTTP/1.1 ".len());
        let status_code = &status[0..3];
        let status_code = status_code.parse::<u16>().unwrap();
        response.set_status(hyper::StatusCode::try_from(status_code).unwrap());
    }
}

unsafe extern "C" fn sapi_server_send_headers(sapi_headers: *mut php::sapi_headers_struct) -> c_int {
    trace!("sapi_server_send_headers");
    let context = php::sapi_globals.server_context as *mut ServerContext;

    let shs: &php::sapi_headers_struct = sapi_headers.as_ref().unwrap();

    set_response_status(&mut (*context).response, (*context).http_version, shs.http_status_line, shs.http_response_code);

    let mut headers = shs.headers;
    let mut zle = php::zend_llist_element::default();
    let mut pos: php::zend_llist_position = &mut zle;
    let mut h = php::zend_llist_get_first_ex(&mut headers, &mut pos);

    while !h.is_null() {
        let header = h as *const php::sapi_header_struct;

        if (*header).header_len > 0 {
            let v: *const c_uchar = (*header).header as *const c_uchar;
            let rs = slice::from_raw_parts(v, (*header).header_len);
            let colon_pos = rs.iter().position(|c| *c == ':' as u8).unwrap();
            let (name, value) = rs.split_at(colon_pos);

            // we must allocate memory here so we do not pass a PHP allocated string to Rust
            let name = String::from_utf8_unchecked(name.to_vec());
            let value = String::from_utf8_unchecked(value[2..].to_vec());
            (*context).response.headers_mut().set_raw(name, value);
        }
        h = php::zend_llist_get_next_ex(&mut headers, &mut pos);
    }


    // used so flush can try and send headers prior to output
    php::sapi_globals.headers_sent = 1;

    // bindgen treats this as a `c_uint` type but this function requires a c_int
    php::SAPI_HEADER_SENT_SUCCESSFULLY as c_int
}

unsafe extern "C" fn sapi_server_read_post(buf: *mut c_char, bytes: usize) -> usize {
    trace!("read_post");

    let context = php::sapi_globals.server_context as *mut ServerContext;
    let body = &(*context).body;
    let copied = ::std::cmp::min(bytes, body.len());
    if copied > 0 {
        let (to_send, to_retain) = body.split_at(copied);
        let ptr = to_send.as_ptr() as *const i8;
        ::std::ptr::copy(ptr, buf, copied);
        (*context).body = to_retain.to_owned().into_boxed_slice();

    }
    copied
}

fn read_cookies(context: &ServerContext) -> *mut c_char {
    let cookies = (*context).headers.get::<Cookie>();
    match cookies {
        Some(c) => {
            let value = format!("{}", c);
            let value = CString::new(value).unwrap();
            value.into_raw()
        }
        None => ptr::null_mut(),
    }
}

unsafe extern "C" fn sapi_server_read_cookies() -> *mut c_char {
    trace!("read_cookies");
    let context = php::sapi_globals.server_context as *mut ServerContext;
    read_cookies(&(*context))
}

fn register_variable(key: &str, value: &str, track_vars_array: *mut php::zval) {
    // TODO answer the below question:
    // for ffi how bad is it to do the cast below? i am essentially saying that PHP will never
    // actually mutate these variables. otherwise, i have to loop back through this hash table and
    // manually free all of the underlying data
    let key = CString::new(key).unwrap();
    let key_ptr = key.as_ptr() as *mut c_char;
    let value_len = value.len();
    let value = CString::new(value).unwrap();
    let value_ptr = value.as_ptr() as *mut c_char;

    unsafe {
        php::php_register_variable_safe(key_ptr, value_ptr, value_len, track_vars_array);
    }
}

fn register_variables(context: &ServerContext, track_vars_array: *mut php::zval) {

    register_variable("PHP_SELF", &context.script, track_vars_array);
    // TODO script name should include path info
    register_variable("SCRIPT_NAME", &context.script, track_vars_array);

    let mut script_filename = context.document_root.clone();
    script_filename.push(&context.script);
    register_variable("SCRIPT_FILENAME", &script_filename.to_string_lossy(), track_vars_array);
    register_variable("DOCUMENT_ROOT", &context.document_root.to_string_lossy(), track_vars_array);
    register_variable("REQUEST_METHOD", context.method.as_ref(), track_vars_array);
    register_variable("REQUEST_URI", context.uri.as_ref(), track_vars_array);
    register_variable("QUERY_STRING", context.uri.query().unwrap_or(""), track_vars_array);
    // per PHP documentation, argv contains the query string on a GET request
    register_variable("argv", context.uri.query().unwrap_or(""), track_vars_array);
    register_variable("SERVER_SOFTWARE", "PHP RPM", track_vars_array);
    register_variable("SERVER_PROTOCOL", &format!("{}", context.http_version), track_vars_array);
    register_variable("SERVER_ADDR", &format!("{}", context.addr.ip()), track_vars_array);
    register_variable("SERVER_PORT", &context.addr.port().to_string(), track_vars_array);

    let headers: &Headers = &context.headers;
    for header in headers.iter() {
        let name = header.name().to_uppercase().replace("-", "_");

        if header.is::<ContentType>() {
            register_variable("CONTENT_TYPE", &header.value_string(), track_vars_array);
        }

        if header.is::<ContentLength>() {
            register_variable("CONTENT_LENGTH", &header.value_string(), track_vars_array);
        }

        let key = format!("HTTP_{}", &name);
        register_variable(&key, &header.value_string(), track_vars_array);
    }

    // TODO
    // SERVER_NAME
    // REMOTE_ADDR
    // REMOTE_PORT
    // PATH_INFO
}

unsafe extern "C" fn sapi_server_register_variables(track_vars_array: *mut php::zval) {
    trace!("sapi_server_register_varibles");

    let context = php::sapi_globals.server_context as *mut ServerContext;
    register_variables(&(*context), track_vars_array);
}

unsafe extern "C" fn sapi_server_log_message(message: *mut c_char, _syslog_type_int: c_int) {
    let m = CStr::from_ptr(message);
    // TODO map syslog levels to log crate macros
    warn!("{}", m.to_string_lossy());
}
