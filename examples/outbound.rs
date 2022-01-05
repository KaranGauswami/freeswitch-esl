use freeswitch_esl::{
    outbound::{Outbound, OutboundSession},
    InboundError,
};

#[tokio::main]
async fn main() -> Result<(), InboundError> {
    env_logger::init();
    let addr = "0.0.0.0:8085"; // Freeswitch host
    let listener = Outbound::bind(addr).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move { process_call(socket).await });
    }
}
async fn process_call(conn: OutboundSession) -> Result<(), InboundError> {
    conn.answer().await?;
    conn.playback("ivr/ivr-welcome.wav").await?;
    conn.playback("misc/misc-freeswitch_is_state_of_the_art.wav")
        .await?;
    Ok(())
}
