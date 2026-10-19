@echo off
title VALENCE Interactive Database CLI (Port 6379)
color 0E
cls
cd /d "c:\project_placement"

if exist "valence_terminal.py" (
    python valence_terminal.py
) else (
    echo [ERROR] valence_terminal.py not found in c:\project_placement!
)
pause
