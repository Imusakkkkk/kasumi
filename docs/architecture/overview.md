# Kasumi Architecture Overview

Kasumi is an experimental Nintendo Switch 2 emulator written primarily in Rust, with a Qt 6 C++ frontend.

## Components

- **Rust core** (`core/`): emulator library `oboromi-core` (crate-type rlib + staticlib).
- **Qt frontend** (`gui-qt/`): C++17 + Qt 6 (Widgets, Concurrent).
- **FFI bridge** (`core/src/ffi.rs` ↔ `gui-qt/src/RustBridge`): C ABI boundary.
- **Build system**: CMake + Corrosion (Rust integration) + Ninja.
- **Dependencies**: unicorn-engine (CPU), ash (Vulkan), Qt6, LLVM/libclang (build-time).

```mermaid
flowchart LR
  GUI[Qt GUI (C++)] -- FFI (C ABI) --> Core[Rust Core]
  Core --> CPU[CPU: Unicorn ARM64]
  Core --> GPU[GPU: Vulkan/SM86]
  Core --> FS[FS]
  Core --> NN[HIPC/NN]
  Core --> SYS[SYS]
  Build[CMake + Corrosion] --> Core
  Build --> GUI
```

## Status
- CPU: functional (Unicorn), no ARMv9 extensions yet.
- GPU: experimental Vulkan init + SM86 decoder.
- Audio: not implemented.
- Memory: flat 12 GB allocation.
- Threading: not implemented.