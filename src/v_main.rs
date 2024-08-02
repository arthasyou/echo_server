use byteorder::{BigEndian, ByteOrder}; // 用于解析字节序
use bytes::BytesMut;
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};

pub mod ftproto {
    include!(concat!(env!("OUT_DIR"), "/ftproto.rs"));
}
mod router;

const CHANNEL_POOL_SIZE: usize = 100;

type Tx = mpsc::Sender<(u16, u16, BytesMut)>;
type Rx = mpsc::Receiver<(u16, u16, BytesMut)>;

struct ChannelPool {
    pool: Arc<Mutex<Vec<Tx>>>,
}

impl ChannelPool {
    fn new(size: usize) -> Self {
        let mut pool = Vec::with_capacity(size);
        for _ in 0..size {
            let (tx, rx) = mpsc::channel::<(u16, u16, BytesMut)>(32);
            pool.push(tx);
            tokio::spawn(process_channel(rx));
        }
        Self {
            pool: Arc::new(Mutex::new(pool)),
        }
    }

    fn get_channel(&self) -> Option<Tx> {
        let mut pool = self.pool.lock().unwrap();
        pool.pop()
    }

    fn return_channel(&self, tx: Tx) {
        let mut pool = self.pool.lock().unwrap();
        pool.push(tx);
    }
}

async fn process_channel(mut rx: Rx) {
    while let Some((error_code, cmd, response_data)) = rx.recv().await {
        // 处理消息
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:10301".to_string();
    let listener = TcpListener::bind(&addr).await?;
    let channel_pool = ChannelPool::new(CHANNEL_POOL_SIZE);

    println!("WebSocket Server is running on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let tx = channel_pool.get_channel().expect("No available channels");
        tokio::spawn(handle_connection(stream, tx, channel_pool.clone()));
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, tx: Tx, channel_pool: ChannelPool) {
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during the websocket handshake");

    println!("New WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();

    tokio::spawn(async move {
        while let Some(message) = read.next().await {
            let message = match message {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    return;
                }
            };

            if message.is_binary() {
                let data = message.into_data();
                if data.len() >= 2 {
                    let cmd = BigEndian::read_u16(&data[0..2]);
                    let payload = &data[2..];
                    let message_data = BytesMut::from(payload);

                    println!(
                        "Received message: cmd={}, data={:?}",
                        cmd,
                        &message_data[..]
                    );

                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        let (response_error_code, response_cmd, processed_message) =
                            process_message(cmd, message_data).await;
                        tx_clone
                            .send((response_error_code, response_cmd, processed_message))
                            .await
                            .expect("Error sending processed message");
                    });
                } else {
                    eprintln!("Header too short: {}", data.len());
                }
            }
        }
        channel_pool.return_channel(tx);
        println!("WebSocket connection closed");
    });
}

async fn process_message(cmd: u16, message: BytesMut) -> (u16, u16, BytesMut) {
    let error_code = 0;
    (error_code, cmd, message)
}
