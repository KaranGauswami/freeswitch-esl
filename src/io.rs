use anyhow::Result;
use bytes::Buf;
use log::debug;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone)]
pub struct MyCodc {}

impl Encoder<String> for MyCodc {
    type Error = tokio::io::Error;
    fn encode(&mut self, item: String, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        debug!("self {}", item);
        dst.extend_from_slice(item.as_bytes());
        return Ok(());
    }
}
#[derive(Debug, Clone)]
pub enum InboundResponse {
    Auth,
    Reply(String),
    ApiResponse(String),
}
impl Decoder for MyCodc {
    type Item = InboundResponse;
    type Error = anyhow::Error;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let a = b"Content-Type: api/response\nContent-Length:";
        debug!("src is {:?}", src);
        if src.starts_with(b"Content-Type: auth/request\n\n") {
            src.advance(src.len());
            Ok(Some(InboundResponse::Auth {}))
        } else if src.starts_with(b"Content-Type: command/reply\nReply-Text: +OK accepted\n") {
            let sts = String::from_utf8(src.to_vec());
            src.advance(src.len());
            Ok(Some(InboundResponse::Reply(sts.unwrap())))
        } else if src.starts_with(a) {
            src.advance(src.len());
            let sts = String::from_utf8(src.to_vec());
            Ok(Some(InboundResponse::ApiResponse(sts.unwrap())))
        } else {
            // src.advance(src.len());
            debug!("{:?}", src);
            Ok(None)
        }
    }
}
