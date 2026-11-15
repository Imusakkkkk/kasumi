use crate::cpu::backend::CpuBackend;
use crate::cpu::unicorn_interface::UnicornCPU;
use crate::fs::nso::LoadedNso;
use crate::memory::translation::{AddressSpace, MemoryRegion, Permissions};
use crate::memory::translation::{GuestVirtAddr, GuestPhysAddr};
use std::pin::Pin;

pub const CORE_COUNT: usize = 8;

#[cfg(not(target_pointer_width = "64"))]
compile_error!("oboromi requires a 64-bit architecture to emulate 12GB of RAM.");
// 12GB Memory
pub const MEMORY_SIZE: u64 = 12 * 1024 * 1024 * 1024;
pub const MEMORY_BASE: u64 = 0x0;

pub struct CpuManager {
    pub cores: Vec<Box<dyn CpuBackend>>,
    // Pin prevents reallocation from invalidating pointers
    #[allow(dead_code)]
    pub shared_memory: Pin<Box<[u8]>>,
    pub address_space: AddressSpace,
}

impl CpuManager {
    pub fn new() -> Self {
        Self::with_memory_size(MEMORY_SIZE)
    }

    pub fn with_memory_size(size: u64) -> Self {
        Self::with_memory_size_and_cores(size, CORE_COUNT)
    }

    pub fn with_memory_size_and_cores(size: u64, cores: usize) -> Self {
        let shared_memory = Pin::new(vec![0u8; size as usize].into_boxed_slice());
        let memory_ptr = shared_memory.as_ptr() as *mut u8;

        let mut v = Vec::with_capacity(cores);

        for i in 0..cores {
            let cpu = unsafe { UnicornCPU::new_with_shared_mem(i as u32, memory_ptr, size) };

            if let Some(cpu) = cpu {
                v.push(Box::new(cpu) as Box<dyn CpuBackend>);
            } else {
                panic!("Failed to create Core {}", i);
            }
        }

        Self {
            cores: v,
            shared_memory,
            address_space: AddressSpace::new(),
        }
    }

    pub fn run_all(&mut self) {
        // for now, just step all cores sequentially (round-robin)
        // in the future, this would be threaded
        for (_i, core) in self.cores.iter_mut().enumerate() {
            // just run one step for testing
            let _ = core.step(1);
        }
    }

    pub fn get_core(&self, id: usize) -> Option<&Box<dyn CpuBackend>> {
        self.cores.get(id)
    }

    pub fn get_core_mut(&mut self, id: usize) -> Option<&mut Box<dyn CpuBackend>> {
        self.cores.get_mut(id)
    }

    pub fn load_nso(&mut self, nso: &LoadedNso) -> Result<(), crate::cpu::error::CpuError> {
        load_nso_into(&mut self.shared_memory, &mut self.address_space, nso)
    }

    pub fn init_cpu_for_nso(&mut self, nso: &LoadedNso) -> Result<(), crate::cpu::error::CpuError> {
        use crate::cpu::error::CpuError;
        let ep = nso.entry_point;
        validate_entry_point(ep, &self.address_space)?;
        if let Some(core) = self.get_core_mut(0) {
            core.set_pc(ep);
        }
        Ok(())
    }
}

pub fn validate_entry_point(ep: u64, aspace: &AddressSpace) -> Result<(), crate::cpu::error::CpuError> {
    use crate::cpu::error::CpuError;
    let perms = crate::memory::translation::Permissions { read: true, write: false, execute: true };
    match aspace.translate(crate::memory::translation::GuestVirtAddr(ep), perms) {
        Ok(_) => Ok(()),
        Err(_) => Err(CpuError::MemoryFault { addr: ep }),
    }
}

