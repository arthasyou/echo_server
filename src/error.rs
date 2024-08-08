use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // #[error("auth error: {0}")]
    // AuthError(String),
    #[error("service utils error: {0}")]
    JwtError(#[from] service_utils_rs::error::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    // #[error("{message:} ({line:}, {column})")]
    // CustomError {
    //     message: String,
    //     line: u32,
    //     column: u32,
    // },
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
