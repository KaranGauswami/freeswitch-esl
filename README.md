# freeswitch-esl (WIP)

FreeSwitch ESL implementation for Rust

# Examples

## Executing simple commands

```rust
use freeswitch_esl::blocking::InboundConn;
use std::net::SocketAddr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([192, 168, 43, 222], 8021));
    let esl = InboundConn::new(addr, "ClueCon")?;

    let response = esl.api("status")?;

    eprintln!("response headers {:?}", response.headers());
    eprintln!("response body {:?}", response.body());

    Ok(())
}


```
