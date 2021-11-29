use std::collections::HashMap;

use anyhow::Result;
use bytes::Buf;
use log::{debug, error, info};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone)]
pub struct EslCodec {}

impl Encoder<String> for EslCodec {
    type Error = tokio::io::Error;
    fn encode(&mut self, item: String, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        debug!("self {}", item);
        dst.extend_from_slice(item.as_bytes());
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub enum InboundResponse {
    Auth,
    Reply(String),
    ApiResponse(String),
}
fn get_header_end(src: &bytes::BytesMut) -> Option<usize> {
    info!("get_header_end:=>{:?}", src);
    // get first new line character
    for (index, chat) in src[..].iter().enumerate() {
        if chat == &b'\n' && src.get(index + 1) == Some(&b'\n') {
            return Some(index);
        }
    }
    None
}
fn parse_body(src: &[u8], length: usize) -> String {
    info!("parsing this body {:?}", String::from_utf8_lossy(src));
    info!(
        "returning this body {:?}",
        String::from_utf8_lossy(&src[2..length + 1])
    );
    String::from_utf8_lossy(&src[2..length + 1]).to_string()
}
fn parse_header(src: &[u8]) -> HashMap<String, String> {
    info!("parsing this header {:#?}", String::from_utf8_lossy(src));
    let data = String::from_utf8_lossy(src).to_string();
    let a = data.split('\n');
    let mut hash = HashMap::new();
    for line in a {
        let mut key_value = line.split(':');
        let key = key_value.next().unwrap().trim().to_string();
        let val = key_value.next().unwrap().trim().to_string();
        hash.insert(key, val);
    }
    info!("returning hashmap : {:?}", hash);
    hash
}
impl Decoder for EslCodec {
    type Item = InboundResponse;
    type Error = anyhow::Error;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        debug!("decode");
        let newline = get_header_end(src);
        if let Some(x) = newline {
            info!("header end is {:?}", newline);
            let headers = parse_header(&src[..x]);
            info!("current remaining {:?}", String::from_utf8_lossy(&src[x..]));
            if let Some(somes) = headers.get("Content-Type") {
                match somes.as_str() {
                    "auth/request" => {
                        src.advance(src.len());
                        info!("returned auth");
                        return Ok(Some(InboundResponse::Auth {}));
                    }
                    "api/response" => {
                        if let Some(body_length) = headers.get("Content-Length") {
                            let body_length = body_length.parse().unwrap();
                            let body = parse_body(&src[x..], body_length);
                            error!("{}", String::from_utf8_lossy(&src[..]));
                            src.advance(src.len());
                            info!("returned api/response");
                            return Ok(Some(InboundResponse::ApiResponse(body)));
                        } else {
                            panic!("content_length not found");
                        }
                    }
                    "command/reply" => {
                        src.advance(src.len());
                        info!("returned command/reply");
                        return Ok(Some(InboundResponse::Reply("OK".to_string())));
                    }
                    _ => {
                        panic!("not handled")
                    }
                }
            }
            panic!("should not reach here");
        } else {
            info!("when header is not there {:?}", src);
            Ok(None)
        }
    }
}
