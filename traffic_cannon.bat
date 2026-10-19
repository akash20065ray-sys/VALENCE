@echo off
title Valence Live Traffic Cannon (5,000 Queries)
color 0e
cls
echo ==============================================================================
echo   VALENCE SYSTEMS: LIVE MULTI-TENANT TRAFFIC CANNON
echo   Firing 5,000 real production queries at 127.0.0.1:6379
echo ==============================================================================
echo.
python "%~dp0traffic_cannon.py" 5000
echo.
pause
