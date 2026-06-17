@echo off
setlocal

set TARGET=%1
if "%TARGET%"=="" set TARGET=test

if "%TARGET%"=="fmt" goto fmt
if "%TARGET%"=="test" goto test
if "%TARGET%"=="build" goto build
if "%TARGET%"=="smoke" goto smoke
if "%TARGET%"=="clean-smoke" goto clean_smoke

echo Unknown target: %TARGET%
echo Usage: make.bat [fmt^|test^|build^|smoke^|clean-smoke]
exit /b 1

:fmt
cargo fmt --check
exit /b %ERRORLEVEL%

:test
cargo test --workspace
exit /b %ERRORLEVEL%

:build
cargo build --workspace
exit /b %ERRORLEVEL%

:smoke
call :build
if errorlevel 1 exit /b %ERRORLEVEL%
call :clean_smoke
if errorlevel 1 exit /b %ERRORLEVEL%
target\debug\worklog.exe --db target\worklog-smoke.db entry add --title "Make smoke draft" --start 2026-06-16T09:00:00Z --end 2026-06-16T10:00:00Z --actor ai --source codex
if errorlevel 1 exit /b %ERRORLEVEL%
target\debug\worklog.exe --db target\worklog-smoke.db entry list --status draft --format json
if errorlevel 1 exit /b %ERRORLEVEL%
target\debug\worklog.exe --db target\worklog-smoke.db entry confirm 1
if errorlevel 1 exit /b %ERRORLEVEL%
target\debug\worklog.exe --db target\worklog-smoke.db export report-source --start 2026-06-16T00:00:00Z --end 2026-06-17T00:00:00Z --format json
if errorlevel 1 exit /b %ERRORLEVEL%
target\debug\worklog.exe --db target\worklog-smoke.db export report-source --start 2026-06-16T00:00:00Z --end 2026-06-17T00:00:00Z --format markdown
exit /b %ERRORLEVEL%

:clean_smoke
if exist target\worklog-smoke.db del target\worklog-smoke.db
if exist target\worklog-smoke.db-wal del target\worklog-smoke.db-wal
if exist target\worklog-smoke.db-shm del target\worklog-smoke.db-shm
exit /b 0
