use byteorder::{BigEndian, ByteOrder}; // 用于解析字节序
use bytes::BytesMut;
use connection::Connection;
use futures_util::{SinkExt, StreamExt};
use socket_events::SocketEvents;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::http::uri;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::handshake::server::{Request, Response},
    tungstenite::Message,
};

pub mod ftproto {
    include!(concat!(env!("OUT_DIR"), "/ftproto.rs"));
}
mod router;

mod connection;
mod socket_events;
mod socket_mgr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 绑定监听地址
    let addr = "0.0.0.0:10301".to_string();
    let listener = TcpListener::bind(&addr).await?;

    println!("WebSocket Server is running on ws://{}", addr);

    let (sender, receiver) = mpsc::unbounded_channel::<SocketEvents>();

    tokio::spawn(socket_mgr::start_loop(receiver));

    while let Ok((stream, client_addr)) = listener.accept().await {
        let mut token_info = String::new();
        match accept_hdr_async(stream, |req: &Request, mut res: Response| {
            let query = req.uri().query().unwrap_or("");
            token_info.push_str(query);
            Ok(res)
        })
        .await
        {
            Err(e) => println!("Websocket connection error : {}", e),
            Ok(ws_stream) => {
                println!("New client addr: {}", client_addr);
                // tokio::spawn(handle_connection(ws_stream, token_info));
                tokio::spawn(connection::handle_connection(
                    ws_stream,
                    sender.clone(),
                    token_info,
                ));
            }
        }
    }

    Ok(())
}

// 假设有一个异步处理函数
async fn process_message(cmd: u16, message: BytesMut) -> (u16, u16, BytesMut) {
    // 模拟数据处理逻辑
    let error_code = 0; // 示例错误码/ 示例命令
    (error_code, cmd, message)
}

// 处理WebSocket连接的函数
async fn handle_connection(stream: WebSocketStream<TcpStream>) {
    // 拆分WebSocket流为写和读部分
    let (mut write, mut read) = stream.split();

    // 创建一个通道用于进程间通信
    let (tx, mut rx) = mpsc::channel::<(u16, u16, BytesMut)>(32);

    // 启动一个任务处理通道中接收到的数据
    tokio::spawn(async move {
        while let Some((error_code, cmd, response_data)) = rx.recv().await {
            // 将处理后的消息发送回客户端
            println!(
                "send response: errorCode={}, cmd={}, data={:?}",
                error_code, cmd, response_data
            );

            // 创建消息头
            let mut header = [0u8; 4];
            BigEndian::write_u16(&mut header[0..2], error_code);
            BigEndian::write_u16(&mut header[2..4], cmd);

            // 拼接消息头和数据
            let mut message = BytesMut::with_capacity(4 + response_data.len());
            message.extend_from_slice(&header);
            message.extend_from_slice(&response_data);

            // 发送消息
            write
                .send(Message::binary(message.freeze()))
                .await
                .expect("Error sending message");
        }
    });

    // 读取客户端发送的消息
    while let Some(message) = read.next().await {
        let message = match message {
            Ok(msg) => {
                println!("received msg: {:?}", msg);
                msg
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                return;
            }
        };

        if message.is_binary() {
            let data = message.into_data();

            if data.len() >= 2 {
                // 解析包头
                let cmd = BigEndian::read_u16(&data[0..2]);

                // 提取数据部分
                let payload = &data[2..];
                let message_data = BytesMut::from(payload);

                println!(
                    "Received message:  cmd={}, data={:?}",
                    // error_code,
                    cmd,
                    &message_data[..]
                );

                // 发送消息到处理任务
                let tx = tx.clone();
                tokio::spawn(async move {
                    // 异步处理消息
                    let (response_error_code, response_cmd, processed_message) =
                        process_message(cmd, message_data).await;
                    // 将处理后的消息发送到通道
                    tx.send((response_error_code, response_cmd, processed_message))
                        .await
                        .expect("Error sending processed message");
                });
            } else {
                eprintln!("Header too short: {}", data.len());
            }
        }
    }

    // 连接关闭时，移除客户端
    println!("WebSocket connection closed");
}
