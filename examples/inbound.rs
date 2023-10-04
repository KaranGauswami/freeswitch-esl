use freeswitch_esl::{Esl, EslError};

#[tokio::main]
async fn main() -> Result<(), EslError> {
    let addr = "localhost:8021"; // Freeswitch host
    let password = "ClueCon";
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let inbound = Esl::inbound(addr, password, Some(tx)).await?;

    let reloadxml = inbound.api("reloadxml").await?;
    println!("reloadxml response : {:?}", reloadxml);

    let reloadxml = inbound.bgapi("reloadxml").await?;
    println!("reloadxml response : {:?}", reloadxml);

    let subscribe = inbound.subscribe(vec!["all"]).await?;
    println!("subscribe all response : {:?}", subscribe);

    loop {
        match rx.recv().await {
            Some(ev) => println!("received event: {:#?}", ev),
            _ => break,
        }
    }
    Ok(())
}
