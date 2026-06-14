@echo off
REM Launcher for the universal chunker. Usage: chunk.cmd [args...]
REM e.g.  chunk.cmd --plan "C:\path\to\huge.md"
REM       chunk.cmd --budget 100000 "C:\path\to\huge.md"
python "%~dp0chunker.py" %*
