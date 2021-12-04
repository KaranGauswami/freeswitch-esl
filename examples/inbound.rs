use std::time::Duration;

use anyhow::Result;
use freeswitch_esl::inbound::Inbound;
use log::{debug, error, info};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let addr = "3.109.206.34:8021".parse().unwrap();
    info!("starting");
    let inbound = Inbound::new(addr).await?;
    let reloadxml = inbound.send_recv(b"api reloadxml\n\n").await;
    error!("reloadxml response : {:?}", reloadxml);
    let sofia = inbound.send_recv(b"bgapi sofia status\n\n").await;
    error!("sofia response : {:?}", sofia);
    let reloadxml = inbound.send_recv(b"api reloadxml\n\n").await;
    error!("reloadxml response : {:?}", reloadxml);
    debug!("finished");
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}
