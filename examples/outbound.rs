use tokio::net::TcpListener;

use freeswitch_esl::{Esl, EslConnection, EslError};

async fn process_call(conn: EslConnection) -> Result<(), EslError> {
    conn.answer().await?;
    println!("answered call");
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
    println!("Received Digit: {}", digit);
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
    let listener = Esl::outbound(listener).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move { process_call(socket).await });
    }
}
