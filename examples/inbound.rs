use std::time::Duration;

use anyhow::Result;
use freeswitch_esl::inbound::Inbound;
use log::{debug, error, info};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let addr = "3.109.206.34:8021";
    debug!("starting");
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let reloadxml = inbound.api("reloadxml").await?;
    info!("reloadxml response : {:?}", reloadxml);
    tokio::time::sleep(Duration::from_secs(1)).await;
    let reloadxml = inbound.bgapi("reloadxml").await;
    error!("reloadxml response : {:?}", reloadxml);
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}
