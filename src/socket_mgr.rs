use crate::constant::error_code::SYSTEM_ERROR;
use crate::error::{Error, Result};
use crate::{connection::Connection, socket_events::SocketEvents};
use bytes::BytesMut;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedReceiver;

pub struct SocketMgr {
    connections: HashMap<u32, Connection>,
    max_client: u32,
    connection_indices: u32,
    connection_index_pool: Vec<u32>,
}

impl SocketMgr {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            max_client: 1000,
            connection_indices: 0,
            connection_index_pool: Vec::new(),
        }
    }

    pub fn add_connection(&mut self, name: String) -> Result<u32> {
        let id = if !self.connection_index_pool.is_empty() {
            self.connection_index_pool.pop().unwrap()
        } else if self.connection_indices < self.max_client {
            self.connection_indices += 1;
            self.connection_indices
        } else {
            0
        };

        if id == 0 {
            return Err(Error::ErrorCode(SYSTEM_ERROR));
        }

        Ok(id)
    }
}

pub async fn start_loop(mut reciever: UnboundedReceiver<SocketEvents>) {
    let mut mgr = SocketMgr::new();
    while let Some(event) = reciever.recv().await {
        match event {
            SocketEvents::Handshake(tx, mut conn) => match mgr.add_connection(conn.name.clone()) {
                Ok(id) => {
                    conn.id = id;
                    mgr.connections.insert(id, conn);

                    // if tx.send(id).is_ok() {
                    //     mgr.broadcast_lobby_info().await;
                    // } else {
                    //     mgr.connections.remove(&id).unwrap();
                    // }
                }
                Err(Error::ErrorCode(code)) => {
                    let msg = BytesMut::new();
                    let _ = conn.msg_sender.send((code, 0, msg));
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            },
            SocketEvents::Disconnect(_) => todo!(),
        }
    }
}
