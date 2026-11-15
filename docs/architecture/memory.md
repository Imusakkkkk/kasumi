# Memory Subsystem

## Current Implementation
- Single 12 GB zeroed `Vec` allocated eagerly (Windows commits up front).
- Flat mapping at base 0 via Unicorn `mem_map_ptr`.
- Shared pointer passed to all cores.

## Limitations
- No page tables, TLB, MMU, or protection.
- No virtual/physical translation.
- CPU/GPU shared memory not managed.

## Future
- Lazy allocation (memmap2), MMU, address translation.