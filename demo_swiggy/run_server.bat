@echo off
title VALENCE In-Memory Database Server (Port 6379)
color 0A
cls
echo ============================================================
echo   VALENCE IN-MEMORY STATE ENGINE (Pure Rust v1.0)
echo   Listening on 127.0.0.1:6379
echo   Architecture: 12 CPU-Pinned Shards ^| Lock-Free Arenas
echo ============================================================
echo.
echo [*] Checking port 6379 availability...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :6379 ^| findstr LISTENING') do (
    echo [*] Releasing existing background process %%a...
    taskkill /f /pid %%a >nul 2>&1
)
cd /d "C:\VALENCE"
set VALENCE_PORT=6379
set VALENCE_SHARDS=12
set VALENCE_DATA_DIR=C:\VALENCE\data
echo [✓] Launching Valence Server across 12 hardware cores...
echo.
"C:\VALENCE\target\release\valence-server.exe"
pause
