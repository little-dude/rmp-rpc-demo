use std::net::SocketAddr;
use std::io;

use futures::{Future, future};
use rmpv::Value;
use tokio_proto::TcpServer;
use tokio_service::{NewService, Service};

use errors::ServiceError;
use message::{Message, Response, Request};
use protocol::Protocol;

pub trait Handler {
    fn handle_request(&self, method: &str, params: &[Value]) -> Result<Value, Value>;
}

pub struct Server<T: Handler + Clone + Sync + Send + 'static>(T);

impl<T: Handler + Clone + Sync + Send + 'static> Server<T> {
    pub fn new(handler: T) -> Self {
        Server(handler)
    }

    pub fn serve(self, address: SocketAddr) {
        TcpServer::new(Protocol, address).serve(self)
    }
}

impl<T: Handler + Clone + Sync + Send + 'static> Service for Server<T> {
    type Request = Message;
    type Response = Message;
    type Error = ServiceError;
    type Future = Box<Future<Item = Message, Error = ServiceError>>;

    fn call(&self, message: Message) -> Self::Future {
        match message {
            Message::Request( Request { method, params, .. }) => {
                let result = self.0.handle_request(&method, &params);
                let response = Message::Response(Response { id: 0, result: result });
                return Box::new(future::ok(response));
            }
            _ => Box::new(future::err("Unsupported message type".into())),
        }
    }
}

impl<T: Handler + Clone + Sync + Send + 'static> NewService for Server<T> {
    type Request = Message;
    type Response = Message;
    type Error = ServiceError;
    type Instance = Server<T>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(Server(self.0.clone()))
    }
}
