use std::{io::BufRead, net::SocketAddr};

use anyhow::Result;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed, FramedRead};

struct Karan {}
impl Karan {
    fn new() -> Self {
        Self {}
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let addr: SocketAddr = "3.109.206.34:8021".parse().unwrap();
    let stream = TcpStream::connect(addr).await?;
    let my_coded = Karan::new();
    let mut transport = Framed::new(stream, my_coded);
    let mut counter = 0;
    while let Some(frames) = transport.next().await {
        println!("lol");
        println!("{:?}", frames);
        if let Ok(something) = frames {
            match something {
                Event::Auth => {
                    println!("got auth");
                    let _ = transport.send("auth ClueCon\n\n".to_string()).await;
                    let _ = transport.next().await;
                }
                Event::Reply(n) => {
                    println!("got reply {}", n);
                    break;
                }
            }
        }
    }
    Ok(())
}
impl Encoder<String> for Karan {
    type Error = tokio::io::Error;
    fn encode(&mut self, item: String, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        println!("self {}", item);
        dst.extend_from_slice(b"auth ClueCon\r\n\r\n");
        return Ok(());
    }
}
#[derive(Debug)]
enum Event {
    Auth,
    Reply(String),
}
impl Decoder for Karan {
    type Item = Event;
    type Error = anyhow::Error;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.starts_with(b"Content-Type: auth/request\n\n") {
            src.clear();
            Ok(Some(Event::Auth {}))
        } else if src.starts_with(b"Content-Type: command/reply\nReply-Text: +OK accepted\n\n") {
            let sts = String::from_utf8(src.to_vec());
            Ok(Some(Event::Reply(sts.unwrap())))
        } else {
            println!("{:?}", src);
            Ok(None)
        }
    }
}
