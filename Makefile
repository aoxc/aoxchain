AOXC_DATA_ROOT ?= $(HOME)/.AOXCData
AOXC_HOME ?= $(AOXC_DATA_ROOT)/home/default
AOXC_BIN_DIR ?= $(AOXC_DATA_ROOT)/bin
AOXC_BIN_PATH ?= $(AOXC_BIN_DIR)/aoxc

.PHONY: help build build-release package-bin test check fmt clippy audit quality quality-quick quality-release ci run-local supervise-local audit-install produce-loop real-chain-prep real-chain-run real-chain-run-once real-chain-health real-chain-tail version manifest policy dev-bootstrap net-mainnet-start net-mainnet-once net-mainnet-status net-mainnet-stop net-testnet-start net-testnet-once net-testnet-status net-testnet-stop net-devnet-start net-devnet-once net-devnet-status net-devnet-stop ops-help ops-doctor ops-start-mainnet ops-start-testnet ops-start-devnet ops-stop-mainnet ops-stop-testnet ops-stop-devnet ops-status-mainnet ops-status-testnet ops-status-devnet ops-restart-mainnet ops-restart-testnet ops-restart-devnet ops-logs-mainnet ops-logs-testnet ops-logs-devnet
help:
	@printf "\nAOXChain developer targets\n\n"
	@printf "  make fmt              - format the workspace\n"
	@printf "  make check            - compile-check the workspace\n"
	@printf "  make test             - run workspace tests\n"
	@printf "  make clippy           - run clippy across workspace targets\n"
	@printf "  make quality-quick    - fmt/check/test quick gate\n"
	@printf "  make quality          - full quality gate\n"
	@printf "  make quality-release  - release-oriented quality gate\n\n"
	@printf "Build and release identity\n"
	@printf "  make quality-release  - release-oriented quality gate\n"
	@printf "  make build-release    - build the release AOXC CLI\n"
	@printf "  make package-bin      - install release binary into $$HOME/.AOXCData/bin (+ compat symlink ./bin/aoxc)\n"
	@printf "  make version          - show AOXC build/version metadata\n"
	@printf "  make manifest         - print build manifest and supply-chain policy\n"
	@printf "  make policy           - print node connection policy\n\n"
	@printf "Developer bootstrap\n"
	@printf "  make dev-bootstrap    - print suggested developer bootstrap flow\n"
	@printf "  make run-local        - run the local packaged node helper\n"
	@printf "  make supervise-local  - run the local supervisor helper\n\n"
	@printf "Local chain loop\n"
	@printf "  make real-chain-run-once - run one bounded daemon cycle\n"
	@printf "  make real-chain-run      - run the local real-chain daemon loop\n"
	@printf "  make real-chain-health   - probe local network health\n"
	@printf "  make real-chain-tail     - tail runtime and health logs\n\n"
	@printf "Environment daemons (production-oriented)\n"
	@printf "  make net-mainnet-start   - bootstrap/start mainnet daemon\n"
	@printf "  make net-mainnet-once    - run one mainnet produce+health cycle\n"
	@printf "  make net-mainnet-status  - show mainnet daemon status\n"
	@printf "  make net-mainnet-stop    - stop mainnet daemon\n"
	@printf "  make net-testnet-start   - bootstrap/start testnet daemon\n"
	@printf "  make net-testnet-once    - run one testnet produce+health cycle\n"
	@printf "  make net-testnet-status  - show testnet daemon status\n"
	@printf "  make net-testnet-stop    - stop testnet daemon\n"
	@printf "  make net-devnet-start    - bootstrap/start devnet daemon\n"
	@printf "  make net-devnet-once     - run one devnet produce+health cycle\n"
	@printf "  make net-devnet-status   - show devnet daemon status\n"
	@printf "  make net-devnet-stop     - stop devnet daemon\n\n"
	@printf "Easy operations CLI (7 to 77)\n"
	@printf "  make ops-help            - show beginner-friendly commands\n"
	@printf "  make ops-doctor          - run environment readiness checks\n"
	@printf "  make ops-start-mainnet   - start mainnet quickly\n"
	@printf "  make ops-start-testnet   - start testnet quickly\n"
	@printf "  make ops-start-devnet    - start devnet quickly\n"
	@printf "  make ops-status-mainnet  - mainnet status\n"
	@printf "  make ops-status-testnet  - testnet status\n"
	@printf "  make ops-status-devnet   - devnet status\n"
	@printf "  make ops-stop-mainnet    - stop mainnet\n"
	@printf "  make ops-stop-testnet    - stop testnet\n"
	@printf "  make ops-stop-devnet     - stop devnet\n"
	@printf "  make ops-logs-mainnet    - tail mainnet logs\n"
	@printf "  make ops-logs-testnet    - tail testnet logs\n"
	@printf "  make ops-logs-devnet     - tail devnet logs\n\n"

