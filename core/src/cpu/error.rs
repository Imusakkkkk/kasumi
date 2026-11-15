#[derive(Debug)]
pub enum CpuError {
    InvalidRegister(u32),
    MemoryFault { addr: u64 },
    ExecutionFailure(String),
    BackendSpecific(String),
    Halted,
}

impl std::fmt::Display for CpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CpuError {}