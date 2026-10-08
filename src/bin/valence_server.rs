//! Valence Production Server Binary
//!
//! Launches the non-blocking single-threaded engine daemon.

use valence::ServerEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("VALENCE_PORT").unwrap_or_else(|_| "6379".to_string());
    let addr = format!("127.0.0.1:{}", port);

    println!("============================================================");
    println!("  VALENCE IN-MEMORY STATE ENGINE (Pure Rust)");
    println!("  Engine Version: 0.1.0");
    println!("  Target Port:    {}", port);
    println!("  Architecture:   Thread-per-core, Forkless Epoch Snapshots");
    println!("============================================================");

    let mut engine = ServerEngine::bind(&addr)?;
    engine.run()?;

    Ok(())
}
