use freeswitch_esl::{Esl, EslError};

#[tokio::test]
async fn reloadxml() -> Result<(), EslError> {
    let addr = "3.109.206.34:8021";
    let inbound = Esl::inbound(addr, "ClueCon").await?;
    let response = inbound.api("reloadxml").await;
    assert_eq!(Ok("[Success]".into()), response);
    Ok(())
}

#[tokio::test]
async fn call_user_that_doesnt_exists() -> Result<(), EslError> {
    let addr = "3.109.206.34:8021";
    let inbound = Esl::inbound(addr, "ClueCon").await?;
    let response = inbound
        .api("originate user/some_user_that_doesnt_exists karan")
        .await
        .unwrap_err();
    assert_eq!(EslError::ApiError("SUBSCRIBER_ABSENT".into()), response);
    Ok(())
}

#[tokio::test]
async fn send_recv_test() -> Result<(), EslError> {
    let addr = "3.109.206.34:8021";
    let inbound = Esl::inbound(addr, "ClueCon").await?;
    let response = inbound.send_recv(b"api reloadxml\n\n").await?;
    let body = response.body().clone().unwrap();
    assert_eq!("+OK [Success]\n", body);
    Ok(())
}

#[tokio::test]
async fn wrong_password() -> core::result::Result<(), EslError> {
    let addr = "3.109.206.34:8021";
    let result = Esl::inbound(addr, "ClueCons").await;
    assert_eq!(EslError::AuthFailed, result.unwrap_err());
    Ok(())
}

#[tokio::test]
async fn multiple_actions() -> core::result::Result<(), EslError> {
    let addr = "3.109.206.34:8021";
    let inbound = Esl::inbound(addr, "ClueCon").await?;
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
async fn concurrent_api() -> core::result::Result<(), EslError> {
    let addr = "3.109.206.34:8021";
    let inbound = Esl::inbound(addr, "ClueCon").await?;
    let response1 = inbound.api("reloadxml");
    let response2 = inbound.api("originate user/some_user_that_doesnt_exists karan");
    let response3 = inbound.api("reloadxml");
    let (result1, result2, result3) = tokio::join!(response1, response2, response3);
    assert_eq!(Ok("[Success]".into()), result1);
    assert_eq!(Err(EslError::ApiError("SUBSCRIBER_ABSENT".into())), result2);
    assert_eq!(Ok("[Success]".into()), result3);
    Ok(())
}

#[tokio::test]
async fn concurrent_bgapi() -> core::result::Result<(), EslError> {
    let addr = "3.109.206.34:8021";
    let inbound = Esl::inbound(addr, "ClueCon").await?;
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
async fn connected_status() -> Result<(), EslError> {
    let addr = "3.109.206.34:8021";
    let inbound = Esl::inbound(addr, "ClueCon").await?;
    assert_eq!(true, inbound.connected());
    Ok(())
}

#[tokio::test]
async fn restart_external_profile() -> Result<(), EslError> {
    let addr = "3.109.206.34:8021";
    let inbound = Esl::inbound(addr, "ClueCon").await?;
    let body = inbound.api("sofia profile external restart").await;
    assert_eq!(
        Ok("Reload XML [Success]\nrestarting: external".into()),
        body
    );
    Ok(())
}

#[tokio::test]
async fn uuid_kill() -> Result<(), EslError> {
    let addr = "3.109.206.34:8021"; // Freeswitch host
    let password = "ClueCon";
    let inbound = Esl::inbound(addr, password).await?;

    let uuid = inbound
        .api("originate {origination_uuid=karan}loopback/1000 &conference(karan)")
        .await?;
    assert_eq!("karan", uuid);
    let uuid_kill_response = inbound.api(&format!("uuid_kill karan")).await?;
    assert_eq!("", uuid_kill_response);
    Ok(())
}
