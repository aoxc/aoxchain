# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.

SHELL := /bin/bash
.DEFAULT_GOAL := help

# --------------------------------------------------------------------
# Canonical AOXC path contract
# --------------------------------------------------------------------
# Canonical AOXC data root:
#   $(HOME)/.AOXCData
#
# Effective default AOXC home:
#   $(AOXC_DATA_ROOT)/home/default
#
# Disposable test homes:
#   $(AOXC_DATA_ROOT)/.test/<label>
#
# Design objective:
# - Preserve a single AOXC-owned namespace beneath the user home.
# - Keep runtime homes explicit and profileable under home/<name>.
# - Align CLI, scripts, and packaging around one stable storage contract.
AOXC_DATA_ROOT ?= $(HOME)/.AOXCData
AOXC_HOME ?= $(AOXC_DATA_ROOT)/home/default
AOXC_BIN_DIR ?= $(AOXC_DATA_ROOT)/bin
AOXC_BIN_PATH ?= $(AOXC_BIN_DIR)/aoxc
AOXC_RELEASES_DIR ?= $(AOXC_DATA_ROOT)/releases
AOXC_NETWORK_BIN_ROOT ?= $(AOXC_DATA_ROOT)/binary

AOXC_HOME_LOCAL ?= $(AOXC_DATA_ROOT)/home/local-dev
AOXC_HOME_REAL ?= $(AOXC_DATA_ROOT)/home/real
AOXC_HOME_DESKTOP_TESTNET ?= $(AOXC_DATA_ROOT)/desktop/testnet/home

AOXC_LOG_ROOT ?= $(AOXC_DATA_ROOT)/logs
AOXC_REAL_LOG_DIR ?= $(AOXC_LOG_ROOT)/real-chain
AOXC_DESKTOP_ROOT ?= $(AOXC_DATA_ROOT)/desktop/testnet
AOXC_DESKTOP_BIN_DIR ?= $(AOXC_DESKTOP_ROOT)/bin
AOXC_DESKTOP_LOG_DIR ?= $(AOXC_DESKTOP_ROOT)/logs

CARGO ?= cargo
RUSTFMT ?= rustfmt
CLIPPY_FLAGS ?= --workspace --all-targets --all-features
TEST_FLAGS ?= --workspace
CHECK_FLAGS ?= --workspace

RELEASE_VERSION ?= $(shell $(CARGO) pkgid -p aoxcmd 2>/dev/null | sed -E 's|.*#||; s|.*@||')
RELEASE_TAG ?= v$(RELEASE_VERSION)
RELEASE_BUNDLE_NAME ?= aoxc-$(RELEASE_TAG)
RELEASE_BUNDLE_DIR ?= $(AOXC_RELEASES_DIR)/$(RELEASE_BUNDLE_NAME)
RELEASE_BUNDLE_BIN_DIR ?= $(RELEASE_BUNDLE_DIR)/bin
RELEASE_BUNDLE_MANIFEST ?= $(RELEASE_BUNDLE_DIR)/BUILD-MANIFEST.txt
RELEASE_BUNDLE_CHECKSUMS ?= $(RELEASE_BUNDLE_DIR)/SHA256SUMS
RELEASE_ARCHIVE_BASENAME ?= $(RELEASE_BUNDLE_NAME)-linux-amd64
RELEASE_ARCHIVE_PATH ?= $(AOXC_RELEASES_DIR)/$(RELEASE_ARCHIVE_BASENAME).tar.gz
RELEASE_BINARIES ?= aoxc aoxchub aoxckit

# --------------------------------------------------------------------
# Shared shell helpers
# --------------------------------------------------------------------
define print_banner
	@printf "\n==> %s\n" "$(1)"
endef

define require_file
	@test -f "$(1)" || { echo "Missing required file: $(1)"; exit 1; }
endef

define ensure_dir
	@mkdir -p "$(1)"
endef

