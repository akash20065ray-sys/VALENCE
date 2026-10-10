//! Wire Protocol Module
//!
//! Exposes RESP (Redis Serialization Protocol) zero-copy parser and frame encoder.

pub mod resp;
pub mod simd;

pub use resp::{Command, ParseError, RespEncoder, RespParser};
pub use simd::find_crlf;