pub fn load_nso_into(
    mem: &mut [u8],
    aspace: &mut AddressSpace,
    nso: &LoadedNso,
) -> Result<(), crate::cpu::error::CpuError> {
    use crate::cpu::error::CpuError;
    let mem_size = mem.len() as u64;

    let text_end = nso.text_addr.checked_add(nso.text.len() as u64).ok_or(CpuError::MemoryFault { addr: nso.text_addr })?;
    if text_end > mem_size { return Err(CpuError::MemoryFault { addr: nso.text_addr }); }
    let ro_end = nso.ro_addr.checked_add(nso.ro.len() as u64).ok_or(CpuError::MemoryFault { addr: nso.ro_addr })?;
    if ro_end > mem_size { return Err(CpuError::MemoryFault { addr: nso.ro_addr }); }
    let data_end = nso.data_addr.checked_add(nso.data.len() as u64).ok_or(CpuError::MemoryFault { addr: nso.data_addr })?;
    if data_end > mem_size { return Err(CpuError::MemoryFault { addr: nso.data_addr }); }
    let bss_addr = nso.data_addr + nso.data.len() as u64;
    let bss_end = bss_addr.checked_add(nso.bss_size as u64).ok_or(CpuError::MemoryFault { addr: bss_addr })?;
    if bss_end > mem_size { return Err(CpuError::MemoryFault { addr: bss_addr }); }

    let mut ranges = vec![
        (nso.text_addr, nso.text.len() as u64),
        (nso.ro_addr, nso.ro.len() as u64),
        (nso.data_addr, nso.data.len() as u64),
        (bss_addr, nso.bss_size as u64),
    ];
    ranges.sort();
    for i in 1..ranges.len() {
        if ranges[i].0 < ranges[i-1].0 + ranges[i-1].1 {
            return Err(CpuError::MemoryFault { addr: ranges[i].0 });
        }
    }

    mem[nso.text_addr as usize..text_end as usize].copy_from_slice(&nso.text);
    mem[nso.ro_addr as usize..ro_end as usize].copy_from_slice(&nso.ro);
    mem[nso.data_addr as usize..data_end as usize].copy_from_slice(&nso.data);
    for b in &mut mem[bss_addr as usize..bss_end as usize] {
        *b = 0;
    }

    let _ = aspace.map_region(MemoryRegion {
        base: GuestVirtAddr(nso.text_addr),
        size: nso.text.len() as u64,
        phys_base: GuestPhysAddr(nso.text_addr),
        perms: Permissions { read: true, write: false, execute: true },
    });
    let _ = aspace.map_region(MemoryRegion {
        base: GuestVirtAddr(nso.ro_addr),
        size: nso.ro.len() as u64,
        phys_base: GuestPhysAddr(nso.ro_addr),
        perms: Permissions { read: true, write: false, execute: false },
    });
    let _ = aspace.map_region(MemoryRegion {
        base: GuestVirtAddr(nso.data_addr),
        size: nso.data.len() as u64,
        phys_base: GuestPhysAddr(nso.data_addr),
        perms: Permissions { read: true, write: true, execute: false },
    });
    let _ = aspace.map_region(MemoryRegion {
        base: GuestVirtAddr(bss_addr),
        size: nso.bss_size as u64,
        phys_base: GuestPhysAddr(bss_addr),
        perms: Permissions { read: true, write: true, execute: false },
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::nso::LoadedNso;
    use crate::memory::translation::{AddressSpace, MemoryRegion, Permissions, GuestVirtAddr, GuestPhysAddr};

    fn make_nso(text_addr: u64, ro_addr: u64, data_addr: u64) -> LoadedNso {
        LoadedNso {
            entry_point: text_addr,
            text: vec![
                0x40, 0x05, 0x80, 0xd2,
                0x81, 0x0c, 0x80, 0xd2,
                0x02, 0x00, 0x01, 0x8b,
                0x23, 0x00, 0x80, 0xd2,
                0x44, 0x00, 0x02, 0x8b,
            ],
            ro: vec![4, 5],
            data: vec![6, 7, 8, 9],
            bss_size: 2,
            text_addr,
            ro_addr,
            data_addr,
        }
    }

    #[test]
    fn load_text_ro_data() {
        let mut mem = vec![0u8; 1024];
        let mut aspace = AddressSpace::new();
        let nso = make_nso(0x100, 0x200, 0x300);
        load_nso_into(&mut mem, &mut aspace, &nso).unwrap();
        assert_eq!(&mem[0x100..0x103], &[0x40, 0x05, 0x80]);
        assert_eq!(&mem[0x200..0x202], &[4, 5]);
        assert_eq!(&mem[0x300..0x304], &[6, 7, 8, 9]);
        assert_eq!(&mem[0x304..0x306], &[0, 0]);
    }

    #[test]
    fn out_of_bounds() {
        let mut mem = vec![0u8; 64];
        let mut aspace = AddressSpace::new();
        let nso = make_nso(0x1000, 0x200, 0x300);
        assert!(load_nso_into(&mut mem, &mut aspace, &nso).is_err());
    }

    #[test]
    fn overlap() {
        let mut mem = vec![0u8; 1024];
        let mut aspace = AddressSpace::new();
        let nso = make_nso(0x100, 0x100, 0x300);
        assert!(load_nso_into(&mut mem, &mut aspace, &nso).is_err());
    }

    #[test]
    fn cpu_init_nso() {
        let nso = make_nso(0x100, 0x200, 0x300);
        let mut mgr = CpuManager::with_memory_size_and_cores(0x10000, 1);
        mgr.load_nso(&nso).unwrap();
        mgr.init_cpu_for_nso(&nso).unwrap();
        let pc = mgr.get_core(0).unwrap().get_pc();
        assert_eq!(pc, nso.entry_point);
    }

    #[test]
    fn nso_end_to_end_exec() {
        let nso = make_nso(0x100, 0x200, 0x300);
        let mut mgr = CpuManager::with_memory_size_and_cores(0x10000, 1);
        mgr.load_nso(&nso).unwrap();
        mgr.init_cpu_for_nso(&nso).unwrap();
        let core = mgr.get_core_mut(0).unwrap();
        for i in 0..5 {
            let r = core.step(1);
            eprintln!("step {} -> {}", i, r);
        }
        eprintln!("X0={}", core.get_x(0).unwrap());
        assert_eq!(core.get_x(0).unwrap(), 42);
        assert_eq!(core.get_x(1).unwrap(), 100);
        assert_eq!(core.get_x(2).unwrap(), 142);
        assert_eq!(core.get_x(3).unwrap(), 1);
        assert_eq!(core.get_x(4).unwrap(), 143);
    }

    #[test]
    fn validate_entry_point_valid() {
        let mut aspace = AddressSpace::new();
        let _ = aspace.map_region(MemoryRegion {
            base: GuestVirtAddr(0x100),
            size: 0x10,
            phys_base: GuestPhysAddr(0x100),
            perms: Permissions { read: true, write: false, execute: true },
        });
        assert!(validate_entry_point(0x100, &aspace).is_ok());
    }

    #[test]
    fn validate_entry_point_invalid() {
        let aspace = AddressSpace::new();
        assert!(validate_entry_point(0x100, &aspace).is_err());
    }
}
