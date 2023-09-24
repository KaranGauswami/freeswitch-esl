use std::{env, net::SocketAddr};

use ntest::timeout;
use regex::Regex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

use anyhow::Result;
use freeswitch_esl::CommandAndApiReplyBody;
use freeswitch_esl::{Code, Esl, EslError};

#[tokio::test]
#[timeout(10000)]
async fn reloadxml_with_api() -> Result<()> {
    let (_, addr) = get_server_address().await?;
    let stream = TcpStream::connect(addr).await?;
    let inbound = Esl::inbound(stream, "ClueCon").await?;
    let response = inbound.api("reloadxml").await;
    assert_eq!(Ok("[Success]".into()), response);
    Ok(())
}
#[tokio::test]
#[timeout(5000)]
async fn reloadxml_with_bgapi() -> Result<()> {
    let (_, addr) = get_server_address().await?;
    // let addr = "localhost:8091";
    let stream = TcpStream::connect(addr).await?;
    let inbound = Esl::inbound(stream, "ClueCon").await?;
    let response = inbound.bgapi("reloadxml").await;
    assert_eq!(Ok("[Success]".into()), response);
    Ok(())
}

#[tokio::test]
#[timeout(10000)]
async fn call_user_that_doesnt_exists() -> Result<()> {
    let (_, addr) = get_server_address().await?;
    let stream = TcpStream::connect(addr).await?;
    let inbound = Esl::inbound(stream, "ClueCon").await?;
    let response = inbound
        .api("originate user/some_user_that_doesnt_exists karan")
        .await
        .unwrap_err();
    assert_eq!(EslError::ApiError("SUBSCRIBER_ABSENT".into()), response);
    Ok(())
}

#[tokio::test]
#[timeout(10000)]
async fn send_recv_test() -> Result<()> {
    let (_, addr) = get_server_address().await?;
    let stream = TcpStream::connect(addr).await?;
    let inbound = Esl::inbound(stream, "ClueCon").await?;
    let response = inbound.send_recv(b"api reloadxml").await?;
    assert_eq!(
        response,
        CommandAndApiReplyBody {
            code: Code::Ok,
            reply_text: "[Success]".into(),
            job_uuid: None
        }
    );
    Ok(())
}

#[tokio::test]
#[timeout(10000)]
async fn wrong_password() -> Result<()> {
    let (_, addr) = get_server_address().await?;
    let stream = TcpStream::connect(addr).await?;
    let result = Esl::inbound(stream, "ClueCons").await;
    assert_eq!(EslError::AuthFailed, result.unwrap_err());
    Ok(())
}

#[tokio::test]
#[timeout(10000)]
async fn multiple_actions() -> Result<()> {
    let (_, addr) = get_server_address().await?;
    let stream = TcpStream::connect(addr).await?;
    let inbound = Esl::inbound(stream, "ClueCon").await?;
    let body = inbound.bgapi("reloadxml").await;
    assert_eq!(Ok("[Success]".into()), body);
    let body = inbound
        .bgapi("originate user/some_user_that_doesnt_exists karan")
        .await;
    assert_eq!(
        Err(EslError::ApiError("SUBSCRIBER_ABSENT".to_string())),
        body
    );
    Ok(())
}

#[tokio::test]
#[timeout(10000)]
async fn concurrent_api() -> Result<()> {
    let (_, addr) = get_server_address().await?;
    let stream = TcpStream::connect(addr).await?;
    let inbound = Esl::inbound(stream, "ClueCon").await?;
    let response1 = inbound.api("reloadxml").await;
    let response2 = inbound
        .api("originate user/some_user_that_doesnt_exists karan")
        .await;
    // let response3 = inbound.api("reloadxml").await;
    // let (response1, response2, response3) = tokio::join!(response1, response2, response3);
    // assert_eq!(Ok("[Success]".into()), response1);
    // assert_eq!(
    //     Err(EslError::ApiError("SUBSCRIBER_ABSENT".into())),
    //     response2
    // );
    // assert_eq!(Ok("[Success]".into()), response3);
    Ok(())
}

#[tokio::test]
#[timeout(10000)]
async fn concurrent_bgapi() -> Result<()> {
    let (_, addr) = get_server_address().await?;
    let stream = TcpStream::connect(addr).await?;
    let inbound = Esl::inbound(stream, "ClueCon").await?;
    let response1 = inbound.bgapi("reloadxml");
    let response2 = inbound.bgapi("originate user/some_user_that_doesnt_exists karan");
    let response3 = inbound.bgapi("reloadxml");
    let (response1, response2, response3) = tokio::join!(response1, response2, response3);
    assert_eq!(Ok("[Success]".to_string()), response1);
    assert_eq!(
        Err(EslError::ApiError("SUBSCRIBER_ABSENT".to_string())),
        response2
    );
    assert_eq!(Ok("[Success]".to_string()), response3);
    Ok(())
}

