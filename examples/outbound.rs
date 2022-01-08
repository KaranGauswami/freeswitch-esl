use freeswitch_esl::{Esl, EslConnection, EslError};

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
