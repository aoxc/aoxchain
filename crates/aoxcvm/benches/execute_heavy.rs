use std::hint::black_box;
use std::time::Instant;

use aoxcvm::vm::machine::{Instruction, Machine, Program};

const WARMUP_ITERS: usize = 200;
const SAMPLE_ITERS: usize = 2_000;

fn heavy_program(rounds: usize) -> Program {
    let mut code = Vec::with_capacity((rounds * 6) + 2);
    code.push(Instruction::Push(1));

    for i in 0..rounds {
        code.push(Instruction::Push(
            (i as u64).wrapping_mul(3).wrapping_add(7),
        ));
        code.push(Instruction::Add);
        code.push(Instruction::Push(i as u64));
        code.push(Instruction::SStore);
        code.push(Instruction::Push(i as u64));
        code.push(Instruction::SLoad);
    }

    code.push(Instruction::Halt);
    Program { code }
}

fn run_case(rounds: usize) {
    let program = heavy_program(rounds);

    for _ in 0..WARMUP_ITERS {
        let result = Machine::new(program.clone(), 2_000_000, 4096)
            .execute()
            .expect("heavy program should execute successfully");
        black_box(result.receipt.gas_used);
    }

    let started = Instant::now();
    for _ in 0..SAMPLE_ITERS {
        let result = Machine::new(program.clone(), 2_000_000, 4096)
            .execute()
            .expect("heavy program should execute successfully");
        black_box(result.receipt.gas_used);
    }

    let elapsed = started.elapsed();
    let nanos_per_iter = elapsed.as_nanos() / (SAMPLE_ITERS as u128);
    println!("rounds={rounds}: {nanos_per_iter} ns/iter ({SAMPLE_ITERS} iterations)");
}

fn main() {
    println!("benchmark=execute_heavy_program");
    for rounds in [128usize, 512, 1024] {
        run_case(rounds);
    }
}
