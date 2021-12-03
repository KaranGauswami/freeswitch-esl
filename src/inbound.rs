use crate::io::{EslCodec, InboundResponse};
use anyhow::Result;
use futures::SinkExt;
use log::debug;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::oneshot::channel as oneshot_channel;
use tokio::sync::oneshot::Sender as OneShotSender;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
pub struct Inbound {
    sender: Sender<String>,
    commands: Arc<Mutex<VecDeque<Option<OneShotSender<InboundResponse>>>>>,
}

impl Inbound {
    pub async fn new(socket: SocketAddr) -> Result<Self, tokio::io::Error> {
        let stream = TcpStream::connect(socket).await?;
        let (sender, mut receiver) = channel(10);
        // let sender = Arc::new(sender);
        let commands = Arc::new(Mutex::new(VecDeque::new()));
        let inner_commands = Arc::clone(&commands);
        let connection = Self { sender, commands };
        let my_coded = EslCodec {};
        let mut transport = Framed::new(stream, my_coded);
        let event = transport.next().await.unwrap().unwrap();
        if InboundResponse::Auth == event {
            let _ = transport.send(b"auth ClueCon\n\n").await;
            transport.next().await;
        }
        let _ = transport
            .send(b"event json BACKGROUND_JOB CHANNEL_EXECUTE_COMPLETE\n\n")
            .await;
        transport.next().await;
        tokio::spawn(async move {
            loop {
                tokio::select! {

                    frame = receiver.recv() => {
                        if let Some(message) = frame {
                            debug!("writing command : {}",message);
                            let _ = transport.send(message.as_bytes()).await;
                        }
                    },
                    something = transport.next() => {
                        let event = something;
                        if let Some(Ok(event)) = event{
                            match event {
                                InboundResponse::Auth => {
                                    debug!("got auth");
                                    let _ = transport.send(b"auth ClueCon\n\n").await;
                                    inner_commands.lock().await.push_back(None);
                                }
                                InboundResponse::Reply(n) => {
                                    debug!("got reply {}", n);
                                    if let Some(Some(tx)) = inner_commands.lock().await.pop_front(){
                                        let _ = tx.send(InboundResponse::Reply(n.clone()));
                                        debug!("send channel data for {}",n);
                                    }
                                }
                                InboundResponse::ApiResponse(n) => {
                                    debug!("got api response {}", n);
                                    if let Some(Some(tx)) = inner_commands.lock().await.pop_front(){
                                        let _ = tx.send(InboundResponse::ApiResponse(n.clone()));
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
        debug!("Send api {}", command);
        self.sender.send(format!("api {}\n\n", command)).await?;
        let (sender, receiver) = oneshot_channel();
        self.commands.lock().await.push_back(Some(sender));

        if let Ok(a) = receiver.await {
            debug!("received data from channel: {:?}", a);
            Ok(a)
        } else {
            Err(anyhow::anyhow!("key"))
        }
    }
    pub async fn bgapi(&self, command: &str) -> Result<InboundResponse> {
        debug!("Send bgapi {}", command);
        let job_uuid = uuid::Uuid::new_v4().to_string();

        self.sender
            .send(format!("bgapi {}\nJob-UUID: {}\n\n", command, job_uuid))
            .await?;
        let (sender, receiver) = oneshot_channel();
        self.commands.lock().await.push_back(Some(sender));
        // commands.push(sender);
        if let Ok(a) = receiver.await {
            debug!("received data from channel: {:?}", a);
            Ok(a)
        } else {
            Err(anyhow::anyhow!("key"))
        }
    }
}