#[tokio::test]
#[timeout(10000)]
async fn connected_status() -> Result<()> {
    let (_, addr) = get_server_address().await?;
    let stream = TcpStream::connect(addr).await?;
    let inbound = Esl::inbound(stream, "ClueCon").await?;
    assert!(inbound.connected());
    Ok(())
}

#[tokio::test]
#[timeout(10000)]
async fn restart_external_profile() -> Result<()> {
    let (_, addr) = get_server_address().await?;
    let stream = TcpStream::connect(addr).await?;
    let inbound = Esl::inbound(stream, "ClueCon").await?;
    let body = inbound.api("sofia profile external restart").await;
    assert_eq!(
        Ok("Reload XML [Success]\nrestarting: external".into()),
        body
    );
    Ok(())
}

#[tokio::test]
#[timeout(30000)]
async fn uuid_kill() -> Result<()> {
    let (_, addr) = get_server_address().await?;
    let password = "ClueCon";
    let stream = TcpStream::connect(addr).await?;
    let inbound = Esl::inbound(stream, password).await?;

    let uuid = inbound
        .api("originate {origination_uuid=karan}loopback/1000 &conference(karan)")
        .await?;
    assert_eq!("karan", uuid);
    let uuid_kill_response = inbound.api("uuid_kill karan").await?;
    assert_eq!("", uuid_kill_response);
    Ok(())
}