# --------------------------------------------------------------------
# Phony targets
# --------------------------------------------------------------------
.PHONY: \
	help paths env-check bootstrap-paths clean-home clean-logs \
	bootstrap-desktop-paths build build-release build-release-all package-bin package-all-bin package-versioned-bin package-versioned-archive package-network-versioned-bin package-desktop-testnet install-bin \
	test test-lib test-workspace check fmt clippy audit \
	quality quality-quick quality-release ci \
	version manifest policy \
	dev-bootstrap run-local supervise-local audit-install produce-loop \
	real-chain-prep real-chain-run real-chain-run-once real-chain-health real-chain-tail \
	net-mainnet-start net-mainnet-once net-mainnet-status net-mainnet-stop \
	net-testnet-start net-testnet-once net-testnet-status net-testnet-stop \
	net-devnet-start net-devnet-once net-devnet-status net-devnet-stop \
	net-dual-start net-dual-once net-dual-status net-dual-stop net-dual-restart \
	ops-help ops-doctor \
	ops-start-mainnet ops-start-testnet ops-start-devnet ops-start-dual ops-auto-start ops-auto-once \
	ops-once-mainnet ops-once-testnet ops-once-devnet \
	ops-stop-mainnet ops-stop-testnet ops-stop-devnet ops-stop-dual \
	ops-status-mainnet ops-status-testnet ops-status-devnet ops-status-dual \
	ops-restart-mainnet ops-restart-testnet ops-restart-devnet ops-restart-dual \
	ops-logs-mainnet ops-logs-testnet ops-logs-devnet ops-dashboard ops-flow-mainnet ops-flow-testnet ops-flow-devnet \
	alpha

