use freeswitch_esl::{inbound::Inbound, InboundError};

#[tokio::main]
async fn main() -> Result<(), InboundError> {
    let addr = "3.109.206.34:8021"; // Freeswitch host
    let password = "ClueCon";
    let inbound = Inbound::new(addr, password).await?;

    let uuid = inbound
        .api("originate {origination_uuid=karan}loopback/1000 &conference(karan)")
        .await?;
    println!("{:?}", uuid);
    let reloadxml = inbound.api(&format!("uuid_kill karan")).await?;
    println!("{:?}", reloadxml);
    Ok(())
}
