#[derive(Debug, Clone)]
pub struct UnityRuntime {
    pub quorum_numerator: u64,
    pub quorum_denominator: u64,
}

impl UnityRuntime {
    #[must_use]
    pub fn new(quorum_numerator: u64, quorum_denominator: u64) -> Self {
        Self {
            quorum_numerator,
            quorum_denominator,
        }
    }
}
