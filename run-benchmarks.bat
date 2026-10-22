@echo off
echo ===============================================================================
echo  Exact Ground State of Frustrated Quantum Spin Glasses (NP-Hard)
echo  Asolaria Balanced Ternary Relaxation ^& 108-Lane Finite-Field NTT Prism
echo ===============================================================================
echo.

where cargo >nul 2>nul
if %ERRORLEVEL% equ 0 (
    echo [*] Running Rust High-Performance Engine...
    cargo run --release
    echo.
) else (
    echo [!] Cargo not detected, falling back to Python...
)

where python >nul 2>nul
if %ERRORLEVEL% equ 0 (
    echo [*] Running Python Verification Suite...
    python python\spin_glass_solver.py
) else (
    echo [!] Python not detected.
)

echo.
pause
