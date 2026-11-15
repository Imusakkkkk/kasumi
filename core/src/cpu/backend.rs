use super::error::CpuError;
use super::state::CpuStateSnapshot;

pub trait CpuBackend: Send + Sync {
    fn reset(&mut self);
    fn step(&mut self, budget: u64) -> Result<u64, CpuError>;
    fn get_pc(&self) -> u64;
    fn set_pc(&mut self, val: u64);
    fn get_sp(&self) -> u64;
    fn set_sp(&mut self, val: u64);
    fn get_x(&self, reg: u32) -> Result<u64, CpuError>;
    fn set_x(&mut self, reg: u32, val: u64) -> Result<(), CpuError>;
    fn read_mem(&self, addr: u64, buf: &mut [u8]) -> Result<(), CpuError>;
    fn write_mem(&mut self, addr: u64, buf: &[u8]) -> Result<(), CpuError>;
    fn halt(&mut self);
    fn snapshot(&self) -> CpuStateSnapshot;
    fn backend_name(&self) -> &'static str;

    fn write_u32(&mut self, vaddr: u64, value: u32) {
        let bytes = value.to_le_bytes();
        let _ = self.write_mem(vaddr, &bytes);
    }
    fn read_u32(&self, vaddr: u64) -> u32 {
        let mut bytes = [0u8; 4];
        if self.read_mem(vaddr, &mut bytes).is_ok() {
            u32::from_le_bytes(bytes)
        } else {
            0
        }
    }
}