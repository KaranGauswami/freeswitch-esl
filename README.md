# UNSTABLE BRANCH

# freeswitch-esl (WIP)

![workflow](https://github.com/KaranGauswami/freeswitch-esl/actions/workflows/rust.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/freeswitch-esl.svg)](https://crates.io/crates/freeswitch-esl)
[![Documentation](https://docs.rs/freeswitch-esl/badge.svg)](https://docs.rs/freeswitch-esl/)

FreeSwitch ESL implementation for Rust

# Examples

## Inbound Example

```rust
use freeswitch_esl::{Esl, EslError};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), EslError> {
    let addr = "localhost:8021"; // Freeswitch host
    let password = "ClueCon";
    let stream = TcpStream::connect(addr).await?;
    let inbound = Esl::inbound(stream, password).await?;

    let reloadxml = inbound.api("reloadxml").await?;
    println!("reloadxml response : {:?}", reloadxml);

    let reloadxml = inbound.bgapi("reloadxml").await?;
    println!("reloadxml response : {:?}", reloadxml);

    Ok(())
}

```

## Outbound Example

```rust
use tokio::net::TcpListener;
use freeswitch_esl::{Esl, EslConnection, EslError};

async fn process_call(conn: EslConnection) -> Result<(), EslError> {
    conn.answer().await?;
    conn.playback("ivr/ivr-welcome.wav").await?;
    let digit = conn
        .play_and_get_digits(
            1,
            1,
            3,
            3000,
            "#",
            "conference/conf-pin.wav",
            "conference/conf-bad-pin.wav",
        )
        .await?;
    conn.playback("ivr/ivr-you_entered.wav").await?;
    conn.playback(&format!("digits/{}.wav", digit)).await?;
    conn.hangup("NORMAL_CLEARING").await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), EslError> {
    let addr = "0.0.0.0:8085"; // Listening address
    println!("Listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        let socket = Esl::outbound(socket).await?;
        tokio::spawn(async move { process_call(socket).await });
    }
}

```

## TODO

- [ ] support for event listener
