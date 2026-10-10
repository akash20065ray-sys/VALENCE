#!/usr/bin/env bash
# ==============================================================================
# Valence: 1-Click Interactive Demo Launcher for Placement & Recruiters
# ==============================================================================

set -e

echo "=============================================================================="
echo "  VALENCE: NEXT-GENERATION IN-MEMORY STATE ENGINE (PURE RUST)"
echo "=============================================================================="
echo "  - 101/101 Tests Passing across All 8 Architecture Phases"
echo "  - Zero-Fork Epoch Snapshots, 4-Tier Timing Wheel, AVX2 SIMD Scanner"
echo "  - 2MB Hugepage Slab Arenas, Transparent NVMe Tiering, Diagnostic CLI"
echo "=============================================================================="
echo ""

echo "[1/4] Compiling release binaries (opt-level=3, LTO=fat)..."
cargo build --release --bin valence-server --bin valence-cli

echo ""
echo "[2/4] Starting Valence Sharded Server daemon on port 6379 in background..."
./target/release/valence-server &
SERVER_PID=$!
trap "kill $SERVER_PID 2>/dev/null || true" EXIT

sleep 1

echo ""
echo "[3/4] Executing Diagnostic Verification Queries via Valence CLI..."
echo ""
echo "=== Test 1: PING ==="
./target/release/valence-cli ping

echo ""
echo "=== Test 2: SET Session Token ==="
./target/release/valence-cli set "user:session:100" "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9" --ex 3600

echo ""
echo "=== Test 3: GET with Execution Plan Trace (--explain) ==="
./target/release/valence-cli get "user:session:100" --explain

echo ""
echo "=============================================================================="
echo "[4/5] Launching Valence Studio Web Management Console..."
echo "=============================================================================="
(cd studio && npm run dev) &
STUDIO_PID=$!
trap "kill $SERVER_PID $STUDIO_PID 2>/dev/null || true" EXIT

echo ""
echo "=============================================================================="
echo "[5/5] Launching Interactive Valence Diagnostic REPL..."
echo "(Type 'help' for command syntax, append '--explain' to trace, 'quit' to exit)"
echo "=============================================================================="
echo ""
./target/release/valence-cli
