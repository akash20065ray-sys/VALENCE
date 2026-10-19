@echo off
title Valence Live Query Waterfall Stream (1,000 Queries)
color 0b
cls
echo ==============================================================================
echo   VALENCE: LIVE INDIVIDUAL QUERY TELEMETRY WATERFALL (1,000 QUERIES)
echo   Streaming 1,000 real queries with individual responses, latencies and CPU cores!
echo ==============================================================================
echo.
python "%~dp0query_waterfall.py" 1000
echo.
pause
