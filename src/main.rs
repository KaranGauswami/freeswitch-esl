use anyhow::Result;
use futures::SinkExt;
use log::debug;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

mod io;

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

use crate::io::{Event, MyCodc};

impl Inbound {
    pub async fn receive(&mut self) {}
    pub async fn new(socket: SocketAddr) -> Result<Self, tokio::io::Error> {
        let stream = TcpStream::connect(socket).await?;
        let (sender, mut receiver) = channel(1);
        let sender = Arc::new(sender);
        let commands = Arc::new(Mutex::new(vec![]));
        let inner_commands = Arc::clone(&commands);
        let connection = Self { sender, commands };
        let my_coded = MyCodc::new();
        let mut transport = Framed::new(stream, my_coded);
        debug!("will read one frame");
        let _ = transport.next().await.unwrap().unwrap();
        debug!("read one frame");
        let _ = transport.send("auth ClueCon\n\n".to_string()).await;
        let _ = transport.next().await.unwrap().unwrap();
        tokio::spawn(async move {
            loop {
                tokio::select! {


                    frame = receiver.recv() => {
                        if let Some(message) = frame {
                            debug!("writing command : {}",message);
                            let _ = transport.send(message).await;
                        }
                    },
                    something = transport.next() => {
                        match something.unwrap().unwrap() {
                            Event::Auth => {
                                debug!("got auth");
                                let _ = transport.send("auth ClueCon\n\n".to_string()).await;
                            }
                            Event::Reply(n) => {
                                let tx = inner_commands.lock().await.pop().unwrap();
                                debug!("got reply {}", n);
                                let _ = tx.send(n.clone()).await;
                                debug!("send channel data for {}",n);
                            }
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
