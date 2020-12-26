use anyhow::Result;
use regex::Regex;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
#[derive(Debug, Clone)]
pub struct ResponseHeaders {
    content_type: String,
    content_length: usize,
}
impl ResponseHeaders {
    fn new(content_type: String, content_length: usize) -> Self {
        Self {
            content_type,
            content_length,
        }
    }
    pub fn content_type(&self) -> &String {
        &self.content_type
    }
    pub fn content_length(&self) -> usize {
        self.content_length
    }
}
#[derive(Debug, Clone)]
pub struct ApiResponse {
    headers: ResponseHeaders,
    body: String,
}
impl ApiResponse {
    fn new(headers: ResponseHeaders, body: String) -> Self {
        Self { headers, body }
    }
    pub fn headers(&self) -> &ResponseHeaders {
        &self.headers
    }
    pub fn body(&self) -> &String {
        &self.body
    }
}
pub struct OutboundConn {
    stream: Arc<Mutex<TcpStream>>,
}
impl OutboundConn {
    pub fn new(addr: SocketAddr, passwd: &str) -> Result<Self> {
        // Connect to ESL
        let mut stream = TcpStream::connect(addr)?;
        let auth_command = format!("auth {}\n\n", passwd);
        let mut buf = [0; 128];

        // Read auth/request
        stream.read(&mut buf)?;

        // Sending Password
        stream.write(auth_command.as_bytes())?;
        stream.read(&mut buf)?;

        // stream. write("event json all\n\n".as_bytes())?;
        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
        })
    }
    pub fn api(&self, command: &str) -> Result<ApiResponse> {
        // Send api command
        let command = format!("api {}\n\n", command);
        let mut stream_lock = self.stream.lock().unwrap();
        stream_lock.write(&command.as_bytes())?;

        // read headers
        let mut buffer = [0; 64];
        stream_lock.read(&mut buffer)?;
        let result = parse(&buffer);

        // read content-type and content-length from header
        let re = Regex::new(r"Content-Type: ([a-z//]+)\nContent-Length: (\d+)")?;
        let cap = re.captures(&result).unwrap();
        let content_type = cap.get(1).unwrap().as_str().to_owned();
        let content_length = cap.get(2).unwrap().as_str().parse::<usize>()?;

        // read response based on content-length
        let mut buf = vec![0; content_length];
        let _ = stream_lock.read_exact(&mut buf)?;
        std::mem::drop(stream_lock);
        let response = String::from_utf8(buf)?;
        let headers = ResponseHeaders::new(content_type, content_length);
        Ok(ApiResponse::new(headers, response))
    }
}
fn parse(buf: &[u8]) -> String {
    let parsed = String::from_utf8_lossy(&buf).to_string();
    parsed
}
