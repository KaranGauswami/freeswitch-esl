use freeswitch_esl::{inbound::Inbound, InboundError};

#[tokio::test]
async fn reloadxml() -> Result<(), InboundError> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let response = inbound.api("reloadxml").await;
    assert_eq!(Ok("[Success]".into()), response);
    Ok(())
}

#[tokio::test]
async fn call_user_that_doesnt_exists() -> Result<(), InboundError> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let response = inbound
        .api("originate user/some_user_that_doesnt_exists karan")
        .await
        .unwrap_err();
    assert_eq!(InboundError::ApiError("SUBSCRIBER_ABSENT".into()), response);
    Ok(())
}

#[tokio::test]
async fn send_recv_test() -> Result<(), InboundError> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let response = inbound.send_recv(b"api reloadxml\n\n").await?;
    let body = response.body().unwrap();
    assert_eq!("+OK [Success]\n", body);
    Ok(())
}

#[tokio::test]
async fn wrong_password() -> core::result::Result<(), InboundError> {
    let addr = "3.109.206.34:8021";
    let result = Inbound::new(addr, "ClueCons").await;
    assert_eq!(InboundError::AuthFailed, result.unwrap_err());
    Ok(())
}

#[tokio::test]
async fn multiple_actions() -> core::result::Result<(), InboundError> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let body = inbound.bgapi("reloadxml").await;
    assert_eq!(Ok("[Success]".into()), body);
    let body = inbound
        .bgapi("originate user/some_user_that_doesnt_exists karan")
        .await;
    assert_eq!(
        Err(InboundError::ApiError("SUBSCRIBER_ABSENT".to_string())),
        body
    );
    Ok(())
}

#[tokio::test]
async fn concurrent_api() -> core::result::Result<(), InboundError> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let response1 = inbound.api("reloadxml");
    let response2 = inbound.api("originate user/some_user_that_doesnt_exists karan");
    let response3 = inbound.api("reloadxml");
    let (result1, result2, result3) = tokio::join!(response1, response2, response3);
    assert_eq!(Ok("[Success]".into()), result1);
    assert_eq!(
        Err(InboundError::ApiError("SUBSCRIBER_ABSENT".into())),
        result2
    );
    assert_eq!(Ok("[Success]".into()), result3);
    Ok(())
}

#[tokio::test]
async fn concurrent_bgapi() -> core::result::Result<(), InboundError> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let response1 = inbound.bgapi("reloadxml");
    let response2 = inbound.bgapi("originate user/some_user_that_doesnt_exists karan");
    let response3 = inbound.bgapi("reloadxml");
    let (response1, response2, response3) = tokio::join!(response1, response2, response3);
    assert_eq!(Ok("[Success]".to_string()), response1);
    assert_eq!(
        Err(InboundError::ApiError("SUBSCRIBER_ABSENT".to_string())),
        response2
    );
    assert_eq!(Ok("[Success]".to_string()), response3);
    Ok(())
}

#[tokio::test]
async fn connected_status() -> Result<(), InboundError> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    assert_eq!(true, inbound.connected());
    Ok(())
}
#[tokio::test]
async fn with_tcpstream() -> Result<(), InboundError> {
    let addr = "3.109.206.34:8021";
    let stream = tokio::net::TcpStream::connect(addr).await?;
    let inbound = Inbound::with_tcpstream(stream, "ClueCon").await?;
    let body = inbound.api("reloadxml").await;
    assert_eq!(Ok("[Success]".into()), body);
    Ok(())
}

#[tokio::test]
async fn restart_external_profile() -> Result<(), InboundError> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let body = inbound.api("sofia profile external restart").await;
    assert_eq!(
        Ok("Reload XML [Success]\nrestarting: external".into()),
        body
    );
    Ok(())
}

#[tokio::test]
async fn uuid_kill() -> Result<(), InboundError> {
    let addr = "3.109.206.34:8021"; // Freeswitch host
    let password = "ClueCon";
    let inbound = Inbound::new(addr, password).await?;

    let uuid = inbound
        .api("originate {origination_uuid=karan}loopback/1000 &conference(karan)")
        .await?;
    assert_eq!("karan", uuid);
    let uuid_kill_response = inbound.api(&format!("uuid_kill karan")).await?;
    assert_eq!("", uuid_kill_response);
    Ok(())
}