# --------------------------------------------------------------------
# Help / diagnostics
# --------------------------------------------------------------------
help:
	@printf "\nAOXChain developer and operator targets\n\n"
	@printf "Path contract\n"
	@printf "  AOXC_DATA_ROOT : %s\n" "$(AOXC_DATA_ROOT)"
	@printf "  AOXC_HOME      : %s\n" "$(AOXC_HOME)"
	@printf "  AOXC_BIN_PATH  : %s\n\n" "$(AOXC_BIN_PATH)"
	@printf "  Desktop testnet home : %s\n" "$(AOXC_HOME_DESKTOP_TESTNET)"
	@printf "  Desktop bin root     : %s\n\n" "$(AOXC_DESKTOP_BIN_DIR)"

	@printf "Workspace quality\n"
	@printf "  make fmt               - format the workspace\n"
	@printf "  make check             - compile-check the workspace\n"
	@printf "  make test              - run workspace tests\n"
	@printf "  make clippy            - run clippy across workspace targets\n"
	@printf "  make audit             - run cargo-audit\n"
	@printf "  make quality-quick     - quick gate\n"
	@printf "  make quality           - full gate\n"
	@printf "  make quality-release   - release-oriented gate\n\n"

	@printf "Build and packaging\n"
	@printf "  make build             - build the workspace\n"
	@printf "  make build-release     - build the release AOXC CLI\n"
	@printf "  make package-bin       - install release binary into %s\n" "$(AOXC_BIN_DIR)"
	@printf "  make build-release-all - build all workspace release binaries\n"
	@printf "  make release-binary-list - print detected workspace binary names\n"
	@printf "  make package-all-bin   - install all release binaries into %s\n" "$(AOXC_BIN_DIR)"
	@printf "  make package-versioned-bin - install all binaries into versioned bundle under %s\n" "$(AOXC_RELEASES_DIR)"
	@printf "  make package-versioned-archive - create tar.gz archive for the versioned bundle\n"
	@printf "  make package-network-versioned-bin - install per-network versioned AOXC CLI copies under %s\n" "$(AOXC_NETWORK_BIN_ROOT)"
	@printf "  make package-desktop-testnet - install all binaries under desktop/testnet root\n"
	@printf "  make version           - show AOXC build/version metadata\n"
	@printf "  make manifest          - print build manifest\n"
	@printf "  make policy            - print node connection policy\n\n"

	@printf "Environment and paths\n"
	@printf "  make paths             - print resolved AOXC paths\n"
	@printf "  make env-check         - validate required local tools and scripts\n"
	@printf "  make bootstrap-paths   - create canonical AOXC directories\n"
	@printf "  make bootstrap-desktop-paths - create desktop/testnet directories\n"
	@printf "  make clean-home        - remove AOXC_HOME only\n"
	@printf "  make clean-logs        - remove AOXC log directories only\n\n"

	@printf "Developer bootstrap\n"
	@printf "  make dev-bootstrap     - print suggested bootstrap flow\n"
	@printf "  make run-local         - run local packaged node helper\n"
	@printf "  make supervise-local   - run local supervisor helper\n"
	@printf "  make produce-loop      - run continuous producer helper\n\n"

	@printf "Local real-chain loop\n"
	@printf "  make real-chain-prep      - prepare real-chain home and logs\n"
	@printf "  make real-chain-run       - run the local real-chain daemon loop\n"
	@printf "  make real-chain-run-once  - run one bounded daemon cycle\n"
	@printf "  make real-chain-health    - probe local health\n"
	@printf "  make real-chain-tail      - tail runtime and health logs\n\n"

	@printf "Network daemons\n"
	@printf "  make net-mainnet-start    - bootstrap/start mainnet daemon\n"
	@printf "  make net-mainnet-once     - run one mainnet cycle\n"
	@printf "  make net-mainnet-status   - show mainnet status\n"
	@printf "  make net-mainnet-stop     - stop mainnet daemon\n"
	@printf "  make net-testnet-start    - bootstrap/start testnet daemon\n"
	@printf "  make net-testnet-once     - run one testnet cycle\n"
	@printf "  make net-testnet-status   - show testnet status\n"
	@printf "  make net-testnet-stop     - stop testnet daemon\n"
	@printf "  make net-devnet-start     - bootstrap/start devnet daemon\n"
	@printf "  make net-devnet-once      - run one devnet cycle\n"
	@printf "  make net-devnet-status    - show devnet status\n"
	@printf "  make net-devnet-stop      - stop devnet daemon\n"
	@printf "  make net-dual-start       - start testnet and mainnet together\n"
	@printf "  make net-dual-once        - run one cycle on dual stack\n"
	@printf "  make net-dual-status      - show dual stack status\n"
	@printf "  make net-dual-stop        - stop dual stack\n"
	@printf "  make net-dual-restart     - restart dual stack\n\n"

	@printf "Easy operations\n"
	@printf "  make ops-help             - show beginner-friendly commands\n"
	@printf "  make ops-doctor           - run environment readiness checks\n"
	@printf "  make ops-start-mainnet    - start mainnet quickly\n"
	@printf "  make ops-start-testnet    - start testnet quickly\n"
	@printf "  make ops-start-devnet     - start devnet quickly\n"
	@printf "  make ops-start-dual       - start testnet+mainnet together\n"
	@printf "  make ops-auto-start       - start AOXC_ENV (default devnet) automatically\n"
	@printf "  make ops-auto-once        - run one cycle on AOXC_ENV (default devnet)\n"
	@printf "  make ops-once-mainnet     - run one bounded cycle on mainnet\n"
	@printf "  make ops-once-testnet     - run one bounded cycle on testnet\n"
	@printf "  make ops-once-devnet      - run one bounded cycle on devnet\n"
	@printf "  make ops-stop-mainnet     - stop mainnet\n"
	@printf "  make ops-stop-testnet     - stop testnet\n"
	@printf "  make ops-stop-devnet      - stop devnet\n"
	@printf "  make ops-stop-dual        - stop testnet+mainnet together\n"
	@printf "  make ops-status-mainnet   - mainnet status\n"
	@printf "  make ops-status-testnet   - testnet status\n"
	@printf "  make ops-status-devnet    - devnet status\n"
	@printf "  make ops-status-dual      - dual status\n"
	@printf "  make ops-restart-mainnet  - restart mainnet\n"
	@printf "  make ops-restart-testnet  - restart testnet\n"
	@printf "  make ops-restart-devnet   - restart devnet\n"
	@printf "  make ops-restart-dual     - restart dual stack\n"
	@printf "  make ops-logs-mainnet     - tail mainnet logs\n"
	@printf "  make ops-logs-testnet     - tail testnet logs\n"
	@printf "  make ops-logs-devnet      - tail devnet logs\n\n"
	@printf "  make ops-dashboard        - show full multi-env dashboard\n"
	@printf "  make ops-flow-mainnet     - full auto operational flow (mainnet)\n"
	@printf "  make ops-flow-testnet     - full auto operational flow (testnet)\n"
	@printf "  make ops-flow-devnet      - full auto operational flow (devnet)\n\n"

