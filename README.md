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
    let addr = SocketAddr::from(([127, 0, 0, 1], 8021));
    let mut esl = FreeswitchESL::new(addr, "ClueCon")?;

    // executing command
    let response = esl.api("sofia status")?;
    eprintln!("response {:?}", response);

    Ok(())
}
```
