include!(concat!(env!("OUT_DIR"), "/ftproto.rs"));

// use prost::Message;

pub fn abc() {
    let a = UserInfoResult::default();
    println!("{:?}", a);
}
