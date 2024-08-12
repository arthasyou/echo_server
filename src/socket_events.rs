use crate::connection::Connection;
use bytes::BytesMut;
use tokio::sync::mpsc::Sender;

pub enum SocketEvents {
    Handshake(Sender<(u16, u16, BytesMut)>, Connection),
    Disconnect(u32),
}
