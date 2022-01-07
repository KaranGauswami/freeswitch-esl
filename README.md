# freeswitch-esl (WIP)

FreeSwitch ESL implementation for Rust

# Examples

## Inbound Example

```rust
use freeswitch_esl::esl::Esl;
use freeswitch_esl::EslError;

#[tokio::main]
async fn main() -> Result<(), EslError> {
    let addr = "3.109.206.34:8021"; // Freeswitch host
    let password = "ClueCon";
    let inbound = Esl::inbound(addr, password).await?;

    let reloadxml = inbound.api("reloadxml").await?;
    println!("reloadxml response : {:?}", reloadxml);

    let reloadxml = inbound.bgapi("reloadxml").await?;
    println!("reloadxml response : {:?}", reloadxml);

    Ok(())
}
```

## Outbound Example

```rust
use freeswitch_esl::{connection::EslConnection, Esl, EslError};

async fn process_call(conn: EslConnection) -> Result<(), EslError> {
    conn.answer().await?;
    conn.playback("ivr/ivr-welcome.wav").await?;
    let digit = conn
        .play_and_get_digits(1, 1, 3, 5000, "#", "conference/conf-pin.wav", "invalid.wav")
        .await?;
    println!("got digit {}", digit);
    conn.playback("ivr/ivr-you_entered.wav").await?;
    conn.playback(&format!("digits/{}.wav", digit)).await?;
    conn.hangup().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), EslError> {
    env_logger::init();
    let addr = "0.0.0.0:8085"; // Listening address
    let listener = Esl::outbound(addr).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move { process_call(socket).await });
    }
}

```
