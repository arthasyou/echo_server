use crate::ftproto::{
    BsPlayArg, BsPlayResult, CancelArg, CancelResult, FruitPlayArg, FruitPlayResult, UserInfoArg,
    UserInfoResult,
};
use bytes::BytesMut;
use prost::Message;

pub async fn route_message(cmd: u16, data: BytesMut) -> Result<BytesMut, &'static str> {
    let payload = &data[..];
    match cmd {
        1001 => handle_user_info(payload).await.map(serialize_response),
        2001 => handle_fruit_play(payload).await.map(serialize_response),
        2002 => handle_bs_play(payload).await.map(serialize_response),
        2003 => handle_cancel(payload).await.map(serialize_response),
        _ => Err("Unknown command"),
    }
    .and_then(|res| res)
}

fn serialize_response<T: Message>(response: T) -> Result<BytesMut, &'static str> {
    let mut buf = BytesMut::with_capacity(response.encoded_len());
    response
        .encode(&mut buf)
        .map_err(|_| "Failed to encode response")?;
    Ok(buf)
}

async fn handle_user_info(payload: &[u8]) -> Result<UserInfoResult, &'static str> {
    let arg = UserInfoArg::decode(payload).map_err(|_| "Failed to decode UserInfoArg")?;
    // 处理逻辑...
    Ok(UserInfoResult {
        user_id: 1,
        name: "Player".to_string(),
        balance: 1000,
        icon: "icon.png".to_string(),
    })
}

async fn handle_fruit_play(payload: &[u8]) -> Result<FruitPlayResult, &'static str> {
    let arg = FruitPlayArg::decode(payload).map_err(|_| "Failed to decode FruitPlayArg")?;
    // 处理逻辑...
    Ok(FruitPlayResult {
        lights: vec![0, 1, 2],
        fruits: vec![],
        odds: 2,
        part: vec![],
        win: 500,
        balance: 1500,
    })
}

async fn handle_bs_play(payload: &[u8]) -> Result<BsPlayResult, &'static str> {
    let arg = BsPlayArg::decode(payload).map_err(|_| "Failed to decode BsPlayArg")?;
    // 处理逻辑...
    Ok(BsPlayResult {
        result: 7,
        win: 200,
        balance: 1200,
    })
}

async fn handle_cancel(payload: &[u8]) -> Result<CancelResult, &'static str> {
    let arg = CancelArg::decode(payload).map_err(|_| "Failed to decode CancelArg")?;
    // 处理逻辑...
    Ok(CancelResult { balance: 1000 })
}
