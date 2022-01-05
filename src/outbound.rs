use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use futures::{SinkExt, StreamExt};
use log::trace;
use serde_json::Value;
use tokio::{
    net::{tcp::OwnedWriteHalf, TcpListener, ToSocketAddrs},
    sync::{
        oneshot::{channel, Sender},
        Mutex,
    },
};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::{event::Event, io::EslCodec, EslError};

pub struct Outbound {
    listener: TcpListener,
}
impl Outbound {
    pub async fn bind(addr: impl ToSocketAddrs) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self { listener })
    }
    pub async fn accept(&self) -> Result<(OutboundSession, SocketAddr), EslError> {
        let (stream, addr) = self.listener.accept().await?;
        let commands = Arc::new(Mutex::new(VecDeque::new()));
        let inner_commands = Arc::clone(&commands);
        let background_jobs = Arc::new(Mutex::new(HashMap::new()));
        let inner_background_jobs = Arc::clone(&background_jobs);
        let esl_codec = EslCodec {};
        let (read_half, write_half) = stream.into_split();
        let mut transport_rx = FramedRead::new(read_half, esl_codec.clone());
        let transport_tx = Arc::new(Mutex::new(FramedWrite::new(write_half, esl_codec.clone())));
        let mut connection = OutboundSession {
            call_uuid: None,
            connection_info: HashMap::new(),
            commands,
            background_jobs,
            transport_tx,
            connected: AtomicBool::new(false),
        };
        tokio::spawn(async move {
            loop {
                if let Some(Ok(event)) = transport_rx.next().await {
                    if let Some(types) = event.headers.get("Content-Type") {
                        if types == "text/event-json" {
                            trace!("got event-json");
                            let data = event
                                .body()
                                .clone()
                                .expect("Unable to get body of event-json");

                            let event_body =
                                parse_json_body(data).expect("Unable to parse body of event-json");
                            let job_uuid = event_body.get("Job-UUID");
                            if let Some(job_uuid) = job_uuid {
                                let job_uuid = job_uuid.as_str().unwrap();
                                if let Some(tx) =
                                    inner_background_jobs.lock().await.remove(job_uuid)
                                {
                                    let _ = tx
                                        .send(event)
                                        .expect("Unable to send channel message from bgapi");
                                }
                                trace!("continued");
                                continue;
                            }
                            if let Some(application_uuid) = event_body.get("Application-UUID") {
                                let job_uuid = application_uuid.as_str().unwrap();
                                if let Some(event_name) = event_body.get("Event-Name") {
                                    if let Some(event_name) = event_name.as_str() {
                                        if event_name == "CHANNEL_EXECUTE_COMPLETE" {
                                            if let Some(tx) =
                                                inner_background_jobs.lock().await.remove(job_uuid)
                                            {
                                                let _ = tx.send(event).expect(
                                                    "Unable to send channel message from bgapi",
                                                );
                                            }
                                            trace!("continued");
                                            trace!("got channel execute complete");
                                        }
                                    }
                                }
                            }
                            continue;
                        } else {
                            trace!("got another event {:?}", event);
                        }
                    }
                    if let Some(tx) = inner_commands.lock().await.pop_front() {
                        let _ = tx.send(event).expect("msg");
                    }
                }
            }
        });
        let response = connection.send_recv(b"connect").await?;
        trace!("{:?}", response);
        connection.connection_info = response.headers().clone();
        let response = connection
            .subscribe(vec!["BACKGROUND_JOB", "CHANNEL_EXECUTE_COMPLETE"])
            .await?;
        trace!("{:?}", response);
        let response = connection.send_recv(b"myevents").await?;
        trace!("{:?}", response);
        let channel_unique_id = connection
            .connection_info
            .get("Channel-Unique-ID")
            .unwrap()
            .as_str()
            .unwrap();
        connection.call_uuid = Some(channel_unique_id.to_string());

        Ok((connection, addr))
    }
}

fn parse_json_body(body: String) -> Result<HashMap<String, Value>, EslError> {
    Ok(serde_json::from_str(&body)?)
}
pub struct OutboundSession {
    call_uuid: Option<String>,
    connection_info: HashMap<String, Value>,
    commands: Arc<Mutex<VecDeque<Sender<Event>>>>,
    transport_tx: Arc<Mutex<FramedWrite<OwnedWriteHalf, EslCodec>>>,
    background_jobs: Arc<Mutex<HashMap<String, Sender<Event>>>>,
    connected: AtomicBool,
}
impl OutboundSession {
    pub async fn execute(&self, app_name: &str, app_args: &str) -> Result<(), EslError> {
        let event_uuid = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = channel();
        self.background_jobs
            .lock()
            .await
            .insert(event_uuid.clone(), tx);
        let call_uuid = self.call_uuid.as_ref().unwrap().clone();
        let command  = format!("sendmsg {}\nexecute-app-name: {}\nexecute-app-arg: {}\ncall-command: execute\nEvent-UUID: {}",call_uuid,app_name,app_args,event_uuid);
        let response = self.send_recv(command.as_bytes()).await?;
        trace!("inside execute {:?}", response);
        let resp = rx.await?;
        trace!("got response from channel {:?}", resp);
        Ok(())
    }
    pub async fn answer(&self) -> Result<(), EslError> {
        self.execute("answer", "").await
    }
    pub async fn playback(&self, file_path: &str) -> Result<(), EslError> {
        self.execute("playback", file_path).await
    }
    pub async fn subscribe(&self, events: Vec<&str>) -> Result<Event, EslError> {
        let message = format!("event json {}", events.join(" "));
        self.send_recv(message.as_bytes()).await
    }
    pub async fn disconnect(self) -> Result<(), EslError> {
        self.send_recv(b"exit").await?;
        self.connected.store(false, Ordering::Relaxed);
        Ok(())
    }
    pub fn connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }
    pub async fn send(&self, item: &[u8]) -> Result<(), EslError> {
        let mut transport = self.transport_tx.lock().await;
        transport.send(item).await
    }
    pub async fn send_recv(&self, item: &[u8]) -> Result<Event, EslError> {
        self.send(item).await?;
        let (tx, rx) = channel();
        self.commands.lock().await.push_back(tx);
        Ok(rx.await?)
    }
}
