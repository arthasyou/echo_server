use std::collections::HashMap;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::{connection::Connection, socket_events::SocketEvents};

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
}

pub async fn start_loop(mut reciever: UnboundedReceiver<SocketEvents>) {
    let mgr = SocketMgr::new();
    while let Some(event) = reciever.recv().await {}
}
