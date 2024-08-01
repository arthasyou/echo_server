use crate::ftproto::{
    BsPlayArg, BsPlayResult, CancelArg, CancelResult, FruitPlayArg, FruitPlayResult, UserInfoArg,
    UserInfoResult,
};
use byteorder::{BigEndian, ByteOrder};
use bytes::BytesMut;
use prost::Message;

// 包头长度
const HEADER_LEN: usize = 4;

pub async fn route_message(data: BytesMut) -> Result<BytesMut, &'static str> {
    // 解析包头
    if data.len() < HEADER_LEN {
        return Err("Invalid data length");
    }

    let error_code = BigEndian::read_u16(&data[0..2]);
    let cmd = BigEndian::read_u16(&data[2..4]);

    if error_code != 0 {
        return Err("Error in message");
    }

    let payload = &data[4..];
    let response = match cmd {
        1001 => handle_user_info(payload).await?,
        2001 => handle_fruit_play(payload).await?,
        2002 => handle_bs_play(payload).await?,
        2003 => handle_cancel(payload).await?,
        _ => return Err("Unknown command"),
    };

    Ok(response)
}

async fn handle_user_info(payload: &[u8]) -> Result<BytesMut, &'static str> {
    let arg = UserInfoArg::decode(payload).map_err(|_| "Failed to decode UserInfoArg")?;
    // 处理逻辑...
    let result = UserInfoResult {
        user_id: 1,
        name: "Player".to_string(),
        balance: 1000,
        icon: "icon.png".to_string(),
    };

    let mut buf = BytesMut::with_capacity(result.encoded_len());
    result
        .encode(&mut buf)
        .map_err(|_| "Failed to encode UserInfoResult")?;
    Ok(buf)
}

async fn handle_fruit_play(payload: &[u8]) -> Result<BytesMut, &'static str> {
    let arg = FruitPlayArg::decode(payload).map_err(|_| "Failed to decode FruitPlayArg")?;
    // 处理逻辑...
    let result = FruitPlayResult {
        lights: vec![0, 1, 2],
        fruits: vec![],
        odds: 2,
        part: vec![],
        win: 500,
        balance: 1500,
    };

    let mut buf = BytesMut::with_capacity(result.encoded_len());
    result
        .encode(&mut buf)
        .map_err(|_| "Failed to encode FruitPlayResult")?;
    Ok(buf)
}

async fn handle_bs_play(payload: &[u8]) -> Result<BytesMut, &'static str> {
    let arg = BsPlayArg::decode(payload).map_err(|_| "Failed to decode BsPlayArg")?;
    // 处理逻辑...
    let result = BsPlayResult {
        result: 7,
        win: 200,
        balance: 1200,
    };

    let mut buf = BytesMut::with_capacity(result.encoded_len());
    result
        .encode(&mut buf)
        .map_err(|_| "Failed to encode BsPlayResult")?;
    Ok(buf)
}

async fn handle_cancel(payload: &[u8]) -> Result<BytesMut, &'static str> {
    let arg = CancelArg::decode(payload).map_err(|_| "Failed to decode CancelArg")?;
    // 处理逻辑...
    let result = CancelResult { balance: 1000 };

    let mut buf = BytesMut::with_capacity(result.encoded_len());
    result
        .encode(&mut buf)
        .map_err(|_| "Failed to encode CancelResult")?;
    Ok(buf)
}
