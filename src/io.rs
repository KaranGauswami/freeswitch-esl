use std::collections::HashMap;

use anyhow::Result;
use bytes::Buf;
use log::{debug, error, info};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone)]
pub struct EslCodec {}

impl Encoder<&[u8]> for EslCodec {
    type Error = tokio::io::Error;
    fn encode(&mut self, item: &[u8], dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(item);
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum InboundResponse {
    Auth,
    Reply(String),
    ApiResponse(String),
}
fn get_header_end(src: &bytes::BytesMut) -> Option<usize> {
    debug!("get_header_end:=>{:?}", src);
    // get first new line character
    for (index, chat) in src[..].iter().enumerate() {
        if chat == &b'\n' && src.get(index + 1) == Some(&b'\n') {
            return Some(index + 1);
        }
    }
    None
}
fn parse_body(src: &[u8], length: usize) -> String {
    info!("parse body src : {}", String::from_utf8_lossy(src));
    info!("length src : {}", length);
    String::from_utf8_lossy(&src[2..length + 1]).to_string()
}
fn parse_header(src: &[u8]) -> Result<HashMap<String, String>> {
    debug!("parsing this header {:#?}", String::from_utf8_lossy(src));
    let data = String::from_utf8_lossy(src).to_string();
    let a = data.split('\n');
    let mut hash = HashMap::new();
    for line in a {
        let mut key_value = line.split(':');
        let key = key_value.next().unwrap().trim().to_string();
        let val = key_value.next().unwrap().trim().to_string();
        hash.insert(key, val);
    }
    debug!("returning hashmap : {:?}", hash);
    Ok(hash)
}
impl Decoder for EslCodec {
    type Item = InboundResponse;
    type Error = anyhow::Error;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        debug!("decode");
        let header_end = get_header_end(src);
        if let Some(header_end) = header_end {
            let headers = parse_header(&src[..(header_end - 1)])?;
            if let Some(somes) = headers.get("Content-Type") {
                match somes.as_str() {
                    "auth/request" => {
                        src.advance(src.len());
                        debug!("returned auth");
                        return Ok(Some(InboundResponse::Auth {}));
                    }
                    "api/response" => {
                        if let Some(body_length) = headers.get("Content-Length") {
                            let body_length = body_length.parse()?;
                            let body = parse_body(&src[header_end..], body_length);
                            error!("advancing");
                            error!("src is {}", String::from_utf8_lossy(src));
                            src.advance(src.len());
                            debug!("returned api/response");
                            return Ok(Some(InboundResponse::ApiResponse(body)));
                        } else {
                            panic!("content_length not found");
                        }
                    }
                    "command/reply" => {
                        let response = String::from_utf8_lossy(src).to_string();
                        info!("{}", response);
                        src.advance(src.len());
                        info!("returned command/reply");
                        return Ok(Some(InboundResponse::Reply(response)));
                    }
                    "text/event-json" => {
                        if let Some(body_length) = headers.get("Content-Length") {
                            let body_length = body_length.parse()?;
                            let body = parse_json_body(&src[header_end..], body_length)?;
                            error!("{:?}", body);
                            let body = format!("{:?}", body);
                            src.advance(src.len());
                            debug!("returned api/response");
                            return Ok(Some(InboundResponse::ApiResponse(body)));
                        } else {
                            panic!("content_length not found");
                        }
                    }
                    _ => {
                        info!("content-type {}", somes.as_str());
                        panic!("not handled")
                    }
                }
            }
            panic!("should not reach here {:?}", headers);
        } else {
            debug!("when header is not there {:?}", src);
            Ok(None)
        }
    }
}

fn parse_json_body(src: &[u8], body_length: usize) -> Result<HashMap<String, String>> {
    let body = String::from_utf8_lossy(&src[2..body_length + 2]);
    Ok(serde_json::from_str(&body)?)
}
