use crate::io::{EslCodec, InboundResponse};
use anyhow::Result;
use futures::SinkExt;
use log::debug;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::oneshot::channel;
use tokio::sync::oneshot::Sender;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
pub struct Inbound {
    commands: Arc<Mutex<VecDeque<Sender<InboundResponse>>>>,
    transport: Arc<Mutex<FramedWrite<OwnedWriteHalf, EslCodec>>>,
}

impl Inbound {
    pub async fn send_recv(&self, item: &[u8]) -> Result<InboundResponse> {
        let mut transport = self.transport.lock().await;
        let _ = transport.send(item).await?;
        let (tx, rx) = channel();
        self.commands.lock().await.push_back(tx);
        if let Ok(data) = rx.await {
            Ok(data)
        } else {
            Err(anyhow::anyhow!("send_recv failed"))
        }
    }
    pub async fn new(socket: SocketAddr) -> Result<Self, tokio::io::Error> {
        let stream = TcpStream::connect(socket).await?;
        // let sender = Arc::new(sender);
        let commands = Arc::new(Mutex::new(VecDeque::new()));
        let inner_commands = Arc::clone(&commands);
        let my_coded = EslCodec {};
        let (read_half, write_half) = stream.into_split();
        let mut transport_rx = FramedRead::new(read_half, my_coded.clone());
        let transport_tx = Arc::new(Mutex::new(FramedWrite::new(write_half, my_coded.clone())));
        let _ = transport_rx.next().await;
        println!("recv event: BEFORE");
        let connection = Self {
            commands,
            transport: transport_tx,
        };
        tokio::spawn(async move {
            loop {
                let something = transport_rx.next().await;
                if let Some(Ok(event)) = something {
                    if let InboundResponse::EventJson(x) = event {
                        debug!("continued");
                        continue;
                    }
                    if let Some(tx) = inner_commands.lock().await.pop_front() {
                        let _ = tx.send(event).expect("msg");
                    }
                }
            }
        });
        let _ = connection.send_recv(b"auth ClueCon\n\n").await;
        let _ = connection
            .send_recv(b"event json BACKGROUND_JOB CHANNEL_EXECUTE_COMPLETE\n\n")
            .await;
        Ok(connection)
    }
    pub async fn api(&self, command: &str) -> Result<InboundResponse> {
        self.send_recv(command.as_bytes()).await
    }
    pub async fn bgapi(&self, command: &str) -> Result<InboundResponse> {
        debug!("Send bgapi {}", command);
        let job_uuid = uuid::Uuid::new_v4().to_string();

        self.send_recv(format!("bgapi {}\nJob-UUID: {}\n\n", command, job_uuid).as_bytes())
            .await
    }
}
