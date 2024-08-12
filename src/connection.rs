use byteorder::{BigEndian, ByteOrder};
use bytes::BytesMut;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender, UnboundedSender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::socket_events::SocketEvents;

type SocketWriter = SplitSink<WebSocketStream<TcpStream>, Message>;
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
        .send(SocketEvents::Handshake(msg_sender, connection))
        .unwrap();
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
