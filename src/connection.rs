use core::sync;

use byteorder::{BigEndian, ByteOrder};
use bytes::BytesMut;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender, UnboundedSender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::socket_events::SocketEvents;

type SocketWriter = SplitSink<WebSocketStream<TcpStream>, Message>;
type SocketReader = SplitStream<WebSocketStream<TcpStream>>;
type MsgSender = Sender<(u16, u16, BytesMut)>;

#[derive(Debug)]
pub struct Connection {
    pub id: u32,
    pub name: String,
    pub msg_sender: MsgSender,
}

impl Connection {
    pub fn new(name: String, msg_sender: MsgSender) -> Self {
        Self {
            id: 0,
            name,
            msg_sender,
        }
    }
}

pub async fn handle_connection(
    ws_stream: WebSocketStream<TcpStream>,
    mgr_sender: UnboundedSender<SocketEvents>,
    token_info: String,
) {
    println!("token info: {}", token_info);
    let (mut socket_writer, mut socket_reader) = ws_stream.split();

    // 创建一个通道用于进程间通信
    let (msg_sender, rx) = mpsc::channel::<(u16, u16, BytesMut)>(4);

    let connection = Connection::new(token_info, msg_sender.clone());

    tokio::spawn(recieve_msg(rx, socket_writer));

    mgr_sender
        .send(SocketEvents::Handshake(connection))
        .unwrap();

    handle_msg(socket_reader, msg_sender, mgr_sender).await;
}

async fn recieve_msg(mut rx: Receiver<(u16, u16, BytesMut)>, mut writer: SocketWriter) {
    while let Some((error_code, cmd, response_data)) = rx.recv().await {
        let mut header = [0u8; 4];
        BigEndian::write_u16(&mut header[0..2], error_code);
        BigEndian::write_u16(&mut header[2..4], cmd);
        let mut message = BytesMut::with_capacity(4 + response_data.len());
        message.extend_from_slice(&header);
        message.extend_from_slice(&response_data);
        if let Err(e) = writer.send(Message::binary(message.freeze())).await {
            eprintln!("Error sending message: {}", e);
            break;
        }
    }
}

async fn handle_msg(
    mut read: SocketReader,
    tx: MsgSender,
    msg_sender: UnboundedSender<SocketEvents>,
) {
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
    msg_sender.send(SocketEvents::Disconnect(1)).unwrap();
}

// 假设有一个异步处理函数
async fn process_message(cmd: u16, message: BytesMut) -> (u16, u16, BytesMut) {
    // 模拟数据处理逻辑
    let error_code = 0; // 示例错误码/ 示例命令
    (error_code, cmd, message)
}
