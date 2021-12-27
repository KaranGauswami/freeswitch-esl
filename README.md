# freeswitch-esl (WIP)

FreeSwitch ESL implementation for Rust

# Examples

## Executing simple commands

```rust

use freeswitch_esl::{inbound::Inbound, InboundError};

#[tokio::main]
async fn main() -> Result<(), InboundError> {
    let addr = "localhost:8021"; // Freeswitch ESL host
    let password = "ClueCon";    // Freeswitch ESL password
    let inbound = Inbound::new(addr, password).await?;

    let reloadxml = inbound.api("reloadxml").await?;
    println!("reloadxml response : {:?}", reloadxml);

    let reloadxml = inbound.bgapi("reloadxml").await?;
    println!("reloadxml response : {:?}", reloadxml);
    Ok(())
}
```
