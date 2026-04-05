use std::hint::black_box;
use std::time::Instant;

use aoxcvm::verifier::determinism::DeterminismVerifier;
use aoxcvm::vm::machine::{Instruction, Program};

const WARMUP_ITERS: usize = 100;
const SAMPLE_ITERS: usize = 1_000;

fn program_of_size(n: usize) -> Program {
    let mut code = Vec::with_capacity((n * 5) + 2);
    code.push(Instruction::Push(1));
    for i in 0..n {
        code.push(Instruction::Push((i as u64) + 2));
        code.push(Instruction::Mul);
        code.push(Instruction::Push(i as u64));
        code.push(Instruction::SStore);
        code.push(Instruction::Push(i as u64));
        code.push(Instruction::SLoad);
    }
    code.push(Instruction::Halt);
    Program { code }
}

fn run_case(verifier: DeterminismVerifier, steps: usize) {
    let program = program_of_size(steps);

    for _ in 0..WARMUP_ITERS {
        let result = verifier
            .verify(program.clone())
            .expect("program must pass deterministic verification");
        black_box(result.receipt.state_root);
    }

    let started = Instant::now();
    for _ in 0..SAMPLE_ITERS {
        let result = verifier
            .verify(program.clone())
            .expect("program must pass deterministic verification");
        black_box(result.receipt.state_root);
    }

    let elapsed = started.elapsed();
    let nanos_per_iter = elapsed.as_nanos() / (SAMPLE_ITERS as u128);
    println!("steps={steps}: {nanos_per_iter} ns/iter ({SAMPLE_ITERS} iterations)");
}

fn main() {
    println!("benchmark=verify_determinism");
    let verifier = DeterminismVerifier {
        gas_limit: 5_000_000,
        max_memory: 16 * 1024,
        max_stack_depth: 2_048,
    };

    for steps in [64usize, 256, 1024] {
        run_case(verifier, steps);
    }
}
