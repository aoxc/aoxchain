.PHONY: help build build-release package-bin test check fmt clippy audit quality quality-quick quality-release ci run-local supervise-local audit-install produce-loop real-chain-prep real-chain-run real-chain-run-once real-chain-health real-chain-tail version manifest policy dev-bootstrap
help:
	@printf "\nAOXChain developer targets\n\n"
	@printf "  make fmt              - format the workspace\n"
	@printf "  make check            - compile-check the workspace\n"
	@printf "  make test             - run workspace tests\n"
	@printf "  make clippy           - run clippy across workspace targets\n"
	@printf "  make quality-quick    - fmt/check/test quick gate\n"
	@printf "  make quality          - full quality gate\n"
	@printf "  make quality-release  - release-oriented quality gate\n"
	@printf "  make build-release    - build the release AOXC CLI\n"
	@printf "  make package-bin      - copy release binary into ./bin\n"
	@printf "  make version          - show AOXC build/version metadata\n"
	@printf "  make manifest         - print build manifest and supply-chain policy\n"
	@printf "  make policy           - print node connection policy\n"
	@printf "  make dev-bootstrap    - print suggested developer bootstrap flow\n"
	@printf "  make real-chain-run   - run the local real-chain daemon loop\n"
	@printf "  make real-chain-tail  - tail local runtime logs\n\n"

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

version:
	cargo run -p aoxcmd -- version

manifest:
	cargo run -p aoxcmd -- build-manifest

policy:
	cargo run -p aoxcmd -- node-connection-policy

dev-bootstrap:
	@printf "export AOXC_HOME=$$PWD/.aoxc-local\n"
	@printf "make fmt && make check && make test\n"
	@printf "cargo run -p aoxcmd -- key-bootstrap --home \"$$AOXC_HOME\" --profile testnet --name validator-01 --password 'TEST#Secure2026!'\n"
	@printf "cargo run -p aoxcmd -- genesis-init --home \"$$AOXC_HOME\" --chain-num 1001 --block-time 6 --treasury 1000000000000\n"
	@printf "cargo run -p aoxcmd -- node-bootstrap --home \"$$AOXC_HOME\"\n"
	@printf "cargo run -p aoxcmd -- produce-once --home \"$$AOXC_HOME\" --tx 'hello-aoxc'\n"

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
