use std::io;

use bytes::{BytesMut, BufMut};
use rmpv;
use tokio_io::codec::{Encoder, Decoder};

use errors::DecodeError;
use message::{Message, Request, Response};

pub struct Codec;

impl Decoder for Codec {
    type Item = (u64, Message);
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        let res: Result<Option<Self::Item>, Self::Error>;
        let position = {
            let mut buf = io::Cursor::new(&src);
            loop {
                match Message::decode(&mut buf) {
                    Ok(message) => {
                        res = match message {
                            Message::Request(Request { id, .. }) | Message::Response(Response { id, .. }) => {
                                Ok(Some((id as u64, message)))
                            },
                            Message::Notification(_) => panic!("Notifications not supported"),
                        };
                        break;
                    }
                    Err(err) => {
                        match err {
                            DecodeError::Truncated => return Ok(None),
                            DecodeError::Invalid => continue,
                            DecodeError::UnknownIo(io_err) => {
                                res = Err(io_err);
                                break;
                            }
                        }
                    }
                }
            }
            buf.position() as usize
        };
        let _ = src.split_to(position);
        res
    }
}

impl Encoder for Codec {
    type Item = (u64, Message);
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        let (id, mut message) = item;
        match message {
            Message::Response(ref mut response) => {
                response.id = id as u32;
            }
            Message::Request(ref mut request) => {
                request.id = id as u32;
            }
            Message::Notification(_) => panic!("Notifications not supported"),
        }
        Ok(rmpv::encode::write_value(&mut buf.writer(), &message.as_value())?)
    }
}
