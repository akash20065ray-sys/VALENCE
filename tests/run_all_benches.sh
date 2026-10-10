#!/usr/bin/env bash
# ==============================================================================
# Valence Automated Comparative Benchmark Suite (Sub-Phase 8.1)
# ==============================================================================
# Executes micro-benchmarks and comparative performance evaluations against
# Redis 7.2 and Dragonfly baselines. Exports metrics to CSV and JSON.

set -e

echo "=============================================================================="
echo "  VALENCE BENCHMARK HARNESS (Pure Rust State Engine)"
echo "=============================================================================="

# 1. Ensure clean build in release profile
echo "[1/3] Compiling Valence in high-performance release mode (opt-level=3, LTO=fat)..."
cargo build --release --bin valence-server --bin valence-cli

# 2. Run Criterion Benchmark Suite
echo "[2/3] Executing Criterion Subsystem Micro-Benchmarks..."
cargo bench --bench suite -- --sample-size 50

# 3. Verify Artifact Generation
echo "[3/3] Verifying Benchmark Export Artifacts..."
if [ -f "target/benchmark_results.csv" ] && [ -f "target/benchmark_results.json" ]; then
    echo "✓ Successfully generated:"
    echo "  - target/benchmark_results.csv"
    echo "  - target/benchmark_results.json"
    echo ""
    echo "Summary Table:"
    cat target/benchmark_results.csv
else
    echo "Error: Benchmark output files were not generated."
    exit 1
fi

echo "=============================================================================="
echo "  BENCHMARK SUITE COMPLETE"
echo "=============================================================================="
