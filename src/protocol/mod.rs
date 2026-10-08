//! Wire Protocol Module
//!
//! Exposes RESP (Redis Serialization Protocol) zero-copy parser and frame encoder.

pub mod resp;

pub use resp::{Command, ParseError, RespEncoder, RespParser};
