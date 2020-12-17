use anyhow::Result;
use regex::Regex;
use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream};
#[derive(Debug, Clone)]
pub struct ApiResponse {
    content_type: String,
    content_length: usize,
    body: String,
}
impl ApiResponse {
    fn new(content_type: String, content_length: usize, body: String) -> Self {
        Self {
            content_type,
            content_length,
            body,
        }
    }
}
pub struct FreeswitchESL {
    stream: TcpStream,
}
impl FreeswitchESL {
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
        Ok(Self { stream })
    }
    pub fn api(&mut self, command: &str) -> Result<ApiResponse> {
        // Send api command
        let command = format!("api {}\n\n", command);
        self.stream.write(&command.as_bytes())?;

        // read headers
        let mut buffer = [0; 64];
        self.stream.read(&mut buffer)?;
        let result = parse(&buffer);

        // read content-type and content-length from header
        let re = Regex::new(r"Content-Type: ([a-z//]+)\nContent-Length: (\d+)")?;
        let cap = re.captures(&result).unwrap();
        let content_type = cap.get(1).unwrap().as_str().to_owned();
        let content_length = cap.get(2).unwrap().as_str().parse::<usize>()?;

        // read reponse based on content-length
        let mut buf = vec![0; content_length];
        let _ = self.stream.read_exact(&mut buf)?;
        let response = String::from_utf8(buf)?;
        Ok(ApiResponse::new(content_type, content_length, response))
    }
}
fn parse(buf: &[u8]) -> String {
    let parsed = String::from_utf8_lossy(&buf).to_string();
    parsed
}
