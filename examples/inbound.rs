use std::time::Duration;

use anyhow::Result;
use freeswitch_esl::inbound::Inbound;
use log::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let addr = "3.109.206.34:8021".parse().unwrap();
    let inbound = Inbound::new(addr).await?;
    let reloadxml = inbound.api("reloadxml").await;
    info!("reloadxml response : {:?}", reloadxml);
    let sofia = inbound.bgapi("sofia status").await;
    info!("sofia response : {:?}", sofia);
    let reloadxml = inbound.api("reloadxml").await;
    info!("reloadxml response : {:?}", reloadxml);
    debug!("finished");
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}
