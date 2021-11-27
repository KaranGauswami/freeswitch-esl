use std::time::Duration;

use anyhow::Result;
use freeswitch_esl::Inbound;
use log::debug;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let addr = "3.109.206.34:8021".parse().unwrap();
    let inbound = Inbound::new(addr).await?;
    let _ = inbound.api("reloadxml").await;
    let _ = inbound.api("sofia status").await;
    debug!("finished");
    tokio::time::sleep(Duration::from_secs(10)).await;
    Ok(())
}
