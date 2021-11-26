#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use anyhow::Result;
use bytes::BytesMut;
use core::slice::SlicePattern;
use log::{debug, error, info};
use std::io::BufRead;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ErrorKind};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Sender;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let addr = "3.109.206.34:8021".parse().unwrap();
    let inbound = Inbound::new(addr).await?;
    let _ = inbound.api("reloadxml").await;
    let _ = inbound.api("sofia profile external restart").await;
    let _ = inbound.api("sofia profile external restart").await;
    debug!("finished");
    tokio::time::sleep(Duration::from_secs(10)).await;
    Ok(())
}
pub struct Inbound {
    sender: Arc<Sender<String>>,
    commands: Arc<Mutex<Vec<Sender<String>>>>,
}

struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
}
impl Connection {
    async fn read_frame(&mut self) -> u8 {
        loop {
            let mut buffers = [0; 10];
            let bytes = self.stream.read(&mut buffers[..]).await.unwrap();
            self.buffer.extend_from_slice(&buffers);
            debug!("parsed is {:?} {}", self.buffer, self.buffer.len());
            if self
                .buffer
                .starts_with(b"Content-Type: api/response\nContent-Length: ")
            {
                println!("breaking now");
                break;
            }
        }
        9
    }
}

fn read_bro() {}
impl Inbound {
    pub async fn receive(&mut self) {}
    pub async fn new(socket: SocketAddr) -> Result<Self, tokio::io::Error> {
        let mut stream = tokio::net::TcpStream::connect(socket).await?;
        let (sender, mut receiver) = channel(1);
        let sender = Arc::new(sender);
        let sender_clone = Arc::clone(&sender);
        let commands = Arc::new(Mutex::new(vec![]));
        let inner_commands = Arc::clone(&commands);
        let mut connection = Self { sender, commands };
        let mut buffer = [0; 8096];
        let read = stream.read(&mut buffer).await;
        let _ = stream.write(b"auth ClueCon\n\n").await;
        let read = stream.read(&mut buffer).await;
        let mut io = Connection {
            stream,
            buffer: BytesMut::new(),
        };
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    frame = receiver.recv() => {
                        if let Some(message) = frame {
                            debug!("writing command : {}",message);
                            let _ = io.stream.write(message.as_bytes()).await;
                        }
                    },
                    bytes = io.read_frame() => {

                    }
                }
            }
        });
        Ok(connection)
    }
    async fn api(&self, command: &str) -> Result<()> {
        debug!("Send api");
        self.sender.send(format!("api {}\n\n", command)).await?;
        let (sender, mut receiver) = channel(10);
        self.commands.lock().await.push(sender);
        // commands.push(sender);
        if let Some(a) = receiver.recv().await {
            debug!("received data from channel: {}", a);
            Ok(())
        } else {
            Ok(())
        }
    }
}
