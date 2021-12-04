use std::time::Duration;

use anyhow::Result;
use freeswitch_esl::inbound::Inbound;
use log::{debug, error, info};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let addr = "3.109.206.34:8021".parse().unwrap();
    info!("starting");
    let inbound = Inbound::new(addr, "ClueCon").await?;
    let reloadxml = inbound
        .bgapi("originate user/1000 &conference(karan)")
        .await;
    error!("reloadxml response : {:?}", reloadxml);
    tokio::time::sleep(Duration::from_secs(2)).await;
    let sofia = inbound
        .bgapi("originate user/1001 &conference(karan)")
        .await;
    error!("sofia response : {:?}", sofia);
    tokio::time::sleep(Duration::from_secs(2)).await;
    let reloadxml = inbound
        .bgapi("originate user/1000 &conference(karan)")
        .await;
    error!("reloadxml response : {:?}", reloadxml);
    tokio::time::sleep(Duration::from_secs(2)).await;
    debug!("finished");
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}
