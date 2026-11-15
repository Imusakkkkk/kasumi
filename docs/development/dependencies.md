# Dependencies

## Required
- Rust (stable, MSVC)
- CMake, Ninja
- MSVC compiler
- Qt6 (Widgets, Concurrent)
- LLVM/libclang (build-time, for bindgen/unicorn)

## Optional
- Vulkan SDK headers (GPU dev)
- Git submodules: dynarmic, ext-boost (not currently used in build)

## Build-time downloads
- Corrosion (CMake FetchContent)
- unicorn-engine (cargo)