#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionLimits {
    pub max_bytecode_size: u32,
    pub max_stack_depth: u16,
    pub max_syscalls: u16,
    pub max_objects_touched: u32,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_bytecode_size: 1_048_576,
            max_stack_depth: 256,
            max_syscalls: 512,
            max_objects_touched: 8_192,
        }
    }
}
