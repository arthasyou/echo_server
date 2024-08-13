use crate::constant::error_code::SYSTEM_ERROR;
use crate::error::{Error, Result};
use crate::ftproto::{
    BsPlayArg, BsPlayResult, CancelArg, CancelResult, FruitPlayArg, FruitPlayResult, UserInfoArg,
    UserInfoResult,
};
use bytes::BytesMut;
use prost::Message;

pub async fn route_message(cmd: u16, data: BytesMut) -> Result<BytesMut> {
    let payload = &data[..];
    match cmd {
        1001 => handle_user_info(payload).await.map(serialize_response),
        2001 => handle_fruit_play(payload).await.map(serialize_response),
        2002 => handle_bs_play(payload).await.map(serialize_response),
        2003 => handle_cancel(payload).await.map(serialize_response),
        _ => Err(Error::ErrorCode(SYSTEM_ERROR)), // Unknown command error code
    }
    .and_then(|res| res)
}

fn serialize_response<T: Message>(response: T) -> Result<BytesMut> {
    let mut buf = BytesMut::with_capacity(response.encoded_len());
    response.encode(&mut buf)?;
    Ok(buf)
}

async fn handle_user_info(payload: &[u8]) -> Result<UserInfoResult> {
    let arg = UserInfoArg::decode(payload)?;

    // Handle logic...
    Ok(UserInfoResult {
        user_id: 1,
        name: "Player".to_string(),
        balance: 1000,
        icon: "icon.png".to_string(),
    })
}

// Implement the other handlers similarly...

async fn handle_fruit_play(payload: &[u8]) -> Result<FruitPlayResult> {
    let arg = FruitPlayArg::decode(payload)?;
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

async fn handle_bs_play(payload: &[u8]) -> Result<BsPlayResult> {
    let arg = BsPlayArg::decode(payload)?;
    // 处理逻辑...
    Ok(BsPlayResult {
        result: 7,
        win: 200,
        balance: 1200,
    })
}

async fn handle_cancel(payload: &[u8]) -> Result<CancelResult> {
    let arg = CancelArg::decode(payload)?;
    // 处理逻辑...
    Ok(CancelResult { balance: 1000 })
}
