# Windows Build (Verified)

## Environment
- Windows 11 x64
- Rust 1.100.0 stable (x86_64-pc-windows-msvc)
- CMake 4.4.4
- Ninja 1.13.2
- MSVC 19.52.37029 (VS 2026)
- Qt 6.12.0 MSVC 64-bit (`C:/Qt/6.12.0/msvc2022_64`)
- LLVM/libclang 23.1.5 (`C:/Program Files/LLVM`)

## Commands
```
cmake -B build -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_PREFIX_PATH="C:/Qt/6.12.0/msvc2022_64"
cmake --build build --config Release
```
Run from Developer PowerShell; build elevated due to Unicorn symlink creation.

## Known Issue
Unicorn build creates a symlink requiring Developer Mode or admin. Enable Windows Developer Mode to avoid admin.