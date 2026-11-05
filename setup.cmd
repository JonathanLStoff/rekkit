@echo off
rem Rekkit setup on Windows (same as ./setup.sh): run this first on a fresh clone. Safe to run again.
rem   setup.cmd           do every step a machine can do
rem   setup.cmd --check   report what is ready; change nothing
rem Exit status: 0 ready, 2 ready except steps that need a person, 1 a step failed.
setlocal
cd /d "%~dp0"
set "RK_PY="
where py >nul 2>nul && set "RK_PY=py -3"
if not defined RK_PY where python >nul 2>nul && set "RK_PY=python"
if not defined RK_PY (
  echo Python 3.11 or newer is required. Install it with: winget install Python.Python.3.13 1>&2
  exit /b 1
)
%RK_PY% scripts\setup.py %*
exit /b %errorlevel%
