# freeswitch-esl

FreeSwitch ESL implementation for Rust

# Examples

## Executing simple commands

```rust
use anyhow::Result;
use freeswitch_esl::blocking::FreeswitchESL;
use std::net::SocketAddr;

fn main() -> Result<()> {

    // connecting to Freeswitch
    let addr = SocketAddr::from(([192, 168, 43, 222], 8021));
    let mut esl = FreeswitchESL::new(addr, "ClueCon")?;

    // executing command
    let response = esl.api("sofia status profile 192.168.43.222 reg")?;
    eprintln!("response {:?}", response);

    Ok(())
}
```
