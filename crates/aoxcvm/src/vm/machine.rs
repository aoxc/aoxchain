#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineState {
    pub pc: u32,
    pub gas_used: u64,
    pub authority_used: u32,
    pub halted: bool,
}

impl Default for MachineState {
    fn default() -> Self {
        Self { pc: 0, gas_used: 0, authority_used: 0, halted: false }
    }
}
