#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Noop,
    MovImm,
    Add,
    Sub,
    Store,
    Load,
    CallHost,
    Checkpoint,
    Commit,
    Rollback,
    JumpIfZero,
    Halt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand_a: u64,
    pub operand_b: u64,
}

impl Instruction {
    pub const fn new(opcode: Opcode, operand_a: u64, operand_b: u64) -> Self {
        Self {
            opcode,
            operand_a,
            operand_b,
        }
    }

    pub const fn gas_cost(&self) -> u64 {
        match self.opcode {
            Opcode::Noop => 1,
            Opcode::MovImm => 2,
            Opcode::Add | Opcode::Sub => 3,
            Opcode::Store | Opcode::Load => 5,
            Opcode::CallHost => 15,
            Opcode::Checkpoint | Opcode::Commit | Opcode::Rollback => 4,
            Opcode::JumpIfZero => 2,
            Opcode::Halt => 0,
        }
    }
}
