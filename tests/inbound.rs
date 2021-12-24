use anyhow::Result;
use freeswitch_esl::inbound::Inbound;

#[tokio::test]
async fn reloadxml() -> Result<()> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let response = inbound.api("reloadxml").await.unwrap();
    let body = response.body().unwrap();
    assert_eq!("+OK [Success]\n", body);
    Ok(())
}

#[tokio::test]
async fn call_user_that_doesnt_exists() -> Result<()> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let response = inbound
        .api("originate user/some_user_that_doesnt_exists karan")
        .await
        .unwrap();
    let body = response.body().unwrap();
    assert_eq!("-ERR SUBSCRIBER_ABSENT\n", body);
    Ok(())
}

#[tokio::test]
async fn send_recv_test() -> Result<()> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let response = inbound.send_recv(b"api reloadxml\n\n").await?;
    let body = response.body().unwrap();
    assert_eq!("+OK [Success]\n", body);
    Ok(())
}

#[tokio::test]
#[should_panic]
async fn wrong_password() {
    let addr = "3.109.206.34:8021";
    Inbound::new(addr, "ClueCons").await.unwrap();
}

#[tokio::test]
async fn multiple_actions() -> Result<()> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let response = inbound.bgapi("reloadxml").await.unwrap();
    let body = response.body().unwrap();
    assert_eq!("+OK [Success]\n", body);
    let response = inbound
        .bgapi("originate user/some_user_that_doesnt_exists karan")
        .await
        .unwrap();
    let body = response.body().unwrap();
    assert_eq!("-ERR SUBSCRIBER_ABSENT\n", body);
    Ok(())
}

#[tokio::test]
async fn concurrent_api() -> Result<()> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let response1 = inbound.api("reloadxml");
    let response2 = inbound.api("originate user/some_user_that_doesnt_exists karan");
    let response3 = inbound.api("reloadxml");
    let (result1, result2, result3) = tokio::join!(response1, response2, response3);
    let (result1, result2, result3) = (result1.unwrap(), result2.unwrap(), result3.unwrap());
    let body = result1.body().unwrap();
    assert_eq!("+OK [Success]\n", body);
    let body = result2.body().unwrap();
    assert_eq!("-ERR SUBSCRIBER_ABSENT\n", body);
    let body = result3.body().unwrap();
    assert_eq!("+OK [Success]\n", body);
    Ok(())
}

#[tokio::test]
async fn concurrent_bgapi() -> Result<()> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let response1 = inbound.bgapi("reloadxml");
    let response2 = inbound.bgapi("originate user/some_user_that_doesnt_exists karan");
    let response3 = inbound.bgapi("reloadxml");
    let (result1, result2, result3) = tokio::join!(response1, response2, response3);
    let (result1, result2, result3) = (result1.unwrap(), result2.unwrap(), result3.unwrap());
    let body = result1.body().unwrap();
    assert_eq!("+OK [Success]\n", body);
    let body = result2.body().unwrap();
    assert_eq!("-ERR SUBSCRIBER_ABSENT\n", body);
    let body = result3.body().unwrap();
    assert_eq!("+OK [Success]\n", body);
    Ok(())
}

#[tokio::test]
async fn connected_status() -> Result<()> {
    let addr = "3.109.206.34:8021";
    let inbound = Inbound::new(addr, "ClueCon").await?;
    assert_eq!(true, inbound.connected());
    Ok(())
}
#[tokio::test]
async fn with_tcpstream() -> Result<()> {
    let addr = "3.109.206.34:8021";
    let stream = tokio::net::TcpStream::connect(addr).await?;
    let inbound = Inbound::with_tcpstream(stream, "ClueCon").await?;
    let response = inbound.api("reloadxml").await.unwrap();
    let body = response.body().unwrap();
    assert_eq!("+OK [Success]\n", body);
    Ok(())
}
