use crate::io::{EslCodec, InboundResponse};
use anyhow::Result;
use futures::SinkExt;
use log::{debug, info};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
pub struct Inbound {
    sender: Arc<Sender<String>>,
    commands: Arc<Mutex<Vec<Option<Sender<InboundResponse>>>>>,
}

impl Inbound {
    pub async fn receive(&mut self) {}
    pub async fn auth(&self, auth_command: &[u8]) -> Result<()> {
        info!("Send auth {:?}", auth_command);
        self.sender
            .send(String::from_utf8_lossy(auth_command).to_string())
            .await?;
        // let (sender, _receiver) = channel(10);
        // let (sender2, _receiver) = channel(10);
        // self.commands.lock().await.push(Some(sender));
        // self.commands.lock().await.push(Some(sender2));
        Ok(())
    }
    pub async fn new(socket: SocketAddr) -> Result<Self, tokio::io::Error> {
        let mut stream = TcpStream::connect(socket).await?;
        let (sender, mut receiver) = channel(1);
        let sender = Arc::new(sender);
        let commands = Arc::new(Mutex::new(vec![]));
        let inner_commands = Arc::clone(&commands);
        let connection = Self { sender, commands };
        let my_coded = EslCodec {};
        let mut buffer = [0; 1024];
        let _ = stream.read(&mut buffer[..]).await;
        let _ = stream.write(b"auth ClueCon\n\n").await;
        let _ = stream.read(&mut buffer[..]).await;
        let mut transport = Framed::new(stream, my_coded);
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
                        let event = something;
                        if let Some(Ok(event)) = event{
                            match event {
                                InboundResponse::Auth => {
                                    debug!("got auth");
                                    let _ = transport.send("auth ClueCon\n\n".to_string()).await;
                                    inner_commands.lock().await.push(None);
                                }
                                InboundResponse::Reply(n) => {
                                    debug!("got reply {}", n);
                                    if let Some(tx) = inner_commands.lock().await.pop().unwrap(){
                                        let _ = tx.send(InboundResponse::Reply(n.clone())).await;
                                        debug!("send channel data for {}",n);
                                    }
                                }
                                InboundResponse::ApiResponse(n) => {
                                    debug!("got api response {}", n);
                                    if let Some(tx) = inner_commands.lock().await.pop().unwrap(){
                                        let _ = tx.send(InboundResponse::ApiResponse(n.clone())).await;
                                        debug!("send channel data for {}",n);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
        // connection.auth(b"auth ClueCon\n\n").await;
        Ok(connection)
    }
    pub async fn api(&self, command: &str) -> Result<InboundResponse> {
        info!("Send api {}", command);
        self.sender.send(format!("api {}\n\n", command)).await?;
        let (sender, mut receiver) = channel(10);
        self.commands.lock().await.push(Some(sender));
        // commands.push(sender);
        if let Some(a) = receiver.recv().await {
            debug!("received data from channel: {:?}", a);
            Ok(a)
        } else {
            Err(anyhow::anyhow!("key"))
        }
    }
}
