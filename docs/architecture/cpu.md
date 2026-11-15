# CPU Subsystem

## Current Implementation
- Unicorn Engine ARM64 (little-endian) via `core/src/cpu/unicorn_interface.rs`.
- `CpuManager` creates 8 cores sharing a single 12 GB memory region.
- `run_all` steps cores sequentially (round-robin), no threading.

## Limitations
- No ARMv9-specific extensions verified.
- No SIMD/FP handling beyond Unicorn.
- No exception/interrupt model.
- No MMU/address translation.

## Future
- Migrate to rapid-arm-emu (research phase, not integrated).