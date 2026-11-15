#[derive(Clone, Debug, Default)]
pub struct CpuStateSnapshot {
    pub x: [u64; 31],
    pub sp: u64,
    pub pc: u64,
    pub pstate: u64,
}