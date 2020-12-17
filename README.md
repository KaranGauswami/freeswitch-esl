# freeswitch-esl

FreeSwitch ESL implementation for Rust

# Examples

## Executing simple commands

```rust
use anyhow::Result;
use freeswitch_esl::blocking::FreeswitchESL;
use std::net::SocketAddr;

fn main() -> Result<()> {
    let addr = SocketAddr::from(([192, 168, 43, 222], 8021));
    let mut esl = FreeswitchESL::new(addr, "ClueCon")?;
    let response = esl.api("sofia status profile 192.168.43.222 reg")?;
    eprintln!("response {:?}", response);
    let response = esl.api("originate user/1002 1002")?;
    eprintln!("response {:?}", response);
    let response = esl.api("originate user/1001 1001")?;
    eprintln!("response {:?}", response);
    Ok(())
}
```
