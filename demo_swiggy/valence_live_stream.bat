@echo off
title Valence Live Wire Trace - Listening on Port 6379
color 0a
cls
python "%~dp0valence_terminal.py" --attach
pause
