/// Minimal validator script descriptor.
///
/// At this stage the lane does not execute full Plutus/UPLC semantics.
/// The script structure exists so the host model can mature without
/// inventing throwaway abstractions later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoScript {
    pub script_hash: [u8; 32],
    pub script_bytes: Vec<u8>,
}
