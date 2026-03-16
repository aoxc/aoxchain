.PHONY: build build-release package-bin test check fmt clippy audit quality quality-quick quality-release ci run-local supervise-local audit-install produce-loop real-chain-prep real-chain-run real-chain-run-once real-chain-health real-chain-tail
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

audit-install:
	cargo install cargo-audit --locked

produce-loop: package-bin
	./scripts/continuous_producer.sh

real-chain-prep: package-bin
	@mkdir -p .aoxc-real logs/real-chain
	@echo "prepared AOXC_HOME=.aoxc-real and logs under logs/real-chain"

real-chain-run: real-chain-prep
	AOXC_HOME_DIR=./.aoxc-real LOG_DIR=./logs/real-chain ./scripts/real_chain_daemon.sh

real-chain-run-once: real-chain-prep
	MAX_CYCLES=1 AOXC_HOME_DIR=./.aoxc-real LOG_DIR=./logs/real-chain ./scripts/real_chain_daemon.sh

real-chain-health: package-bin
	./bin/aoxc network-smoke --timeout-ms 3000 --bind-host 127.0.0.1 --port 0 --payload AOXC_REAL_HEALTH

real-chain-tail:
	tail -n 120 -f logs/real-chain/runtime.log logs/real-chain/health.log