paths:
	@printf "AOXC_DATA_ROOT=%s\n" "$(AOXC_DATA_ROOT)"
	@printf "AOXC_HOME=%s\n" "$(AOXC_HOME)"
	@printf "AOXC_HOME_LOCAL=%s\n" "$(AOXC_HOME_LOCAL)"
	@printf "AOXC_HOME_REAL=%s\n" "$(AOXC_HOME_REAL)"
	@printf "AOXC_HOME_DESKTOP_TESTNET=%s\n" "$(AOXC_HOME_DESKTOP_TESTNET)"
	@printf "AOXC_BIN_DIR=%s\n" "$(AOXC_BIN_DIR)"
	@printf "AOXC_BIN_PATH=%s\n" "$(AOXC_BIN_PATH)"
	@printf "AOXC_RELEASES_DIR=%s\n" "$(AOXC_RELEASES_DIR)"
	@printf "AOXC_NETWORK_BIN_ROOT=%s\n" "$(AOXC_NETWORK_BIN_ROOT)"
	@printf "AOXC_LOG_ROOT=%s\n" "$(AOXC_LOG_ROOT)"
	@printf "AOXC_REAL_LOG_DIR=%s\n" "$(AOXC_REAL_LOG_DIR)"
	@printf "AOXC_DESKTOP_ROOT=%s\n" "$(AOXC_DESKTOP_ROOT)"
	@printf "AOXC_DESKTOP_BIN_DIR=%s\n" "$(AOXC_DESKTOP_BIN_DIR)"
	@printf "AOXC_DESKTOP_LOG_DIR=%s\n" "$(AOXC_DESKTOP_LOG_DIR)"
	@printf "RELEASE_TAG=%s\n" "$(RELEASE_TAG)"
	@printf "RELEASE_BUNDLE_DIR=%s\n" "$(RELEASE_BUNDLE_DIR)"
	@printf "RELEASE_ARCHIVE_PATH=%s\n" "$(RELEASE_ARCHIVE_PATH)"

env-check:
	$(call print_banner,Validating local build environment)
	@command -v $(CARGO) >/dev/null 2>&1 || { echo "cargo not found"; exit 1; }
	@command -v git >/dev/null 2>&1 || { echo "git not found"; exit 1; }
	@command -v bash >/dev/null 2>&1 || { echo "bash not found"; exit 1; }
	$(call require_file,./scripts/quality_gate.sh)
	$(call require_file,./scripts/run-local.sh)
	$(call require_file,./scripts/node_supervisor.sh)
	$(call require_file,./scripts/continuous_producer.sh)
	$(call require_file,./scripts/real_chain_daemon.sh)
	$(call require_file,./scripts/network_env_daemon.sh)
	$(call require_file,./scripts/network_stack.sh)
	$(call require_file,./scripts/aoxc_easy.sh)
	@echo "Environment check passed."

bootstrap-paths:
	$(call print_banner,Creating canonical AOXC directories)
	$(call ensure_dir,$(AOXC_DATA_ROOT))
	$(call ensure_dir,$(AOXC_DATA_ROOT)/home/default)
	$(call ensure_dir,$(AOXC_DATA_ROOT)/home/local-dev)
	$(call ensure_dir,$(AOXC_DATA_ROOT)/home/real)
	$(call ensure_dir,$(AOXC_DATA_ROOT)/bin)
	$(call ensure_dir,$(AOXC_DATA_ROOT)/logs)
	$(call ensure_dir,$(AOXC_DATA_ROOT)/logs/real-chain)
	$(call ensure_dir,$(AOXC_DATA_ROOT)/.test)
	@echo "AOXC path bootstrap complete."

