@echo off
rem ==============================================================================
rem Valence: 1-Click Interactive Demo Launcher for Placement & Recruiters
rem ==============================================================================

color 0A
cls
echo ==============================================================================
echo   VALENCE: NEXT-GENERATION IN-MEMORY STATE ENGINE (PURE RUST)
echo ==============================================================================
echo   - 101/101 Tests Passing across All 8 Architecture Phases
echo   - Zero-Fork Epoch Snapshots, 4-Tier Timing Wheel, AVX2 SIMD Scanner
echo   - 2MB Hugepage Slab Arenas, Transparent NVMe Tiering, Diagnostic CLI
echo ==============================================================================
echo.

echo [1/4] Compiling release binaries (opt-level=3, LTO=fat)...
cargo build --release --bin valence-server --bin valence-cli
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Build failed.
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo [2/4] Starting Valence Sharded Server daemon on port 6379...
start "Valence Engine Server" cmd /k "target\release\valence-server.exe"

rem Wait 1 second for port binding
timeout /t 1 /nobreak >nul

echo.
echo [3/4] Executing Diagnostic Verification Queries via Valence CLI...
echo.
echo === Test 1: PING ===
target\release\valence-cli.exe ping

echo.
echo === Test 2: SET Session Token ===
target\release\valence-cli.exe set "user:session:100" "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9" --ex 3600

echo.
echo === Test 3: GET with Execution Plan Trace (--explain) ===
target\release\valence-cli.exe get "user:session:100" --explain

echo.
echo.
echo ==============================================================================
echo [4/5] Launching Valence Studio Web Management Console...
echo ==============================================================================
start "Valence Studio Dashboard" cmd /c "cd studio && npm run dev"

echo.
echo ==============================================================================
echo [5/5] Launching Interactive Valence Diagnostic REPL...
echo (Type 'help' for command syntax, append '--explain' to trace, 'quit' to exit)
echo ==============================================================================
echo.
target\release\valence-cli.exe