async fn get_server_address() -> Result<(JoinHandle<()>, SocketAddr)> {
    let listener = TcpListener::bind("localhost:0").await?;
    if let Ok(value) = env::var("INTEGRATION") {
        if value.parse::<bool>().unwrap_or_default() {
            let handle = tokio::spawn(async {});
            return Ok((handle, "127.0.0.1:8021".parse().unwrap()));
        }
    }
    let local_address = listener.local_addr()?;
    let server = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let _ = socket.write_all(b"Content-Type: auth/request\n\n").await;

                let mut buffer = [0; 1024];
                let mut received_data = Vec::new();

                loop {
                    let n = match socket.read(&mut buffer).await {
                        Ok(0) => break, // Connection closed
                        Ok(n) => n,
                        Err(_) => break, // Error reading data
                    };
                    received_data.extend_from_slice(&buffer[0..n]);
                    // Check for two newline characters in the received data
                    while let Some(index) = received_data
                        .windows(2)
                        .position(|window| window == b"\n\n")
                    {
                        // Extract the data before the two newlines
                        let data_before_newlines = &received_data[0..index];

                        // Convert the data to a string for comparison
                        let mut data_string =
                            String::from_utf8_lossy(data_before_newlines).to_string();

                        // HACK
                        let response_text: Vec<String> = if data_string.starts_with("bgapi")
                            && data_string.contains("Job-UUID")
                        {
                            let re =
                                Regex::new(r"(?P<bgapi>.+)\nJob-UUID: (?P<uuid>[0-9a-fA-F-]+)")
                                    .unwrap();
                            let captures = re.captures(&data_string).unwrap();
                            // Extract components
                            let _ = &captures["bgapi"];
                            let uuid_old = &captures["uuid"];
                            let uuid_old = uuid_old.to_owned();

                            let new_uuids = uuid::Uuid::new_v4().to_string();
                            data_string = data_string.replace(&uuid_old, &new_uuids);
                            let reloadxml_app = format!("bgapi reloadxml\nJob-UUID: {}", new_uuids);
                            let some_user_that_doesnt_exists = format!("bgapi originate user/some_user_that_doesnt_exists karan\nJob-UUID: {}",new_uuids);

                            let first_1 = "Content-Type: command/reply\nReply-Text: +OK Job-UUID: UUID_PLACEHOLDER\nJob-UUID: UUID_PLACEHOLDER\n\n";
                            if data_string == reloadxml_app {
                                let second_1 = "Content-Length: 575\nContent-Type: text/event-plain\n\nEvent-Name: BACKGROUND_JOB\nCore-UUID: 0cb916f9-98ad-4fce-bcd5-5fe03c745316\nFreeSWITCH-Hostname: ip-172-31-5-95\nFreeSWITCH-Switchname: ip-172-31-5-95\nFreeSWITCH-IPv4: 172.31.5.95\nFreeSWITCH-IPv6: %3A%3A1\nEvent-Date-Local: 2023-09-24%2005%3A48%3A28\nEvent-Date-GMT: Sun,%2024%20Sep%202023%2005%3A48%3A28%20GMT\nEvent-Date-Timestamp: 1695534508726403\nEvent-Calling-File: mod_event_socket.c\nEvent-Calling-Function: api_exec\nEvent-Calling-Line-Number: 1572\nEvent-Sequence: 1041\nJob-UUID: UUID_PLACEHOLDER\nJob-Command: reloadxml\nContent-Length: 14\n\n+OK [Success]\n";
                                let first = first_1.replace("UUID_PLACEHOLDER", &uuid_old);
                                let second = second_1.replace("UUID_PLACEHOLDER", &uuid_old);
                                vec![first, second]
                            } else if data_string == some_user_that_doesnt_exists {
                                let second_1 = "Content-Length: 643\nContent-Type: text/event-plain\n\nEvent-Name: BACKGROUND_JOB\nCore-UUID: 0cb916f9-98ad-4fce-bcd5-5fe03c745316\nFreeSWITCH-Hostname: ip-172-31-5-95\nFreeSWITCH-Switchname: ip-172-31-5-95\nFreeSWITCH-IPv4: 172.31.5.95\nFreeSWITCH-IPv6: %3A%3A1\nEvent-Date-Local: 2023-09-24%2009%3A21%3A50\nEvent-Date-GMT: Sun,%2024%20Sep%202023%2009%3A21%3A50%20GMT\nEvent-Date-Timestamp: 1695547310806421\nEvent-Calling-File: mod_event_socket.c\nEvent-Calling-Function: api_exec\nEvent-Calling-Line-Number: 1572\nEvent-Sequence: 6150\nJob-UUID: UUID_PLACEHOLDER\nJob-Command: originate\nJob-Command-Arg: user/some_user_that_doesnt_exists%20karan\nContent-Length: 23\n\n-ERR SUBSCRIBER_ABSENT\n";
                                let first = first_1.replace("UUID_PLACEHOLDER", &uuid_old);
                                let second = second_1.replace("UUID_PLACEHOLDER", &uuid_old);
                                vec![first, second]
                            } else {
                                panic!("Unhandled application")
                            }
                        } else {
                            // data_string.contains("Job-UUID")

                            let response_text = match data_string.as_ref() {
                            "auth ClueCon" => {
                                "Content-Type: command/reply\nReply-Text: +OK accepted\n\n"
                            }
                            "auth ClueCons"=>{
                                "Content-Type: command/reply\nReply-Text: -ERR invalid\n\n"
                            }
                            "api reloadxml" => {
                                "Content-Type: api/response\nContent-Length: 14\n\n+OK [Success]\n"
                            }
                            "api sofia profile external restart" => {
                                "Content-Type: api/response\nContent-Length: 41\n\nReload XML [Success]\nrestarting: external"
                            }
                            "api originate {origination_uuid=karan}loopback/1000 &conference(karan)" => {
                                "Content-Type: api/response\nContent-Length: 10\n\n+OK karan\n"
                            }
                            "api uuid_kill karan" => {
                                "Content-Type: api/response\nContent-Length: 4\n\n+OK\n"
                            }
                            "event plain BACKGROUND_JOB CHANNEL_EXECUTE_COMPLETE"=>{
                                "Content-Type: command/reply\nReply-Text: +OK event listener enabled plain\n\n"
                            }
                            "api originate user/some_user_that_doesnt_exists karan"=>{
                                "Content-Type: api/response\nContent-Length: 23\n\n-ERR SUBSCRIBER_ABSENT\n"
                            },
                            "bgapi reloadxml"=>{
                                "Content-Length: 575\nContent-Type: text/event-plain\n\nEvent-Name: BACKGROUND_JOB\nCore-UUID: 0cb916f9-98ad-4fce-bcd5-5fe03c745316\nFreeSWITCH-Hostname: ip-172-31-5-95\nFreeSWITCH-Switchname: ip-172-31-5-95\nFreeSWITCH-IPv4: 172.31.5.95\nFreeSWITCH-IPv6: %3A%3A1\nEvent-Date-Local: 2023-09-24%2005%3A48%3A28\nEvent-Date-GMT: Sun,%2024%20Sep%202023%2005%3A48%3A28%20GMT\nEvent-Date-Timestamp: 1695534508726403\nEvent-Calling-File: mod_event_socket.c\nEvent-Calling-Function: api_exec\nEvent-Calling-Line-Number: 1572\nEvent-Sequence: 1041\nJob-UUID: dcab6b81-ec71-4552-b897-88721870fe16\nJob-Command: reloadxml\nContent-Length: 14\n\n+OK [Success]\n"
                            },
                            _ => {
                                "Content-Type: command/reply\nReply-Text: -ERR command not found\n\n"
                            }
                        };
                            vec![response_text.to_string()]
                        };
                        let response_text = response_text.iter();
                        for response in response_text {
                            println!("writing response {:?}", response);
                            if socket.write_all(response.as_bytes()).await.is_err() {
                                eprintln!("error writing data");
                                break; // Error writing data
                            }
                        }

                        // Remove the processed data from the received_data buffer
                        received_data.drain(0..=index + 1);
                    }
                }
            });
        }
    });
    Ok((server, local_address))
}
