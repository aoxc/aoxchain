use std::hint::black_box;
use std::time::Instant;

use aoxcvm::gas::meter::GasMeter;

const WARMUP_ITERS: usize = 1_000;
const SAMPLE_ITERS: usize = 10_000;
const CHARGES_PER_ITER: usize = 1_000;

fn run_case(charge: u64) {
    for _ in 0..WARMUP_ITERS {
        let mut meter = GasMeter::new(10_000_000);
        for _ in 0..CHARGES_PER_ITER {
            meter
                .charge(charge)
                .expect("charge should stay within benchmark budget");
        }
        black_box(meter.used());
    }

    let started = Instant::now();
    for _ in 0..SAMPLE_ITERS {
        let mut meter = GasMeter::new(10_000_000);
        for _ in 0..CHARGES_PER_ITER {
            meter
                .charge(charge)
                .expect("charge should stay within benchmark budget");
        }
        black_box(meter.used());
    }

    let elapsed = started.elapsed();
    let nanos_per_iter = elapsed.as_nanos() / (SAMPLE_ITERS as u128);
    println!("charge={charge}: {nanos_per_iter} ns/iter ({SAMPLE_ITERS} iterations)");
}

fn main() {
    println!("benchmark=gas_accounting");
    for charge in [1u64, 3, 25, 200, 5_000] {
        run_case(charge);
    }
}
