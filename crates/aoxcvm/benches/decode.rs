use std::hint::black_box;
use std::time::Instant;

use aoxcvm::auth::envelope::{AuthEnvelope, AuthEnvelopeLimits, SignatureEntry};
use aoxcvm::auth::scheme::{AuthProfile, SignatureAlgorithm};

const WARMUP_ITERS: usize = 2_000;
const SAMPLE_ITERS: usize = 25_000;

fn signer(algorithm: SignatureAlgorithm, key_id: &str, bytes: usize) -> SignatureEntry {
    SignatureEntry {
        algorithm,
        key_id: key_id.to_owned(),
        signature: vec![0xAB; bytes],
    }
}

fn run_case(name: &str, envelope: &AuthEnvelope, iterations: usize) {
    for _ in 0..WARMUP_ITERS {
        black_box(
            envelope
                .validate(AuthProfile::HybridMandatory, AuthEnvelopeLimits::default())
                .expect("benchmark envelope must remain valid"),
        );
    }

    let started = Instant::now();
    for _ in 0..iterations {
        black_box(
            envelope
                .validate(AuthProfile::HybridMandatory, AuthEnvelopeLimits::default())
                .expect("benchmark envelope must remain valid"),
        );
    }
    let elapsed = started.elapsed();
    let nanos_per_iter = elapsed.as_nanos() / (iterations as u128);

    println!("{name}: {nanos_per_iter} ns/iter ({iterations} iterations)");
}

fn main() {
    println!("benchmark=decode_auth_envelope_validation");
    for signer_count in [1usize, 2, 4, 8] {
        let mut signers = Vec::with_capacity(signer_count);
        signers.push(signer(SignatureAlgorithm::Ed25519, "classic-1", 64));
        for i in 1..signer_count {
            signers.push(signer(
                SignatureAlgorithm::MlDsa65,
                &format!("pq-{i}"),
                1536,
            ));
        }

        let envelope = AuthEnvelope {
            domain: "tx".to_string(),
            nonce: 42,
            signers,
        };

        run_case(&format!("signers={signer_count}"), &envelope, SAMPLE_ITERS);
    }
}
