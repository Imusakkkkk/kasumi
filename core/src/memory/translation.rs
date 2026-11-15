use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GuestVirtAddr(pub u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GuestPhysAddr(pub u64);

#[derive(Clone, Copy, Debug)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TranslationError {
    Unmapped,
    PermissionDenied,
}

#[derive(Clone, Debug)]
pub struct MemoryRegion {
    pub base: GuestVirtAddr,
    pub size: u64,
    pub phys_base: GuestPhysAddr,
    pub perms: Permissions,
}

pub struct AddressSpace {
    regions: BTreeMap<GuestVirtAddr, MemoryRegion>,
}

impl AddressSpace {
    pub fn new() -> Self {
        Self { regions: BTreeMap::new() }
    }

    pub fn map_region(&mut self, region: MemoryRegion) -> Result<(), TranslationError> {
        // reject overlaps
        for r in self.regions.values() {
            let a_start = r.base.0;
            let a_end = a_start + r.size;
            let b_start = region.base.0;
            let b_end = b_start + region.size;
            if a_start < b_end && b_start < a_end {
                return Err(TranslationError::PermissionDenied);
            }
        }
        self.regions.insert(region.base, region);
        Ok(())
    }

    pub fn translate(&self, va: GuestVirtAddr, need: Permissions) -> Result<GuestPhysAddr, TranslationError> {
        for r in self.regions.values() {
            if va.0 >= r.base.0 && va.0 < r.base.0 + r.size {
                if (need.read && !r.perms.read) || (need.write && !r.perms.write) || (need.execute && !r.perms.execute) {
                    return Err(TranslationError::PermissionDenied);
                }
                let offset = va.0 - r.base.0;
                return Ok(GuestPhysAddr(r.phys_base.0 + offset));
            }
        }
        Err(TranslationError::Unmapped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_translation() {
        let mut aspace = AddressSpace::new();
        let r = MemoryRegion {
            base: GuestVirtAddr(0x1000),
            size: 0x1000,
            phys_base: GuestPhysAddr(0x2000),
            perms: Permissions { read: true, write: true, execute: true },
        };
        aspace.map_region(r).unwrap();
        let pa = aspace.translate(GuestVirtAddr(0x1000), Permissions { read: true, write: false, execute: false }).unwrap();
        assert_eq!(pa, GuestPhysAddr(0x2000));
    }

    #[test]
    fn unmapped() {
        let aspace = AddressSpace::new();
        assert_eq!(aspace.translate(GuestVirtAddr(0x1000), Permissions { read: true, write: false, execute: false }), Err(TranslationError::Unmapped));
    }

    #[test]
    fn perm_denied() {
        let mut aspace = AddressSpace::new();
        let r = MemoryRegion {
            base: GuestVirtAddr(0x1000),
            size: 0x1000,
            phys_base: GuestPhysAddr(0x2000),
            perms: Permissions { read: true, write: false, execute: true },
        };
        aspace.map_region(r).unwrap();
        assert_eq!(aspace.translate(GuestVirtAddr(0x1000), Permissions { read: false, write: true, execute: false }), Err(TranslationError::PermissionDenied));
    }

    #[test]
    fn boundary() {
        let mut aspace = AddressSpace::new();
        let r = MemoryRegion {
            base: GuestVirtAddr(0x1000),
            size: 0x1000,
            phys_base: GuestPhysAddr(0x2000),
            perms: Permissions { read: true, write: true, execute: true },
        };
        aspace.map_region(r).unwrap();
        assert!(aspace.translate(GuestVirtAddr(0x2000), Permissions { read: true, write: false, execute: false }).is_err());
    }

    #[test]
    fn overlap() {
        let mut aspace = AddressSpace::new();
        let r1 = MemoryRegion {
            base: GuestVirtAddr(0x1000),
            size: 0x1000,
            phys_base: GuestPhysAddr(0x2000),
            perms: Permissions { read: true, write: true, execute: true },
        };
        let r2 = MemoryRegion {
            base: GuestVirtAddr(0x1500),
            size: 0x1000,
            phys_base: GuestPhysAddr(0x3000),
            perms: Permissions { read: true, write: true, execute: true },
        };
        aspace.map_region(r1).unwrap();
        assert!(aspace.map_region(r2).is_err());
    }
}