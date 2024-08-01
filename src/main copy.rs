use futures_util::{SinkExt, StreamExt};

use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};

pub mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    proto::abc();
    // 绑定监听地址
    let addr = "0.0.0.0:10301".to_string();
    let listener = TcpListener::bind(&addr).await?;

    println!("WebSocket Server is running on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }

    Ok(())
}

// 假设有一个异步处理函数
async fn process_message(message: Message) -> Message {
    // 模拟数据处理逻辑
    message
}

// 处理WebSocket连接的函数
async fn handle_connection(stream: TcpStream) {
    // WebSocket握手
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during the websocket handshake");

    println!("New WebSocket connection established");

    // 拆分WebSocket流为写和读部分
    let (mut write, mut read) = ws_stream.split();

    // 创建一个通道用于进程间通信
    let (tx, mut rx) = mpsc::channel::<Message>(32);

    // 启动一个任务处理通道中接收到的数据
    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            // 将处理后的消息发送回客户端
            println!("send response: {:?}", response);
            write.send(response).await.expect("Error sending message");
        }
    });

    // 读取客户端发送的消息
    while let Some(message) = read.next().await {
        let message = match message {
            Ok(msg) => {
                println!("recived msg: {:?}", msg);
                msg
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                return;
            }
        };

        if message.is_text() || message.is_binary() {
            // 发送消息到处理任务
            let tx = tx.clone();
            tokio::spawn(async move {
                // 异步处理消息
                let processed_message = process_message(message).await;
                // 将处理后的消息发送到通道
                tx.send(processed_message)
                    .await
                    .expect("Error sending processed message");
            });
        }
    }
}
