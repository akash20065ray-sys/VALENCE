@echo off
rem ==============================================================================
rem Valence Automated Comparative Benchmark Suite (Windows Launcher)
rem ==============================================================================

echo ==============================================================================
echo   VALENCE BENCHMARK HARNESS (Pure Rust State Engine)
echo ==============================================================================

echo [1/3] Compiling Valence in high-performance release mode (opt-level=3, LTO=fat)...
cargo build --release --bin valence-server --bin valence-cli
if %ERRORLEVEL% NEQ 0 (
    echo Error during build.
    exit /b %ERRORLEVEL%
)

echo [2/3] Executing Criterion Subsystem Micro-Benchmarks...
cargo bench --bench suite -- --sample-size 50
if %ERRORLEVEL% NEQ 0 (
    echo Error during cargo bench.
    exit /b %ERRORLEVEL%
)

echo [3/3] Verifying Benchmark Export Artifacts...
if exist target\benchmark_results.csv (
    echo Successfully generated target\benchmark_results.csv and target\benchmark_results.json
    type target\benchmark_results.csv
) else (
    echo Warning: target\benchmark_results.csv not found.
)

echo ==============================================================================
echo   BENCHMARK SUITE COMPLETE
echo ==============================================================================