bootstrap-desktop-paths:
	$(call print_banner,Creating AOXC desktop testnet directories)
	$(call ensure_dir,$(AOXC_DESKTOP_ROOT))
	$(call ensure_dir,$(AOXC_HOME_DESKTOP_TESTNET))
	$(call ensure_dir,$(AOXC_DESKTOP_BIN_DIR))
	$(call ensure_dir,$(AOXC_DESKTOP_LOG_DIR))
	@echo "AOXC desktop testnet path bootstrap complete."

clean-home:
	$(call print_banner,Removing effective AOXC home)
	@rm -rf "$(AOXC_HOME)"
	@echo "Removed: $(AOXC_HOME)"

clean-logs:
	$(call print_banner,Removing AOXC logs)
	@rm -rf "$(AOXC_LOG_ROOT)"
	@echo "Removed: $(AOXC_LOG_ROOT)"

# --------------------------------------------------------------------
# Build / quality
# --------------------------------------------------------------------
build:
	$(call print_banner,Building workspace)
	$(CARGO) build --workspace

build-release:
	$(call print_banner,Building release AOXC CLI)
	$(CARGO) build --release -p aoxcmd --bin aoxc

build-release-all:
	$(call print_banner,Building all release AOXC binaries)
	$(CARGO) build --release --workspace --bins

release-binary-list:
	$(call print_banner,Detected workspace binary names)
	@if [ -z "$(RELEASE_BINARIES)" ]; then \
		echo "No binaries detected from cargo metadata."; \
		exit 1; \
	fi
	@for binary in $(RELEASE_BINARIES); do echo "$$binary"; done

package-bin: build-release bootstrap-paths
	$(call print_banner,Packaging release binary)
	@mkdir -p "$(AOXC_BIN_DIR)" bin
	@cp target/release/aoxc "$(AOXC_BIN_PATH)"
	@chmod +x "$(AOXC_BIN_PATH)"
	@ln -sf "$(AOXC_BIN_PATH)" bin/aoxc
	@echo "Installed binary: $(AOXC_BIN_PATH)"
	@echo "Compatibility symlink: ./bin/aoxc -> $(AOXC_BIN_PATH)"

package-all-bin: build-release-all bootstrap-paths
	$(call print_banner,Packaging all release binaries)
	@set -euo pipefail; \
	mkdir -p "$(AOXC_BIN_DIR)" bin; \
	if [ -z "$(RELEASE_BINARIES)" ]; then echo "No binaries detected from cargo metadata."; exit 1; fi; \
	for binary in $(RELEASE_BINARIES); do \
		cp "target/release/$$binary" "$(AOXC_BIN_DIR)/$$binary"; \
		chmod +x "$(AOXC_BIN_DIR)/$$binary"; \
		ln -sf "$(AOXC_BIN_DIR)/$$binary" "bin/$$binary"; \
	done
	@echo "Installed binaries under: $(AOXC_BIN_DIR)"
	@echo "Compatibility symlinks created under ./bin"

