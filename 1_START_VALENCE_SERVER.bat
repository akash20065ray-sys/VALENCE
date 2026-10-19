@echo off
title VALENCE Core Database Engine (Port 6379)
color 0A
cls
echo ============================================================
echo   VALENCE IN-MEMORY STATE ENGINE (Pure Rust v1.0)
echo   Target Port:   127.0.0.1:6379 (12 CPU-Pinned Shards)
echo   Architecture:  Thread-per-core, Lock-Free Arenas, O(1) Wheel
echo ============================================================
echo.
echo [*] Cleaning up any previous instances...
taskkill /F /IM valence-server.exe >nul 2>&1
ping 127.0.0.1 -n 2 >nul

cd /d "c:\project_placement"

if exist "valence-server.exe" (
    echo [OK] Starting Valence Database Server...
    echo [OK] Listening on 127.0.0.1:6379
    echo.
    valence-server.exe
) else if exist "C:\VALENCE\target\release\valence-server.exe" (
    echo [OK] Starting Valence Database Server...
    echo [OK] Listening on 127.0.0.1:6379
    echo.
    "C:\VALENCE\target\release\valence-server.exe"
) else (
    echo [ERROR] valence-server.exe not found!
)

echo.
echo ============================================================
echo Server stopped.
echo ============================================================
pause
