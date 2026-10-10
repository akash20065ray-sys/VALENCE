//! Valence Production Server Binary
//!
//! Launches the non-blocking thread-per-core sharded engine daemon.

use valence::{CpuTopology, ShardedServerEngine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("VALENCE_PORT").unwrap_or_else(|_| "6379".to_string());
    let addr = format!("127.0.0.1:{}", port);
    let topology = CpuTopology::discover();
    let num_shards = std::env::var("VALENCE_SHARDS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(topology.total_cores.max(1));
    let data_dir = std::env::var("VALENCE_DATA_DIR").unwrap_or_else(|_| "./data".to_string());

    println!("============================================================");
    println!("  VALENCE IN-MEMORY STATE ENGINE (Pure Rust)");
    println!("  Engine Version: 0.1.0");
    println!("  Target Port:    {}", port);
    println!("  Hardware Cores: {}", topology.total_cores);
    println!("  Active Shards:  {}", num_shards);
    println!("  Data Directory: {}", data_dir);
    println!("  Architecture:   Thread-per-core, Forkless Epoch Snapshots");
    println!("============================================================");

    let mut engine = ShardedServerEngine::bind_with_shards(&addr, num_shards)?
        .with_data_dir(data_dir);
    engine.run()?;

    Ok(())
}
