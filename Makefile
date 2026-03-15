.PHONY: build build-release package-bin test check fmt clippy audit quality quality-quick quality-release ci run-local supervise-local

build:
	cargo build --workspace

build-release:
	cargo build --release -p aoxcmd --bin aoxc

package-bin: build-release
	mkdir -p bin
	cp target/release/aoxc bin/aoxc
	chmod +x bin/aoxc
	@echo "Packaged binary: ./bin/aoxc"

test:
	cargo test --workspace

check:
	cargo check --workspace

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets --all-features

audit:
	cargo audit

quality:
	./scripts/quality_gate.sh full

quality-quick:
	./scripts/quality_gate.sh quick

quality-release:
	./scripts/quality_gate.sh release

ci: quality

run-local: package-bin
	./scripts/run-local.sh


supervise-local: package-bin
	./scripts/node_supervisor.sh
