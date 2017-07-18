extern crate rmpv;
extern crate rmp_rpc_demo;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate futures;

use std::thread;
use std::time::Duration;

use tokio_core::reactor::Core;
use rmpv::Value;
use futures::Future;
use rmp_rpc_demo::client::Client;
use rmp_rpc_demo::server::{Server, Handler};

#[derive(Clone)]
struct CalculatorServer;

impl Handler for CalculatorServer {
    fn handle_request(&self, method: &str, params: &[Value]) -> Result<Value, Value> {
        if params.len() != 2 {
            return Err("Expected two arguments".into());
        }
        if !params[0].is_i64() || !params[1].is_i64() {
            return Err("Invalid argument".into());
        }
        let res = match method {
            "add" => params[0].as_i64().unwrap() + params[1].as_i64().unwrap(),
            "sub" => params[0].as_i64().unwrap() - params[1].as_i64().unwrap(),
            _ => return Err("Unknown method".into()),
        };
        Ok(res.into())
    }
}

fn main() {
    let addr = "127.0.0.1:12345".parse().unwrap();

    thread::spawn(move || {
        Server::new(CalculatorServer {}).serve(addr);
    });

    thread::sleep(Duration::from_millis(100));

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let requests = Client::connect(&addr, &handle)
        .and_then(|client| {
            client.request("add", vec![1.into(), 2.into()])
                .and_then(move |response| {
                    println!("{:?}", response);
                    client.request("sub", vec![2.into(), 1.into()])
                })
                .and_then(|response| {
                    println!("{:?}", response);
                    Ok(())
                })
        });
    let _ = core.run(requests);
}
