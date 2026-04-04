use crate::*;

#[must_use]
pub fn default_lane_registry() -> LaneRegistry {
    LaneRegistry::new(vec![
        LaneRegistryPolicy::new(
            "native-mainnet",
            1,
            1,
            "gov://bootstrap/native/v1",
            LanePolicy::new("native", 21_000, 8, 64 * 1024, 5_000_000, 64),
        ),
        LaneRegistryPolicy::new(
            "evm-mainnet",
            1,
            1,
            "gov://bootstrap/evm/v1",
            LanePolicy::new("evm", 21_000, 16, 128 * 1024, 15_000_000, 64),
        ),
        LaneRegistryPolicy::new(
            "wasm-mainnet",
            1,
            1,
            "gov://bootstrap/wasm/v1",
            LanePolicy::new("wasm", 35_000, 24, 256 * 1024, 20_000_000, 32),
        ),
        LaneRegistryPolicy::new(
            "sui-move-mainnet",
            1,
            1,
            "gov://bootstrap/sui_move/v1",
            LanePolicy::new("sui_move", 40_000, 20, 128 * 1024, 12_000_000, 32),
        ),
    ])
}

#[must_use]
pub fn default_lanes() -> Vec<Box<dyn ExecutionLane + Send + Sync>> {
    vec![
        Box::new(DeterministicLane::new("native")),
        Box::new(DeterministicLane::new("evm")),
        Box::new(DeterministicLane::new("wasm")),
        Box::new(DeterministicLane::new("sui_move")),
    ]
}

