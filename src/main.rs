extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
//extern crate env_logger;
extern crate php_sys as php;

use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;

use futures::{Future, Stream};

use hyper::server::{Http, Request, Response, Service};

pub mod sapi;

/// Server representation
///
/// It is silly, but we do need to separate the document root from the script name. For example:
/// the document root may be `/web` and the script name may be `foo/index.php`. If we treat it as
/// one path, then we will not know where the document root truly is.
struct Server {
    document_root: PathBuf,
    script_name: String,
    addr: SocketAddr,
}

impl Server {
    pub fn new(document_root: PathBuf, script_name: String, addr: SocketAddr) -> Server {
        Server {
            document_root: document_root,
            script_name: script_name,
            addr: addr,
        }
    }
}

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Request) -> Self::Future {
        let doc_root = self.document_root.clone();
        let script_name = self.script_name.clone();
        let addr = self.addr.clone();
        let (method, uri, http_version, headers, body) = request.deconstruct();
        Box::new(body.concat2().and_then(move |chunk| {
            let response = sapi::execute(method, uri, http_version, headers, chunk.as_ref(), doc_root.as_path(), &script_name, &addr);
            futures::future::ok(response)
        }))
    }
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let doc_root = std::env::args().nth(1).unwrap();
    let doc_root = PathBuf::from(doc_root);
    let abs_doc_root = fs::canonicalize(&doc_root).unwrap();
    let index = "index.php"; // TODO make this configurable

    //env_logger::init().expect("Failed to start logger");
    let server = Http::new().bind(&addr, move || Ok(Server::new(abs_doc_root.clone(), index.to_string(), addr.clone()))).unwrap();
    println!("Listening on {}", addr);

    sapi::bootstrap();
    server.run().unwrap();
    sapi::teardown();
}
