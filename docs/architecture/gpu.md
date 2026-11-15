# GPU Subsystem

## Current Implementation
- Vulkan instance/device with compute queue (`core/src/gpu/mod.rs`).
- NVIDIA SM86 targeting.
- SM86 instruction decoder generated (`sm86_decoder_generated.rs`).
- SPIR-V translation module (`spirv.rs`).

## Missing
- Command processing, resource management, pipelines, textures, framebuffers, rendering.

## Tests
- `gpu_test.rs` (decoder/translation), `vk_state_new_or_skip`.