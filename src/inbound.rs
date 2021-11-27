use crate::io::{InboundResponse, MyCodc};
use anyhow::Result;
use futures::SinkExt;
use log::debug;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
pub struct Inbound {
    sender: Arc<Sender<String>>,
    commands: Arc<Mutex<Vec<Sender<InboundResponse>>>>,
}

impl Inbound {
    pub async fn receive(&mut self) {}
    pub async fn new(socket: SocketAddr) -> Result<Self, tokio::io::Error> {
        let stream = TcpStream::connect(socket).await?;
        let (sender, mut receiver) = channel(1);
        let sender = Arc::new(sender);
        let commands = Arc::new(Mutex::new(vec![]));
        let inner_commands = Arc::clone(&commands);
        let connection = Self { sender, commands };
        let my_coded = MyCodc {};
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
                        let event = something.unwrap().unwrap();
                        match event {
                            InboundResponse::Auth => {
                                debug!("got auth");
                                let _ = transport.send("auth ClueCon\n\n".to_string()).await;
                            }
                            InboundResponse::Reply(n) => {
                                let tx = inner_commands.lock().await.pop().unwrap();
                                debug!("got reply {}", n);
                                let _ = tx.send(InboundResponse::Reply(n.clone())).await;
                                debug!("send channel data for {}",n);
                            }
                            InboundResponse::ApiResponse(n) => {
                                let tx = inner_commands.lock().await.pop().unwrap();
                                debug!("got api response {}", n);
                                let _ = tx.send(InboundResponse::ApiResponse(n.clone())).await;
                                debug!("send channel data for {}",n);
                            }
                        }
                    }
                }
            }
        });
        Ok(connection)
    }
    pub async fn api(&self, command: &str) -> Result<()> {
        debug!("Send api");
        self.sender.send(format!("api {}\n\n", command)).await?;
        let (sender, mut receiver) = channel(10);
        self.commands.lock().await.push(sender);
        // commands.push(sender);
        if let Some(a) = receiver.recv().await {
            debug!("received data from channel: {:?}", a);
            Ok(())
        } else {
            Ok(())
        }
    }
}
