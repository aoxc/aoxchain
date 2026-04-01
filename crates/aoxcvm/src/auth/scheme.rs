#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthScheme {
    Ed25519,
    MlDsa,
    SlhDsa,
    HybridEd25519MlDsa,
    Threshold,
}

impl AuthScheme {
    pub fn is_post_quantum_ready(self) -> bool {
        matches!(self, Self::MlDsa | Self::SlhDsa | Self::HybridEd25519MlDsa | Self::Threshold)
    }
}
