@echo off
call "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvarsall.bat" x64 >nul 2>&1
set "LIB=c:\project_placement\win_lib;%LIB%"
"C:\Users\akash\.cargo\bin\cargo.exe" %*
