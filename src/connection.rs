use futures_util::stream::SplitSink;
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::socket_events::SocketEvents;

type SocketWriter = SplitSink<WebSocketStream<TcpStream>, Message>;

#[derive(Debug)]
pub struct Connection {
    pub id: u32,
    pub socket_writer: SocketWriter,
}

impl Connection {
    pub fn new(id: u32, socket_writer: SocketWriter) -> Self {
        Self { id, socket_writer }
    }
}

pub async fn handle_connection(
    ws_stream: WebSocketStream<TcpStream>,
    sender: UnboundedSender<SocketEvents>,
    token_info: String,
) {
    println!("token info: {}", token_info);
}
