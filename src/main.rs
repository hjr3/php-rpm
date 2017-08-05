extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;

use hyper::server::{Http, Request, Response, Service};

pub mod sapi;

struct Server {
    document_root: String,
}

impl Server {
    pub fn new(document_root: String) -> Server {
        Server {
            document_root: document_root,
        }
    }
}

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = futures::future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, request: Request) -> Self::Future {
        let response = sapi::execute(request, &self.document_root);
        futures::future::ok(response)
    }
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let filename = std::env::args().nth(1).unwrap();

    let server = Http::new().bind(&addr, move || Ok(Server::new(filename.to_string()))).unwrap();
    println!("Listening on {}", addr);

    sapi::bootstrap();
    server.run().unwrap();
    sapi::teardown();
}
