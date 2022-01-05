use std::collections::HashMap;

use bytes::Buf;
use log::trace;
use serde_json::Value;
use tokio_util::codec::{Decoder, Encoder};

use crate::{event::Event, EslError};

#[derive(Debug, Clone)]
pub struct EslCodec {}

impl Encoder<&[u8]> for EslCodec {
    type Error = EslError;
    fn encode(&mut self, item: &[u8], dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(item);
        dst.extend_from_slice(b"\n\n");
        Ok(())
    }
}

fn get_header_end(src: &bytes::BytesMut) -> Option<usize> {
    trace!("get_header_end:=>{:?}", src);
    // get first new line character
    for (index, chat) in src[..].iter().enumerate() {
        if chat == &b'\n' && src.get(index + 1) == Some(&b'\n') {
            return Some(index + 1);
        }
    }
    None
}
fn parse_body(src: &[u8], length: usize) -> String {
    trace!("parse body src : {}", String::from_utf8_lossy(src));
    trace!("length src : {}", length);
    String::from_utf8_lossy(&src[..length]).to_string()
}
fn parse_header(src: &[u8]) -> Result<HashMap<String, Value>, std::io::Error> {
    trace!("parsing this header {:#?}", String::from_utf8_lossy(src));
    let data = String::from_utf8_lossy(src).to_string();
    let a = data.split('\n');
    let mut hash = HashMap::new();
    for line in a {
        let mut key_value = line.split(':');
        let key = key_value.next().unwrap().trim().to_string();
        let val = key_value.next().unwrap().trim().to_string();
        hash.insert(key, serde_json::json!(val));
    }
    trace!("returning hashmap : {:?}", hash);
    Ok(hash)
}

impl Decoder for EslCodec {
    type Item = Event;
    type Error = EslError;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        trace!("decode");
        let header_end = get_header_end(src);
        if header_end.is_none() {
            return Ok(None);
        }
        let header_end = header_end.unwrap();
        let headers = parse_header(&src[..(header_end - 1)])?;
        trace!("parsed headers are : {:?}", headers);
        let body_start = header_end + 1;
        if let Some(length) = headers.get("Content-Length") {
            let length = length.as_str().unwrap();
            let body_length = length.parse()?;
            if src.len() < (header_end + body_length + 1) {
                trace!("returned because size was not enough");
                return Ok(None);
            }
            let body = parse_body(&src[body_start..], body_length);
            src.advance(body_start + body_length);
            Ok(Some(Event {
                headers,
                body: Some(body),
            }))
        } else {
            src.advance(body_start);
            Ok(Some(Event {
                headers,
                body: None,
            }))
        }
    }
}
