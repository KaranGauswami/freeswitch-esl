use freeswitch_esl::inbound::Inbound;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let addr = "3.109.206.34:8021"; // Freeswitch host
    let password = "ClueCon";
    let inbound = Inbound::new(addr, password)
        .await
        .expect("Unable to create inbound connection");

    let reloadxml = inbound
        .api("reloadxml")
        .await
        .expect("Unable to send api command");
    println!("reloadxml response : {:?}", reloadxml);

    let reloadxml = inbound.bgapi("reloadxml").await;
    println!("reloadxml response : {:?}", reloadxml);

    Ok(())
}
