# freeswitch-esl

FreeSwitch ESL implementation for Rust

# Examples

## Executing simple commands

```rust
use anyhow::Result;
use freeswitch_esl::blocking::OutboundConn;
use std::net::SocketAddr;

fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8021));
    let mut esl = OutboundConn::new(addr, "ClueCon")?;
    let response = esl.api("sofia status")?;
    eprintln!("response headers {:?}", response.headers());
    eprintln!("response body {:?}", response.body());

    Ok(())
}
```
