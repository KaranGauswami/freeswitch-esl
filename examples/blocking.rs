use freeswitch_esl::blocking::InboundConn;
use std::net::SocketAddr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([192, 168, 43, 222], 8021));
    let esl = InboundConn::new(addr, "ClueCon")?;

    let response = esl.api("status")?;

    println!("response headers {:?}", response.headers());
    println!("response body {:?}", response.body());

    Ok(())
}
