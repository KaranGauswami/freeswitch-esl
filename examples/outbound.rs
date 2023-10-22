use freeswitch_esl::{Esl, EslConnection, EslError};
use tokio::net::TcpListener;

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
    println!("got digit {}", digit);
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
        tokio::spawn(async move {
            let stream = Esl::outbound(socket).await.unwrap();
            process_call(stream).await.unwrap();
        });
    }
}
