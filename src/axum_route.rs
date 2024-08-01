use crate::ftproto::{
    BsPlayArg, BsPlayResult, CancelArg, CancelResult, FruitPlayArg, FruitPlayResult, UserInfoArg,
    UserInfoResult,
};
use async_trait::async_trait;
use byteorder::{BigEndian, ByteOrder};
use bytes::BytesMut;
use prost::Message;

type Handler =
    dyn Fn(BytesMut) -> Box<dyn futures::Future<Output = Result<BytesMut, &'static str>> + Send>;

struct Router {
    routes: std::collections::HashMap<u16, Box<Handler>>,
}

impl Router {
    fn new() -> Self {
        let mut routes: std::collections::HashMap<u16, Box<Handler>> =
            std::collections::HashMap::new();
        routes.insert(
            1001,
            Box::new(|payload| Box::new(handle_user_info(payload))),
        );
        routes.insert(
            2001,
            Box::new(|payload| Box::new(handle_fruit_play(payload))),
        );
        routes.insert(2002, Box::new(|payload| Box::new(handle_bs_play(payload))));
        routes.insert(2003, Box::new(|payload| Box::new(handle_cancel(payload))));
        Self { routes }
    }

    async fn route(&self, cmd: u16, data: BytesMut) -> Result<BytesMut, &'static str> {
        if let Some(handler) = self.routes.get(&cmd) {
            handler(data).await
        } else {
            Err("Unknown command")
        }
    }
}

async fn serialize_response<T: Message>(response: T) -> Result<BytesMut, &'static str> {
    let mut buf = BytesMut::with_capacity(response.encoded_len());
    response
        .encode(&mut buf)
        .map_err(|_| "Failed to encode response")?;
    Ok(buf)
}

async fn handle_user_info(payload: BytesMut) -> Result<BytesMut, &'static str> {
    let arg = UserInfoArg::decode(&payload[..]).map_err(|_| "Failed to decode UserInfoArg")?;
    // 处理逻辑...
    let result = UserInfoResult {
        user_id: 1,
        name: "Player".to_string(),
        balance: 1000,
        icon: "icon.png".to_string(),
    };
    serialize_response(result).await
}

async fn handle_fruit_play(payload: BytesMut) -> Result<BytesMut, &'static str> {
    let arg = FruitPlayArg::decode(&payload[..]).map_err(|_| "Failed to decode FruitPlayArg")?;
    // 处理逻辑...
    let result = FruitPlayResult {
        lights: vec![0, 1, 2],
        fruits: vec![],
        odds: 2,
        part: vec![],
        win: 500,
        balance: 1500,
    };
    serialize_response(result).await
}

async fn handle_bs_play(payload: BytesMut) -> Result<BytesMut, &'static str> {
    let arg = BsPlayArg::decode(&payload[..]).map_err(|_| "Failed to decode BsPlayArg")?;
    // 处理逻辑...
    let result = BsPlayResult {
        result: 7,
        win: 200,
        balance: 1200,
    };
    serialize_response(result).await
}

async fn handle_cancel(payload: BytesMut) -> Result<BytesMut, &'static str> {
    let arg = CancelArg::decode(&payload[..]).map_err(|_| "Failed to decode CancelArg")?;
    // 处理逻辑...
    let result = CancelResult { balance: 1000 };
    serialize_response(result).await
}

#[tokio::main]
async fn main() {
    let router = Router::new();
    // 示例调用
    let cmd = 1001;
    let data = BytesMut::new();
    match router.route(cmd, data).await {
        Ok(response) => println!("Response: {:?}", response),
        Err(err) => println!("Error: {:?}", err),
    }
}
