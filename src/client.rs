use std::io;
use std::net::SocketAddr;

use futures::Future;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_proto::multiplex::ClientService;
use tokio_proto::TcpClient;
use tokio_service::Service;
use rmpv::Value;

use message::{Message, Request};
use protocol::Protocol;


pub struct Client(ClientService<TcpStream, Protocol>);

pub type Response = Box<Future<Item = Result<Value, Value>, Error = io::Error>>;

impl Client {
    pub fn connect(addr: &SocketAddr, handle: &Handle) -> Box<Future<Item = Client, Error = io::Error>> {
        let ret = TcpClient::new(Protocol)
            .connect(addr, handle)
            .map(Client);
        Box::new(ret)
    }

    pub fn request(&self, method: &str, params: Vec<Value>) -> Response {
        let req = Message::Request(Request {
            // we can set this to 0 because under the hood it's handle by tokio at the
            // protocol/codec level
            id: 0,
            method: method.to_string(),
            params: params,
        });
        let resp = self.0.call(req).and_then(|resp| {
            match resp {
                Message::Response(response) => Ok(response.result),
                _ => panic!("Response is not a Message::Response"),
            }
        });
        Box::new(resp) as Response
    }
}
