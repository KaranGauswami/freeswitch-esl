use crate::error::EslError;
use crate::esl::EslConnectionType;
use crate::parser::{
    parse_any_freeswitch_event, parse_auth_request, CommandAndApiReplyBody, FreeswitchReply,
};
use dashmap::DashMap;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::io::{AsyncReadExt, AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{
    oneshot::{channel, Sender},
    Mutex,
};
use tracing::{error, info, trace, warn};
#[derive(Debug)]
/// contains Esl connection with freeswitch
pub struct EslConnection {
    password: String,
    commands: Arc<Mutex<VecDeque<Sender<CommandAndApiReplyBody>>>>,
    transport_tx: Arc<Mutex<WriteHalf<TcpStream>>>,
    background_jobs: Arc<DashMap<String, Sender<FreeswitchReply>>>,
    connected: AtomicBool,
    pub(crate) call_uuid: Option<String>,
    connection_info: Option<HashMap<String, String>>,
}

impl EslConnection {
    /// returns call uuid in outbound mode
    pub async fn call_uuid(&self) -> Option<String> {
        self.call_uuid.clone()
    }
    /// disconnects from freeswitch
    pub async fn disconnect(self) -> Result<(), EslError> {
        self.send_recv(b"exit").await?;
        self.connected.store(false, Ordering::Relaxed);
        Ok(())
    }
    /// returns status of esl connection
    pub fn connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }
    pub(crate) async fn send(&self, item: &[u8]) -> Result<(), EslError> {
        let mut transport = self.transport_tx.lock().await;
        let error = transport.write_all(item).await;
        error!("Error writing data into TCP stream {:?}", error);
        // TODO: fix this write
        let error = transport.write_all(b"\n\n").await;
        error!("Error writing data into TCP stream {:?}", error);
        Ok(())
    }
    /// sends raw message to freeswitch and receives reply
    pub async fn send_recv(&self, item: &[u8]) -> Result<CommandAndApiReplyBody, EslError> {
        self.send(item).await?;
        let (tx, rx) = channel();
        self.commands.lock().await.push_back(tx);
        Ok(rx.await?)
    }

    pub(crate) async fn new(
        mut stream: TcpStream,
        password: impl ToString,
        connection_type: EslConnectionType,
    ) -> Result<Self, EslError> {
        let commands = Arc::new(Mutex::new(VecDeque::new()));
        let inner_commands = Arc::clone(&commands);
        let background_jobs = Arc::new(DashMap::new());
        let inner_background_jobs = Arc::clone(&background_jobs);
        let mut dst = Vec::new();
        let mut bufs = [0; 1024];
        if connection_type == EslConnectionType::Inbound {
            // transport_rx.next().await;
            let bytes = stream.read(&mut bufs[..]).await.unwrap();
            let auth_message = String::from_utf8_lossy(&bufs[..bytes]);
            let _ = parse_auth_request(&auth_message);
        }
        let (mut read_half, write_half) = tokio::io::split(stream);
        let transport_tx = Arc::new(Mutex::new(write_half));
        let mut connection = Self {
            password: password.to_string(),
            commands,
            background_jobs,
            transport_tx,
            connected: AtomicBool::new(false),
            call_uuid: None,
            connection_info: None,
        };
        tokio::spawn(async move {
            loop {
                let read_bytes = read_half.read(&mut bufs[..]).await.unwrap();

                if read_bytes == 0 {
                    break;
                }
                dst.extend_from_slice(&bufs[0..read_bytes]);
                loop {
                    let inputs = String::from_utf8_lossy(&dst).to_string();
                    if let Ok(event) = parse_any_freeswitch_event(&inputs) {
                        let (remaining, parsed) = event;
                        dst.drain(0..inputs.len() - remaining.len());
                        match parsed {
                            FreeswitchReply::AuthRequest => {
                                if let Some(tx) = inner_commands.lock().await.pop_front() {
                                    tx.send(CommandAndApiReplyBody::default()).expect("msg");
                                }
                            }

                            FreeswitchReply::CommandAndApiReply(n) => {
                                if let Some(tx) = inner_commands.lock().await.pop_front() {
                                    tx.send(n).expect("msg");
                                }
                            }
                            FreeswitchReply::Event(n) => {
                                if let (Some(job_uuid), Some(event_name)) =
                                    (n.headers.get("Job-UUID"), n.headers.get("Event-Name"))
                                {
                                    if event_name == "BACKGROUND_JOB" {
                                        if let Some(tx) = inner_background_jobs.remove(job_uuid) {
                                            match tx.1.send(FreeswitchReply::Event(n.clone())) {
                                                Ok(_) => {}
                                                Err(e) => {
                                                    warn!(
                                                        "error notifying background jobs {:?}",
                                                        e
                                                    );
                                                }
                                            }
                                        } else {
                                            warn!(
                                                "this is background job was not present {:?}",
                                                job_uuid
                                            );
                                        }
                                    }
                                }
                                if let Some(event_name) = n.headers.get("Event-Name") {
                                    if event_name == "CHANNEL_EXECUTE_COMPLETE" {
                                        if let Some(application_uuid) =
                                            n.headers.get("Application-UUID")
                                        {
                                            if let Some(tx) =
                                                inner_background_jobs.remove(application_uuid)
                                            {
                                                match tx.1.send(
                                                    FreeswitchReply::CommandAndApiReply(
                                                        CommandAndApiReplyBody {
                                                            headers: n.headers.clone(),
                                                            code: n.code.clone(),
                                                            reply_text: n.body.clone(),
                                                            job_uuid: None,
                                                        },
                                                    ),
                                                ) {
                                                    Ok(_) => {}
                                                    Err(e) => {
                                                        warn!(
                                                            "error notifying background jobs {:?}",
                                                            e
                                                        );
                                                    }
                                                }
                                            } else {
                                                warn!(
                                                    "this is background job was not present {:?}",
                                                    application_uuid
                                                );
                                            }
                                        }
                                    }
                                    if event_name == "CHANNEL_DATA" {
                                        info!("content-type {:?}", n.headers.get("Content-Type"));
                                        if let Some(content_type) = n.headers.get("Content-Type") {
                                            if content_type == "command/reply" {
                                                if let Some(tx) =
                                                    inner_commands.lock().await.pop_front()
                                                {
                                                    tx.send(CommandAndApiReplyBody {
                                                        headers: n.headers,
                                                        ..Default::default()
                                                    })
                                                    .expect("msg");
                                                }
                                            };
                                        }
                                    }
                                }
                            }
                            _ => {
                                panic!("handle this case bro")
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        });
        match connection_type {
            EslConnectionType::Inbound => {
                let auth_response = connection.auth().await?;
                trace!("auth_response {:?}", auth_response);
                connection
                    .subscribe(vec!["BACKGROUND_JOB", "CHANNEL_EXECUTE_COMPLETE"])
                    .await?;
            }
            EslConnectionType::Outbound => {
                let response = connection.send_recv(b"connect").await?;
                connection.connection_info = Some(response.headers.clone());
                let response = connection
                    .subscribe(vec!["BACKGROUND_JOB", "CHANNEL_EXECUTE_COMPLETE"])
                    .await?;
                trace!("{:?}", response);
                let response = connection.send_recv(b"myevents").await?;
                trace!("{:?}", response);
                let connection_info = connection.connection_info.as_ref().unwrap();

                let channel_unique_id = connection_info.get("Channel-Unique-ID").unwrap().as_str();
                connection.call_uuid = Some(channel_unique_id.to_string());
            }
        }
        Ok(connection)
    }

    /// subscribes to given events
    pub async fn subscribe(&self, events: Vec<&str>) -> Result<CommandAndApiReplyBody, EslError> {
        let message = format!("event plain {}", events.join(" "));
        self.send_recv(message.as_bytes()).await
    }

    pub(crate) async fn auth(&self) -> Result<String, EslError> {
        {
            let auth_response = self
                .send_recv(format!("auth {}", self.password).as_bytes())
                .await?;
            match auth_response.code {
                crate::parser::Code::Ok => {
                    self.connected.store(true, Ordering::Relaxed);
                    Ok("AuthSuccess".into())
                }
                crate::parser::Code::Err => {
                    self.connected.store(false, Ordering::Relaxed);
                    Err(EslError::AuthFailed)
                }
            }
        }
    }

    /// For hanging up call in outbound mode
    pub async fn hangup(&self, reason: &str) -> Result<CommandAndApiReplyBody, EslError> {
        self.execute("hangup", reason).await
    }

    /// executes application in freeswitch
    pub async fn execute(
        &self,
        app_name: &str,
        app_args: &str,
    ) -> Result<CommandAndApiReplyBody, EslError> {
        let event_uuid = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = channel();
        self.background_jobs.insert(event_uuid.clone(), tx);
        let call_uuid = self.call_uuid.as_ref().unwrap().clone();
        let command  = format!("sendmsg {}\nexecute-app-name: {}\nexecute-app-arg: {}\ncall-command: execute\nEvent-UUID: {}",call_uuid,app_name,app_args,event_uuid);
        let response = self.send_recv(command.as_bytes()).await?;
        trace!("inside execute {:?}", response);
        let resp = rx.await?;
        match resp {
            FreeswitchReply::CommandAndApiReply(n) => Ok(n),
            _ => {
                panic!("this should not happened {:?}", resp);
            }
        }
    }

    /// answers call in outbound mode
    pub async fn answer(&self) -> Result<CommandAndApiReplyBody, EslError> {
        self.execute("answer", "").await
    }

    /// sends api command to freeswitch
    pub async fn api(&self, command: &str) -> Result<String, EslError> {
        let response = self.send_recv(format!("api {}", command).as_bytes()).await;
        let event = response?;
        match event.code {
            crate::parser::Code::Ok => Ok(event.reply_text),
            crate::parser::Code::Err => Err(EslError::ApiError(event.reply_text)),
        }
    }

    /// sends bgapi commands to freeswitch
    pub async fn bgapi(&self, command: &str) -> Result<String, EslError> {
        trace!("Send bgapi {}", command);
        let job_uuid = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = channel();
        self.background_jobs.insert(job_uuid.clone(), tx);

        self.send_recv(format!("bgapi {}\nJob-UUID: {}", command, job_uuid).as_bytes())
            .await?;

        let resp = rx.await?;
        match resp {
            FreeswitchReply::Event(n) => match n.code {
                crate::parser::Code::Ok => Ok(n.body),
                crate::parser::Code::Err => Err(EslError::ApiError(n.body)),
            },
            _ => {
                panic!("unwanted data")
            }
        }
    }
}
