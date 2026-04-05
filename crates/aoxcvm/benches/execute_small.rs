use std::hint::black_box;
use std::time::Instant;

use aoxcvm::vm::machine::{Instruction, Machine, Program};

const WARMUP_ITERS: usize = 2_000;
const SAMPLE_ITERS: usize = 20_000;

fn small_program() -> Program {
    Program {
        code: vec![
            Instruction::Push(2),
            Instruction::Push(3),
            Instruction::Mul,
            Instruction::Push(11),
            Instruction::Add,
            Instruction::Halt,
        ],
    }
}

fn main() {
    println!("benchmark=execute_small_program");

    for _ in 0..WARMUP_ITERS {
        let result = Machine::new(small_program(), 10_000, 1024)
            .execute()
            .expect("small program should execute successfully");
        black_box(result.stack.last().copied().unwrap_or_default());
    }

    let started = Instant::now();
    for _ in 0..SAMPLE_ITERS {
        let result = Machine::new(small_program(), 10_000, 1024)
            .execute()
            .expect("small program should execute successfully");
        black_box(result.stack.last().copied().unwrap_or_default());
    }

    let elapsed = started.elapsed();
    let nanos_per_iter = elapsed.as_nanos() / (SAMPLE_ITERS as u128);
    println!("execute_small_program: {nanos_per_iter} ns/iter ({SAMPLE_ITERS} iterations)");
}
