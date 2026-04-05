use std::hint::black_box;
use std::time::Instant;

use aoxcvm::state::JournaledState;

const WARMUP_ITERS: usize = 2_000;
const SAMPLE_ITERS: usize = 30_000;

fn populated_state(entries: usize) -> JournaledState {
    let mut state = JournaledState::default();
    for i in 0..entries {
        state.put(
            format!("object:{i:08}").into_bytes(),
            format!("value:{i:08}").into_bytes(),
        );
    }
    state
}

fn run_case(entries: usize) {
    let state = populated_state(entries);
    let key = format!("object:{:08}", entries / 2).into_bytes();

    for _ in 0..WARMUP_ITERS {
        black_box(
            state
                .get(black_box(&key))
                .expect("key must be present in benchmark state"),
        );
    }

    let started = Instant::now();
    for _ in 0..SAMPLE_ITERS {
        black_box(
            state
                .get(black_box(&key))
                .expect("key must be present in benchmark state"),
        );
    }
    let elapsed = started.elapsed();
    let nanos_per_iter = elapsed.as_nanos() / (SAMPLE_ITERS as u128);

    println!("entries={entries}: {nanos_per_iter} ns/iter ({SAMPLE_ITERS} iterations)");
}

fn main() {
    println!("benchmark=object_lookup_state");
    for entries in [1_000usize, 10_000, 25_000] {
        run_case(entries);
    }
}