alpha:
	@printf "AOXC Alpha: Genesis V1\n"
	@printf "  make policy           - print node connection policy\n"
	@printf "  make dev-bootstrap    - print suggested developer bootstrap flow\n"
	@printf "  make real-chain-run   - run the local real-chain daemon loop\n"
	@printf "  make real-chain-tail  - tail local runtime logs\n\n"

build:
	cargo build --workspace

build-release:
	cargo build --release -p aoxcmd --bin aoxc

package-bin: build-release
	mkdir -p "$(AOXC_BIN_DIR)" bin
	cp target/release/aoxc "$(AOXC_BIN_PATH)"
	chmod +x "$(AOXC_BIN_PATH)"
	ln -sf "$(AOXC_BIN_PATH)" bin/aoxc
	@echo "Installed binary: $(AOXC_BIN_PATH)"
	@echo "Compatibility symlink: ./bin/aoxc -> $(AOXC_BIN_PATH)"

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
	@printf "export AOXC_HOME=$$HOME/.AOXCData/home/local-dev\n"
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
	@mkdir -p "$(AOXC_DATA_ROOT)/home/real" "$(AOXC_DATA_ROOT)/logs/real-chain"
	@echo "prepared AOXC_HOME=$(AOXC_DATA_ROOT)/home/real and logs under $(AOXC_DATA_ROOT)/logs/real-chain"

real-chain-run: real-chain-prep
	AOXC_HOME_DIR="$(AOXC_DATA_ROOT)/home/real" LOG_DIR="$(AOXC_DATA_ROOT)/logs/real-chain" ./scripts/real_chain_daemon.sh

real-chain-run-once: real-chain-prep
	MAX_CYCLES=1 AOXC_HOME_DIR="$(AOXC_DATA_ROOT)/home/real" LOG_DIR="$(AOXC_DATA_ROOT)/logs/real-chain" ./scripts/real_chain_daemon.sh

real-chain-health: package-bin
	"$(AOXC_BIN_PATH)" network-smoke --timeout-ms 3000 --bind-host 127.0.0.1 --port 0 --payload AOXC_REAL_HEALTH

real-chain-tail:
	tail -n 120 -f "$(AOXC_DATA_ROOT)/logs/real-chain/runtime.log" "$(AOXC_DATA_ROOT)/logs/real-chain/health.log"

net-mainnet-start: package-bin
	./scripts/network_env_daemon.sh start mainnet

net-mainnet-once: package-bin
	./scripts/network_env_daemon.sh once mainnet

net-mainnet-status:
	./scripts/network_env_daemon.sh status mainnet

net-mainnet-stop:
	./scripts/network_env_daemon.sh stop mainnet

net-testnet-start: package-bin
	./scripts/network_env_daemon.sh start testnet

net-testnet-once: package-bin
	./scripts/network_env_daemon.sh once testnet

net-testnet-status:
	./scripts/network_env_daemon.sh status testnet

net-testnet-stop:
	./scripts/network_env_daemon.sh stop testnet

net-devnet-start: package-bin
	./scripts/network_env_daemon.sh start devnet

net-devnet-once: package-bin
	./scripts/network_env_daemon.sh once devnet

net-devnet-status:
	./scripts/network_env_daemon.sh status devnet

net-devnet-stop:
	./scripts/network_env_daemon.sh stop devnet

ops-help:
	./scripts/aoxc_easy.sh help

ops-doctor:
	./scripts/aoxc_easy.sh doctor

ops-start-mainnet: package-bin
	./scripts/aoxc_easy.sh start mainnet

ops-start-testnet: package-bin
	./scripts/aoxc_easy.sh start testnet

ops-start-devnet: package-bin
	./scripts/aoxc_easy.sh start devnet

ops-stop-mainnet:
	./scripts/aoxc_easy.sh stop mainnet

ops-stop-testnet:
	./scripts/aoxc_easy.sh stop testnet

ops-stop-devnet:
	./scripts/aoxc_easy.sh stop devnet

ops-status-mainnet:
	./scripts/aoxc_easy.sh status mainnet

ops-status-testnet:
	./scripts/aoxc_easy.sh status testnet

ops-status-devnet:
	./scripts/aoxc_easy.sh status devnet

ops-restart-mainnet: package-bin
	./scripts/aoxc_easy.sh restart mainnet

ops-restart-testnet: package-bin
	./scripts/aoxc_easy.sh restart testnet

ops-restart-devnet: package-bin
	./scripts/aoxc_easy.sh restart devnet

ops-logs-mainnet:
	./scripts/aoxc_easy.sh logs mainnet

ops-logs-testnet:
	./scripts/aoxc_easy.sh logs testnet

ops-logs-devnet:
	./scripts/aoxc_easy.sh logs devnet
