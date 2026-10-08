//! Server Networking & Engine Module
//!
//! Provides non-blocking TCP socket management, event-driven command processing,
//! and server engine multiplexing.

pub mod connection;
pub mod engine;

pub use connection::{ClientConnection, ServerStats};
pub use engine::ServerEngine;
