// uncomment for valgrind support
//#![feature(alloc_system)]
//extern crate alloc_system;

extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate php_sys as php;
extern crate clap;
extern crate yansi;
extern crate num_cpus;

use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Arg, App};
use yansi::Paint;

use futures::{Future, Stream};
use futures_cpupool::CpuPool;

use hyper::server::{Http, Request, Response, Service};

pub mod sapi;

/// Server representation
struct Server {
    document_root: PathBuf,
    dir_index: String,
    addr: SocketAddr,
    thread_pool: CpuPool,
}

impl Server {
    pub fn new(document_root: PathBuf, dir_index: String, addr: SocketAddr, thread_pool: CpuPool) -> Server {
        Server {
            document_root: document_root,
            dir_index: dir_index,
            addr: addr,
            thread_pool: thread_pool,
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
        let dir_index = self.dir_index.clone();
        let addr = self.addr.clone();
        let thread_pool = self.thread_pool.clone();
        let (method, uri, http_version, headers, body) = request.deconstruct();
        Box::new(body.concat2().and_then(move |chunk| {
            thread_pool.spawn_fn(move || {
                let response = sapi::execute(method, uri, http_version, headers, chunk.as_ref(), doc_root.as_path(), &dir_index, &addr);
                futures::future::ok(response)
            })
        }))
    }
}

fn main() {

    env_logger::init().expect("Failed to start logger");

    let matches = App::new("php-rpm")
        .arg(
            Arg::with_name("host")
                .long("host")
                .short("h")
                .value_name("host")
                .takes_value(true)
                .help(
                    "Listening ip and port for server. Default: 0.0.0.0:3000",
                ),
        )
        .arg(
            Arg::with_name("index")
                .long("index")
                .short("i")
                .value_name("index")
                .takes_value(true)
                .help(
                    "Name of directory index file. Default: index.php",
                ),
        )
        .arg(
            Arg::with_name("threads")
                .long("threads")
                .short("t")
                .value_name("threads")
                .takes_value(true)
                .help(
                    "Number of threads to use. Default: CPUs * 2",
                ),
        )
        .arg(Arg::with_name("doc_root")
             .help("Path to document root for built-in web server. Default: ./")
             .index(1))
        .get_matches();

    let addr = matches.value_of("host").unwrap_or("0.0.0.0:3000");
    let addr = addr.parse::<SocketAddr>().unwrap();

    let doc_root = matches.value_of("doc_root").unwrap_or("./");
    let doc_root = PathBuf::from(doc_root);
    let abs_doc_root = fs::canonicalize(&doc_root).unwrap();

    let index = matches.value_of("index").unwrap_or("index.php");
    let index = index.to_string();

    let threads = matches.value_of("threads").and_then(|t| t.parse::<usize>().ok()).unwrap_or(num_cpus::get() * 2);
    let thread_pool = CpuPool::new(threads);

    println!("    => address: {}", Paint::white(addr.ip()));
    println!("    => port: {}", Paint::white(addr.port()));
    println!("    => document root: {}", Paint::white(abs_doc_root.display()));
    println!("    => directory index: {}", Paint::white(&index));
    println!("    => threads: {}", Paint::white(threads));
    // TODO: log level, tls, php.ini path

    let server = Http::new().bind(&addr, move || Ok(Server::new(abs_doc_root.clone(), index.clone(), addr.clone(), thread_pool.clone()))).unwrap();

    sapi::bootstrap(threads);
    server.run().unwrap();
    sapi::teardown();
}
