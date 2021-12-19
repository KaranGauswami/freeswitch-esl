use crate::io::{EslCodec, InboundResponse};
use anyhow::Result;
use futures::SinkExt;
use log::debug;
use log::error;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::net::ToSocketAddrs;
use tokio::sync::oneshot::channel;
use tokio::sync::oneshot::Sender;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
pub struct Inbound {
    password: String,
    commands: Arc<Mutex<VecDeque<Sender<InboundResponse>>>>,
    transport_rx: Arc<Mutex<FramedWrite<OwnedWriteHalf, EslCodec>>>,
    background_jobs: Arc<Mutex<HashMap<String, Sender<InboundResponse>>>>,
}

impl Inbound {
    pub async fn send_recv(&self, item: &[u8]) -> Result<InboundResponse> {
        let mut transport = self.transport_rx.lock().await;
        let _ = transport.send(item).await?;
        let (tx, rx) = channel();
        self.commands.lock().await.push_back(tx);
        if let Ok(data) = rx.await {
            Ok(data)
        } else {
            Err(anyhow::anyhow!("send_recv failed"))
        }
    }
    pub async fn new(
        socket: impl ToSocketAddrs,
        password: impl ToString,
    ) -> Result<Self, tokio::io::Error> {
        let stream = TcpStream::connect(socket).await?;
        // let sender = Arc::new(sender);
        let commands = Arc::new(Mutex::new(VecDeque::new()));
        let inner_commands = Arc::clone(&commands);
        let background_jobs = Arc::new(Mutex::new(HashMap::new()));
        let inner_background_jobs = Arc::clone(&background_jobs);
        let my_coded = EslCodec {};
        let (read_half, write_half) = stream.into_split();
        let mut transport_rx = FramedRead::new(read_half, my_coded.clone());
        let transport_tx = Arc::new(Mutex::new(FramedWrite::new(write_half, my_coded.clone())));
        let _ = transport_rx.next().await;
        let connection = Self {
            password: password.to_string(),
            commands,
            background_jobs,
            transport_rx: transport_tx,
        };
        tokio::spawn(async move {
            loop {
                if let Some(Ok(event)) = transport_rx.next().await {
                    if let InboundResponse::EventJson(data) = &event {
                        let my_hash_map: HashMap<String, String> =
                            serde_json::from_str(data).unwrap();
                        let job_uuid = my_hash_map.get("Job-UUID");
                        if let Some(job_uuid) = job_uuid {
                            if let Some(tx) = inner_background_jobs.lock().await.remove(job_uuid) {
                                error!("sending message in bgapi channel");
                                let _ = tx.send(event).unwrap();
                            }
                            debug!("continued");
                        }
                        continue;
                    }
                    if let Some(tx) = inner_commands.lock().await.pop_front() {
                        let _ = tx.send(event).expect("msg");
                    }
                }
            }
        });
        let _ = connection
            .send_recv(format!("auth {}\n\n", connection.password).as_bytes())
            .await;
        let _ = connection
            .send_recv(b"event json BACKGROUND_JOB CHANNEL_EXECUTE_COMPLETE\n\n")
            .await;
        Ok(connection)
    }
    pub async fn api(&self, command: &str) -> Result<InboundResponse> {
        self.send_recv(format!("api {}\n\n", command).as_bytes())
            .await
    }
    pub async fn bgapi(&self, command: &str) -> Result<InboundResponse> {
        debug!("Send bgapi {}", command);
        let job_uuid = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = channel();
        self.background_jobs
            .lock()
            .await
            .insert(job_uuid.clone(), tx);

        self.send_recv(format!("bgapi {}\nJob-UUID: {}\n\n", command, job_uuid).as_bytes())
            .await?;

        if let Ok(resp) = rx.await {
            Ok(resp)
        } else {
            Err(anyhow::anyhow!("error in receiving bgapi"))
        }
    }
}
