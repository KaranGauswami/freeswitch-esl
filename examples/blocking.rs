use anyhow::Result;
use freeswitch_esl::blocking::OutboundConn;
use std::net::SocketAddr;

fn main() -> Result<()> {
    let addr = SocketAddr::from(([192, 168, 43, 222], 8021));
    let esl = OutboundConn::new(addr, "ClueCon")?;

    let response = esl.api("status")?;

    eprintln!("response headers {:?}", response.headers());
    eprintln!("response body {:?}", response.body());

    Ok(())
}
