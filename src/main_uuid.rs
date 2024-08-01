use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

type Tx = mpsc::Sender<Message>;
type PeerMap = Arc<Mutex<HashMap<Uuid, Tx>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:10301".to_string();
    let listener = TcpListener::bind(&addr).await?;

    println!("WebSocket Server is running on ws://{}", addr);

    let peer_map: PeerMap = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let peer_map = Arc::clone(&peer_map);
        tokio::spawn(handle_connection(peer_map, stream));
    }

    Ok(())
}

async fn handle_connection(peer_map: PeerMap, stream: TcpStream) {
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during the websocket handshake");

    let id = Uuid::new_v4();
    println!("New WebSocket connection established: {}", id);

    let (mut write, mut read) = ws_stream.split();

    let (tx, mut rx) = mpsc::channel::<Message>(32);
    {
        let mut peers = peer_map.lock().unwrap();
        peers.insert(id, tx);
    }

    // 每个连接独立的任务处理通道中接收到的数据
    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            write.send(response).await.expect("Error sending message");
        }
    });

    while let Some(message) = read.next().await {
        let message = match message {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        };

        if message.is_text() || message.is_binary() {
            let tx = {
                let peers = peer_map.lock().unwrap();
                peers.get(&id).unwrap().clone()
            };
            tokio::spawn(async move {
                let processed_message = process_message(message).await;
                tx.send(processed_message)
                    .await
                    .expect("Error sending processed message");
            });
        }
    }

    // 连接关闭时，移除客户端
    {
        let mut peers = peer_map.lock().unwrap();
        peers.remove(&id);
    }

    println!("WebSocket connection closed: {}", id);
}

// 模拟的异步处理函数
async fn process_message(message: Message) -> Message {
    // 模拟数据处理逻辑
    message
}
