#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Noop,
    Add,
    Sub,
    Store,
    Load,
    CallHost,
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
            Opcode::Add | Opcode::Sub => 2,
            Opcode::Store | Opcode::Load => 5,
            Opcode::CallHost => 15,
            Opcode::Halt => 0,
        }
    }
}
