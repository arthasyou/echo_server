use crate::error::Result;
use crate::route::{Handler, Router};
use bytes::BytesMut;

mod constant;
mod error;
mod route;

// 定义一个处理函数
async fn handle_user_info(data: BytesMut) -> Result<BytesMut> {
    // 模拟处理逻辑
    let response = BytesMut::from("User Info: John Doe");
    Ok(response)
}

// 定义另一个处理函数
async fn handle_order(data: BytesMut) -> Result<BytesMut> {
    // 模拟处理逻辑
    let response = BytesMut::from("Order: #12345");
    Ok(response)
}

#[tokio::main]
async fn main() {
    // 创建路由器
    let mut router = Router::new();

    // 添加路由
    router
        .add_route(1001, handle_user_info)
        .add_route(1002, handle_order);

    // 模拟处理消息
    let user_info = router
        .handle_message(1001, BytesMut::from(""))
        .await
        .unwrap();
    println!("{}", String::from_utf8_lossy(&user_info));

    let order_info = router
        .handle_message(1002, BytesMut::from(""))
        .await
        .unwrap();
    println!("{}", String::from_utf8_lossy(&order_info));
}