package-versioned-bin: build-release-all
	$(call print_banner,Packaging versioned release bundle)
	@set -euo pipefail; \
	mkdir -p "$(RELEASE_BUNDLE_BIN_DIR)"; \
	for binary in $(RELEASE_BINARIES); do \
		cp "target/release/$$binary" "$(RELEASE_BUNDLE_BIN_DIR)/$$binary"; \
		chmod +x "$(RELEASE_BUNDLE_BIN_DIR)/$$binary"; \
	done; \
	printf "AOXC Release Bundle\n" > "$(RELEASE_BUNDLE_MANIFEST)"; \
	printf "release_tag=%s\n" "$(RELEASE_TAG)" >> "$(RELEASE_BUNDLE_MANIFEST)"; \
	printf "bundle_name=%s\n" "$(RELEASE_BUNDLE_NAME)" >> "$(RELEASE_BUNDLE_MANIFEST)"; \
	printf "bundle_dir=%s\n" "$(RELEASE_BUNDLE_DIR)" >> "$(RELEASE_BUNDLE_MANIFEST)"; \
	printf "generated_at_utc=%s\n" "$$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$(RELEASE_BUNDLE_MANIFEST)"; \
	printf "binaries=%s\n" "$(RELEASE_BINARIES)" >> "$(RELEASE_BUNDLE_MANIFEST)"; \
	( \
		cd "$(RELEASE_BUNDLE_DIR)" && \
		sha256sum bin/* > "$(notdir $(RELEASE_BUNDLE_CHECKSUMS))" \
	); \
	echo "Versioned bundle directory: $(RELEASE_BUNDLE_DIR)"; \
	echo "Bundle manifest: $(RELEASE_BUNDLE_MANIFEST)"; \
	echo "Checksums: $(RELEASE_BUNDLE_CHECKSUMS)"

package-versioned-archive: package-versioned-bin
	$(call print_banner,Creating versioned release archive)
	@set -euo pipefail; \
	mkdir -p "$(AOXC_RELEASES_DIR)"; \
	tar -C "$(AOXC_RELEASES_DIR)" -czf "$(RELEASE_ARCHIVE_PATH)" "$(RELEASE_BUNDLE_NAME)"; \
	echo "Versioned release archive: $(RELEASE_ARCHIVE_PATH)"

package-network-versioned-bin: build-release bootstrap-paths
	$(call print_banner,Packaging network-scoped AOXC release binaries)
	@set -euo pipefail; \
	mkdir -p "$(AOXC_NETWORK_BIN_ROOT)/mainnet" "$(AOXC_NETWORK_BIN_ROOT)/testnet" "$(AOXC_NETWORK_BIN_ROOT)/devnet"; \
	for env in mainnet testnet devnet; do \
		target_path="$(AOXC_NETWORK_BIN_ROOT)/$$env/aoxc-$(RELEASE_TAG)"; \
		cp target/release/aoxc "$$target_path"; \
		chmod +x "$$target_path"; \
		ln -sfn "$$target_path" "$(AOXC_NETWORK_BIN_ROOT)/$$env/aoxc-current"; \
		echo "[$$env] installed $$target_path"; \
	done

package-desktop-testnet: build-release-all bootstrap-desktop-paths
	$(call print_banner,Packaging desktop testnet binaries and profile layout)
	@mkdir -p "$(AOXC_DESKTOP_BIN_DIR)"
	@cp target/release/aoxc "$(AOXC_DESKTOP_BIN_DIR)/aoxc"
	@cp target/release/aoxchub "$(AOXC_DESKTOP_BIN_DIR)/aoxchub"
	@cp target/release/aoxckit "$(AOXC_DESKTOP_BIN_DIR)/aoxckit"
	@chmod +x "$(AOXC_DESKTOP_BIN_DIR)/aoxc" "$(AOXC_DESKTOP_BIN_DIR)/aoxchub" "$(AOXC_DESKTOP_BIN_DIR)/aoxckit"
	@echo "Desktop binaries installed under: $(AOXC_DESKTOP_BIN_DIR)"
	@echo "Use AOXC_HOME=$(AOXC_HOME_DESKTOP_TESTNET) for desktop-testnet runtime isolation."

install-bin: package-bin

test:
	$(call print_banner,Running workspace tests)
	$(CARGO) test $(TEST_FLAGS)

test-lib:
	$(call print_banner,Running library tests)
	$(CARGO) test --workspace --lib

test-workspace: test

check:
	$(call print_banner,Checking workspace)
	$(CARGO) check $(CHECK_FLAGS)

fmt:
	$(call print_banner,Formatting workspace)
	$(CARGO) fmt --all

clippy:
	$(call print_banner,Running clippy)
	$(CARGO) clippy $(CLIPPY_FLAGS)

audit:
	$(call print_banner,Running cargo-audit)
	$(CARGO) audit

quality:
	$(call print_banner,Running full quality gate)
	./scripts/quality_gate.sh full

quality-quick:
	$(call print_banner,Running quick quality gate)
	./scripts/quality_gate.sh quick

quality-release:
	$(call print_banner,Running release quality gate)
	./scripts/quality_gate.sh release

ci: fmt check test clippy audit

# --------------------------------------------------------------------
# Informational CLI surfaces
# --------------------------------------------------------------------
version:
	$(CARGO) run -p aoxcmd -- version

manifest:
	$(CARGO) run -p aoxcmd -- build-manifest

policy:
	$(CARGO) run -p aoxcmd -- node-connection-policy

# --------------------------------------------------------------------
# Developer bootstrap and local helpers
# --------------------------------------------------------------------
dev-bootstrap:
	@printf "export AOXC_HOME=%s\n" "$(AOXC_HOME_LOCAL)"
	@printf "make bootstrap-paths fmt check test\n"
	@printf "cargo run -p aoxcmd -- key-bootstrap --home \"%s\" --profile testnet --name validator-01 --password 'TEST#Secure2026!'\n" "$(AOXC_HOME_LOCAL)"
	@printf "cargo run -p aoxcmd -- genesis-init --home \"%s\" --chain-num 1001 --block-time 6 --treasury 1000000000000\n" "$(AOXC_HOME_LOCAL)"
	@printf "cargo run -p aoxcmd -- node-bootstrap --home \"%s\"\n" "$(AOXC_HOME_LOCAL)"
	@printf "cargo run -p aoxcmd -- produce-once --home \"%s\" --tx 'hello-aoxc'\n" "$(AOXC_HOME_LOCAL)"

run-local: package-bin
	./scripts/run-local.sh

supervise-local: package-bin
	./scripts/node_supervisor.sh

audit-install:
	$(CARGO) install cargo-audit --locked

produce-loop: package-bin
	./scripts/continuous_producer.sh

# --------------------------------------------------------------------
# Local real-chain workflow
# --------------------------------------------------------------------
real-chain-prep: bootstrap-paths package-bin
	$(call ensure_dir,$(AOXC_HOME_REAL))
	$(call ensure_dir,$(AOXC_REAL_LOG_DIR))
	@echo "Prepared AOXC_HOME=$(AOXC_HOME_REAL)"
	@echo "Prepared LOG_DIR=$(AOXC_REAL_LOG_DIR)"

real-chain-run: real-chain-prep
	AOXC_HOME_DIR="$(AOXC_HOME_REAL)" LOG_DIR="$(AOXC_REAL_LOG_DIR)" ./scripts/real_chain_daemon.sh

real-chain-run-once: real-chain-prep
	MAX_CYCLES=1 AOXC_HOME_DIR="$(AOXC_HOME_REAL)" LOG_DIR="$(AOXC_REAL_LOG_DIR)" ./scripts/real_chain_daemon.sh

real-chain-health: package-bin
	"$(AOXC_BIN_PATH)" network-smoke --timeout-ms 3000 --bind-host 127.0.0.1 --port 0 --payload AOXC_REAL_HEALTH

real-chain-tail:
	tail -n 120 -f "$(AOXC_REAL_LOG_DIR)/runtime.log" "$(AOXC_REAL_LOG_DIR)/health.log"

# --------------------------------------------------------------------
# Network daemon wrappers
# --------------------------------------------------------------------
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

net-dual-start: package-bin
	./scripts/network_stack.sh start

net-dual-once: package-bin
	./scripts/network_stack.sh once

net-dual-status:
	./scripts/network_stack.sh status

net-dual-stop:
	./scripts/network_stack.sh stop

net-dual-restart: package-bin
	./scripts/network_stack.sh restart

# --------------------------------------------------------------------
# Easy operator wrappers
# --------------------------------------------------------------------
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

ops-start-dual: package-bin
	./scripts/aoxc_easy.sh start-dual

ops-auto-start: package-bin
	./scripts/aoxc_easy.sh auto-start

ops-auto-once: package-bin
	./scripts/aoxc_easy.sh auto-once

ops-once-mainnet: package-bin
	./scripts/aoxc_easy.sh once mainnet

ops-once-testnet: package-bin
	./scripts/aoxc_easy.sh once testnet

ops-once-devnet: package-bin
	./scripts/aoxc_easy.sh once devnet

ops-stop-mainnet:
	./scripts/aoxc_easy.sh stop mainnet

ops-stop-testnet:
	./scripts/aoxc_easy.sh stop testnet

ops-stop-devnet:
	./scripts/aoxc_easy.sh stop devnet

ops-stop-dual:
	./scripts/aoxc_easy.sh stop-dual

ops-status-mainnet:
	./scripts/aoxc_easy.sh status mainnet

ops-status-testnet:
	./scripts/aoxc_easy.sh status testnet

ops-status-devnet:
	./scripts/aoxc_easy.sh status devnet

ops-status-dual:
	./scripts/aoxc_easy.sh status-dual

ops-restart-mainnet: package-bin
	./scripts/aoxc_easy.sh restart mainnet

ops-restart-testnet: package-bin
	./scripts/aoxc_easy.sh restart testnet

ops-restart-devnet: package-bin
	./scripts/aoxc_easy.sh restart devnet

ops-restart-dual: package-bin
	./scripts/aoxc_easy.sh restart-dual

ops-logs-mainnet:
	./scripts/aoxc_easy.sh logs mainnet

ops-logs-testnet:
	./scripts/aoxc_easy.sh logs testnet

ops-logs-devnet:
	./scripts/aoxc_easy.sh logs devnet

ops-dashboard:
	./scripts/aoxc_easy.sh dashboard

ops-flow-mainnet:
	./scripts/aoxc_easy.sh flow mainnet

ops-flow-testnet:
	./scripts/aoxc_easy.sh flow testnet

ops-flow-devnet:
	./scripts/aoxc_easy.sh flow devnet

# --------------------------------------------------------------------
# Legacy / alpha convenience surface
# --------------------------------------------------------------------
alpha:
	@printf "AOXC Alpha: Genesis V1\n"
	@printf "  make policy           - print node connection policy\n"
	@printf "  make dev-bootstrap    - print suggested developer bootstrap flow\n"
	@printf "  make real-chain-run   - run the local real-chain daemon loop\n"
	@printf "  make real-chain-tail  - tail local runtime logs\n\n"

# --------------------------------------------------------------------
# AOXCHub environment-aligned entry points
# --------------------------------------------------------------------
AOXHUB_ROOT_CONFIG_MAINNET ?= configs/aoxhub/mainnet.toml
AOXHUB_ROOT_CONFIG_TESTNET ?= configs/aoxhub/testnet.toml
AOXHUB_HOME_MAINNET ?= $(AOXC_DATA_ROOT)/home/mainnet
AOXHUB_HOME_TESTNET ?= $(AOXC_DATA_ROOT)/home/testnet

.PHONY: hub-mainnet hub-testnet cli-mainnet-version cli-testnet-version

hub-mainnet:
	$(call print_banner,Launching AOXCHub in MAINNET default mode)
	@AOXCHUB_DEFAULT_ENV=mainnet cargo run -p aoxchub

hub-testnet:
	$(call print_banner,Launching AOXCHub in TESTNET default mode)
	@AOXCHUB_DEFAULT_ENV=testnet cargo run -p aoxchub

cli-mainnet-version:
	$(call print_banner,AOXC CLI version in MAINNET context)
	@AOXC_ENV=mainnet AOXHUB_CONFIG=$(AOXHUB_ROOT_CONFIG_MAINNET) AOXC_HOME=$(AOXHUB_HOME_MAINNET) $(AOXC_BIN_PATH) version

cli-testnet-version:
	$(call print_banner,AOXC CLI version in TESTNET context)
	@AOXC_ENV=testnet AOXHUB_CONFIG=$(AOXHUB_ROOT_CONFIG_TESTNET) AOXC_HOME=$(AOXHUB_HOME_TESTNET) $(AOXC_BIN_PATH) version
