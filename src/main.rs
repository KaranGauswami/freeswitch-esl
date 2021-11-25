#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use anyhow::Result;
use log::{debug, error, info};
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
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    frame = receiver.recv() => {
                        if let Some(message) = frame {
                            debug!("writing command : {}",message);
                            let _ = stream.write(message.as_bytes()).await;
                        }
                    },
                    bytes = stream.read(&mut buffer) => {
                        match bytes{
                            Ok(n) => {
                                debug!("received data: {:?} {:?}",n,String::from_utf8_lossy(&buffer[0..n]));
                                let mut commands = inner_commands.lock().await;
                                // TODO: Tomorrow store all this data in main io struct and take reference from zookeeper_async => handle_response method
                                if let Some(tx) = commands.pop(){
                                    debug!("sending channel data: {:?}",n);
                                    let _ = tx.send(n.to_string()).await;
                                }
                            },
                            Err(_)=> {error!("Error in reading")}
                        }
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
