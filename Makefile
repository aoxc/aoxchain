# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
#
# --------------------------------------------------------------------
# AOXC Makefile
# --------------------------------------------------------------------
# Operational design objectives:
# - Provide one auditable entry point for build, test, packaging,
#   environment installation, daemon orchestration, and operator flows.
# - Keep repository environment bundles as the sole source of truth.
# - Materialize runtime identity deterministically into AOXC-owned homes.
# - Preserve backward compatibility for legacy loaders expecting the
#   default runtime home beneath ~/.AOXCData/home/default.
# - Fail closed on unsupported environments, missing canonical artifacts,
#   or integrity mismatches.
#
# Notes:
# - This file assumes GNU Make and a Linux operator environment.
# - Script integration points are preserved through ./scripts/* helpers.
# - Runtime environment bundles are expected beneath:
#     configs/environments/<env>/
# - Canonical identity artifacts are copied into:
#     ~/.AOXCData/home/<env>/identity/
#   and mirrored into:
#     ~/.AOXCData/home/default/identity/
#   when the selected environment is activated.

SHELL := /bin/bash
.DEFAULT_GOAL := help

# --------------------------------------------------------------------
# Core tools
# --------------------------------------------------------------------
CARGO ?= cargo
RUSTFMT ?= rustfmt
PYTHON ?= python3
BASH ?= bash
SHA256SUM ?= sha256sum
TAR ?= tar

# --------------------------------------------------------------------
# Workspace quality flags
# --------------------------------------------------------------------
CLIPPY_FLAGS ?= --workspace --all-targets --all-features
TEST_FLAGS ?= --workspace
CHECK_FLAGS ?= --workspace

# --------------------------------------------------------------------
# Canonical AOXC path contract
# --------------------------------------------------------------------
AOXC_DATA_ROOT ?= $(HOME)/.AOXCData
AOXC_HOME ?= $(AOXC_DATA_ROOT)/home/default
AOXC_BIN_DIR ?= $(AOXC_DATA_ROOT)/bin
AOXC_BIN_PATH ?= $(AOXC_BIN_DIR)/aoxc
AOXC_RELEASES_DIR ?= $(AOXC_DATA_ROOT)/releases
AOXC_NETWORK_BIN_ROOT ?= $(AOXC_DATA_ROOT)/binary

AOXC_HOME_LOCAL ?= $(AOXC_DATA_ROOT)/home/local-dev
AOXC_HOME_REAL ?= $(AOXC_DATA_ROOT)/home/real
AOXC_HOME_MAINNET ?= $(AOXC_DATA_ROOT)/home/mainnet
AOXC_HOME_TESTNET ?= $(AOXC_DATA_ROOT)/home/testnet
AOXC_HOME_DEVNET ?= $(AOXC_DATA_ROOT)/home/devnet
AOXC_HOME_LOCALNET ?= $(AOXC_DATA_ROOT)/home/localnet
AOXC_HOME_DESKTOP_TESTNET ?= $(AOXC_DATA_ROOT)/desktop/testnet/home

AOXC_LOG_ROOT ?= $(AOXC_DATA_ROOT)/logs
AOXC_REAL_LOG_DIR ?= $(AOXC_LOG_ROOT)/real-chain
AOXC_MAINNET_LOG_DIR ?= $(AOXC_LOG_ROOT)/mainnet
AOXC_TESTNET_LOG_DIR ?= $(AOXC_LOG_ROOT)/testnet
AOXC_DEVNET_LOG_DIR ?= $(AOXC_LOG_ROOT)/devnet
AOXC_LOCALNET_LOG_DIR ?= $(AOXC_LOG_ROOT)/localnet

AOXC_DESKTOP_ROOT ?= $(AOXC_DATA_ROOT)/desktop/testnet
AOXC_DESKTOP_BIN_DIR ?= $(AOXC_DESKTOP_ROOT)/bin
AOXC_DESKTOP_LOG_DIR ?= $(AOXC_DESKTOP_ROOT)/logs

# --------------------------------------------------------------------
# Release metadata
# --------------------------------------------------------------------
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
# Canonical environment bundle contract
# --------------------------------------------------------------------
AOXC_ENV ?= devnet
AOXC_ENVIRONMENTS_ROOT ?= configs/environments
AOXC_ENV_SOURCE_DIR ?= $(AOXC_ENVIRONMENTS_ROOT)/$(AOXC_ENV)

ENV_MANIFEST_FILE ?= manifest.v1.json
ENV_GENESIS_FILE ?= genesis.v1.json
ENV_GENESIS_SHA256_FILE ?= genesis.v1.sha256
ENV_VALIDATORS_FILE ?= validators.json
ENV_BOOTNODES_FILE ?= bootnodes.json
ENV_CERTIFICATE_FILE ?= certificate.json
ENV_PROFILE_FILE ?= profile.toml
ENV_RELEASE_POLICY_FILE ?= release-policy.toml

ENV_RUNTIME_MANIFEST_FILE ?= manifest.json
ENV_RUNTIME_GENESIS_FILE ?= genesis.json
ENV_RUNTIME_GENESIS_SHA256_FILE ?= genesis.sha256
ENV_RUNTIME_VALIDATORS_FILE ?= validators.json
ENV_RUNTIME_BOOTNODES_FILE ?= bootnodes.json
ENV_RUNTIME_CERTIFICATE_FILE ?= certificate.json
ENV_RUNTIME_PROFILE_FILE ?= profile.toml
ENV_RUNTIME_RELEASE_POLICY_FILE ?= release-policy.toml
ENV_RUNTIME_ACTIVE_FILE ?= active-environment
ENV_RUNTIME_INSTALL_RECEIPT ?= install.receipt
ENV_RUNTIME_FINGERPRINT_FILE ?= genesis.fingerprint.sha256

# --------------------------------------------------------------------
# Shared shell helpers
# --------------------------------------------------------------------
define print_banner
	@printf "\n==> %s\n" "$(1)"
endef

define require_file
	@test -f "$(1)" || { echo "Missing required file: $(1)"; exit 1; }
endef

define require_dir
	@test -d "$(1)" || { echo "Missing required directory: $(1)"; exit 1; }
endef

define ensure_dir
	@mkdir -p "$(1)"
endef

define require_command
	@command -v "$(1)" >/dev/null 2>&1 || { echo "Missing required command: $(1)"; exit 1; }
endef

define remove_if_exists
	@if [ -e "$(1)" ]; then rm -rf "$(1)"; fi
endef

define copy_if_present
	@if [ -f "$(1)" ]; then cp "$(1)" "$(2)"; fi
endef

define assert_supported_env
	@case "$(1)" in \
		mainnet|testnet|devnet|localnet|local-dev) ;; \
		*) echo "Unsupported AOXC_ENV: $(1)"; exit 1 ;; \
	esac
endef

define env_home_for
$(if $(filter mainnet,$(1)),$(AOXC_HOME_MAINNET),\
$(if $(filter testnet,$(1)),$(AOXC_HOME_TESTNET),\
$(if $(filter devnet,$(1)),$(AOXC_HOME_DEVNET),\
$(if $(filter localnet,$(1)),$(AOXC_HOME_LOCALNET),\
$(if $(filter local-dev,$(1)),$(AOXC_HOME_LOCAL),)))))
endef

define env_log_for
$(if $(filter mainnet,$(1)),$(AOXC_MAINNET_LOG_DIR),\
$(if $(filter testnet,$(1)),$(AOXC_TESTNET_LOG_DIR),\
$(if $(filter devnet,$(1)),$(AOXC_DEVNET_LOG_DIR),\
$(if $(filter localnet,$(1)),$(AOXC_LOCALNET_LOG_DIR),\
$(if $(filter local-dev,$(1)),$(AOXC_LOG_ROOT)/local-dev,)))))
endef

define runtime_home
$(call env_home_for,$(AOXC_ENV))
endef

define runtime_log_dir
$(call env_log_for,$(AOXC_ENV))
endef

define runtime_identity_dir
$(call runtime_home)/identity
endef

define runtime_state_dir
$(call runtime_home)/state
endef

define runtime_config_dir
$(call runtime_home)/config
endef

define runtime_operator_dir
$(call runtime_home)/operator
endef

define default_identity_dir
$(AOXC_HOME)/identity
endef

# --------------------------------------------------------------------
# Phony targets
# --------------------------------------------------------------------
.PHONY: \
	help paths env-check bootstrap-paths bootstrap-desktop-paths bootstrap-env-paths \
	clean-home clean-logs clean-env-home clean-env-logs clean-env-identity \
	build build-release build-release-all build-release-mainnet build-release-testnet build-release-devnet build-release-matrix \
	package-bin release-binary-list package-all-bin package-versioned-bin package-versioned-archive package-network-versioned-bin package-desktop-testnet install-bin publish-release \
	test test-lib test-workspace check fmt clippy audit \
	quality quality-quick quality-release ci \
	db-init-sqlite db-status-sqlite db-event-sqlite db-release-sqlite db-history-sqlite \
	version manifest policy \
	env-print env-source-check env-install env-verify env-activate env-status env-fingerprint env-doctor env-reinstall env-clean env-reset env-show-active env-sync-default \
	env-install-mainnet env-install-testnet env-install-devnet env-install-localnet \
	env-verify-mainnet env-verify-testnet env-verify-devnet env-verify-localnet \
	env-activate-mainnet env-activate-testnet env-activate-devnet env-activate-localnet \
	env-bootstrap-mainnet env-bootstrap-testnet env-bootstrap-devnet env-bootstrap-localnet \
	dev-bootstrap run-local supervise-local audit-install produce-loop \
	real-chain-prep real-chain-run real-chain-run-once real-chain-health real-chain-tail \
	net-mainnet-start net-mainnet-once net-mainnet-status net-mainnet-stop \
	net-testnet-start net-testnet-once net-testnet-status net-testnet-stop \
	net-devnet-start net-devnet-once net-devnet-status net-devnet-stop \
	net-dual-start net-dual-once net-dual-status net-dual-stop net-dual-restart \
	ops-help ops-doctor ops-auto-prepare ops-auto-bootstrap \
	ops-start-mainnet ops-start-testnet ops-start-devnet ops-start-dual ops-auto-start ops-auto-once \
	ops-stop-mainnet ops-stop-testnet ops-stop-devnet ops-stop-dual \
	ops-status-mainnet ops-status-testnet ops-status-devnet ops-status-dual \
	ops-restart-mainnet ops-restart-testnet ops-restart-devnet ops-restart-dual \
	ops-logs-mainnet ops-logs-testnet ops-logs-devnet ops-dashboard ops-flow-mainnet ops-flow-testnet ops-flow-devnet ops-autonomy-blueprint \
	ui-mainnet ui-testnet ui-devnet alpha

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
	@printf "  make build-release-mainnet - produce mainnet-scoped binary artifact\n"
	@printf "  make build-release-testnet - produce testnet-scoped binary artifact\n"
	@printf "  make build-release-devnet - produce devnet-scoped binary artifact\n"
	@printf "  make build-release-matrix - build all network-scoped release artifacts\n"
	@printf "  make package-bin       - install release binary into %s\n" "$(AOXC_BIN_DIR)"
	@printf "  make build-release-all - build all workspace release binaries\n"
	@printf "  make release-binary-list - print detected workspace binary names\n"
	@printf "  make package-all-bin   - install all release binaries into %s\n" "$(AOXC_BIN_DIR)"
	@printf "  make package-versioned-bin - install all binaries into versioned bundle under %s\n" "$(AOXC_RELEASES_DIR)"
	@printf "  make package-versioned-archive - create tar.gz archive for the versioned bundle\n"
	@printf "  make package-network-versioned-bin - install per-network versioned AOXC CLI copies under %s\n" "$(AOXC_NETWORK_BIN_ROOT)"
	@printf "  make publish-release   - create release archive and generate release evidence bundle\n"
	@printf "  make package-desktop-testnet - install all binaries under desktop/testnet root\n"
	@printf "  make version           - show AOXC build/version metadata\n"
	@printf "  make manifest          - print build manifest\n"
	@printf "  make policy            - print node connection policy\n\n"

	@printf "Environment and paths\n"
	@printf "  make paths             - print resolved AOXC paths\n"
	@printf "  make env-check         - validate required local tools and scripts\n"
	@printf "  make bootstrap-paths   - create canonical AOXC directories\n"
	@printf "  make bootstrap-desktop-paths - create desktop/testnet directories\n"
	@printf "  make bootstrap-env-paths AOXC_ENV=devnet - create runtime subdirectories for selected env\n"
	@printf "  make env-print AOXC_ENV=devnet - print resolved env source/runtime paths\n"
	@printf "  make env-source-check AOXC_ENV=devnet - validate canonical environment bundle\n"
	@printf "  make env-install AOXC_ENV=devnet - install canonical identity bundle into runtime home\n"
	@printf "  make env-verify AOXC_ENV=devnet - verify installed runtime identity against canonical source\n"
	@printf "  make env-activate AOXC_ENV=devnet - install, verify, and activate environment\n"
	@printf "  make env-sync-default AOXC_ENV=devnet - mirror selected env identity into default runtime home\n"
	@printf "  make env-status AOXC_ENV=devnet - print runtime identity status\n"
	@printf "  make env-fingerprint AOXC_ENV=devnet - print runtime genesis fingerprint\n"
	@printf "  make env-doctor AOXC_ENV=devnet - run end-to-end environment readiness diagnostics\n"
	@printf "  make env-reinstall AOXC_ENV=devnet - clean and reinstall runtime identity\n"
	@printf "  make env-reset AOXC_ENV=devnet - remove runtime home and log material for selected env\n"
	@printf "  make env-show-active AOXC_ENV=devnet - print active environment markers\n"
	@printf "  make db-init-sqlite    - initialize sqlite-backed operator memory and AOXC db layout\n"
	@printf "  make db-status-sqlite  - print sqlite-backed operator memory status\n"
	@printf "  make db-history-sqlite - print recent autonomous operation history\n"
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
	@printf "  make ops-auto-prepare     - bootstrap paths and activate AOXC_ENV automatically\n"
	@printf "  make ops-auto-bootstrap   - prepare environment and start AOXC_ENV automatically\n"
	@printf "  make ops-start-mainnet    - start mainnet quickly\n"
	@printf "  make ops-start-testnet    - start testnet quickly\n"
	@printf "  make ops-start-devnet     - start devnet quickly\n"
	@printf "  make ops-start-dual       - start testnet+mainnet together\n"
	@printf "  make ops-auto-start       - start AOXC_ENV (default devnet) automatically\n"
	@printf "  make ops-auto-once        - run one cycle on AOXC_ENV (default devnet)\n"
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
	@printf "  make ops-logs-devnet      - tail devnet logs\n"
	@printf "  make ops-dashboard        - show full multi-env dashboard\n"
	@printf "  make ops-flow-mainnet     - full auto operational flow (mainnet)\n"
	@printf "  make ops-flow-testnet     - full auto operational flow (testnet)\n"
	@printf "  make ops-flow-devnet      - full auto operational flow (devnet)\n"
	@printf "  make ops-autonomy-blueprint - print autonomous system delivery blueprint\n\n"

	@printf "AOXCHub UI surfaces\n"
	@printf "  make ui-mainnet        - run AOXCHub with mainnet profile\n"
	@printf "  make ui-testnet        - run AOXCHub with testnet profile\n"
	@printf "  make ui-devnet         - run AOXCHub with testnet-safe defaults and devnet label\n\n"

paths:
	@printf "AOXC_ENV=%s\n" "$(AOXC_ENV)"
	@printf "AOXC_DATA_ROOT=%s\n" "$(AOXC_DATA_ROOT)"
	@printf "AOXC_HOME=%s\n" "$(AOXC_HOME)"
	@printf "AOXC_HOME_LOCAL=%s\n" "$(AOXC_HOME_LOCAL)"
	@printf "AOXC_HOME_REAL=%s\n" "$(AOXC_HOME_REAL)"
	@printf "AOXC_HOME_MAINNET=%s\n" "$(AOXC_HOME_MAINNET)"
	@printf "AOXC_HOME_TESTNET=%s\n" "$(AOXC_HOME_TESTNET)"
	@printf "AOXC_HOME_DEVNET=%s\n" "$(AOXC_HOME_DEVNET)"
	@printf "AOXC_HOME_LOCALNET=%s\n" "$(AOXC_HOME_LOCALNET)"
	@printf "AOXC_HOME_DESKTOP_TESTNET=%s\n" "$(AOXC_HOME_DESKTOP_TESTNET)"
	@printf "AOXC_BIN_DIR=%s\n" "$(AOXC_BIN_DIR)"
	@printf "AOXC_BIN_PATH=%s\n" "$(AOXC_BIN_PATH)"
	@printf "AOXC_RELEASES_DIR=%s\n" "$(AOXC_RELEASES_DIR)"
	@printf "AOXC_NETWORK_BIN_ROOT=%s\n" "$(AOXC_NETWORK_BIN_ROOT)"
	@printf "AOXC_LOG_ROOT=%s\n" "$(AOXC_LOG_ROOT)"
	@printf "AOXC_REAL_LOG_DIR=%s\n" "$(AOXC_REAL_LOG_DIR)"
	@printf "AOXC_MAINNET_LOG_DIR=%s\n" "$(AOXC_MAINNET_LOG_DIR)"
	@printf "AOXC_TESTNET_LOG_DIR=%s\n" "$(AOXC_TESTNET_LOG_DIR)"
	@printf "AOXC_DEVNET_LOG_DIR=%s\n" "$(AOXC_DEVNET_LOG_DIR)"
	@printf "AOXC_LOCALNET_LOG_DIR=%s\n" "$(AOXC_LOCALNET_LOG_DIR)"
	@printf "AOXC_DESKTOP_ROOT=%s\n" "$(AOXC_DESKTOP_ROOT)"
	@printf "AOXC_DESKTOP_BIN_DIR=%s\n" "$(AOXC_DESKTOP_BIN_DIR)"
	@printf "AOXC_DESKTOP_LOG_DIR=%s\n" "$(AOXC_DESKTOP_LOG_DIR)"
	@printf "AOXC_ENV_SOURCE_DIR=%s\n" "$(AOXC_ENV_SOURCE_DIR)"
	@printf "AOXC_RUNTIME_HOME=%s\n" "$(call runtime_home)"
	@printf "AOXC_RUNTIME_IDENTITY_DIR=%s\n" "$(call runtime_identity_dir)"
	@printf "AOXC_RUNTIME_LOG_DIR=%s\n" "$(call runtime_log_dir)"
	@printf "RELEASE_TAG=%s\n" "$(RELEASE_TAG)"
	@printf "RELEASE_BUNDLE_DIR=%s\n" "$(RELEASE_BUNDLE_DIR)"
	@printf "RELEASE_ARCHIVE_PATH=%s\n" "$(RELEASE_ARCHIVE_PATH)"

env-check:
	$(call print_banner,Validating local build environment)
	@command -v $(CARGO) >/dev/null 2>&1 || { echo "cargo not found"; exit 1; }
	@command -v git >/dev/null 2>&1 || { echo "git not found"; exit 1; }
	@command -v bash >/dev/null 2>&1 || { echo "bash not found"; exit 1; }
	@command -v python3 >/dev/null 2>&1 || { echo "python3 not found"; exit 1; }
	@command -v sha256sum >/dev/null 2>&1 || { echo "sha256sum not found"; exit 1; }
	$(call require_file,./scripts/quality_gate.sh)
	$(call require_file,./scripts/run-local.sh)
	$(call require_file,./scripts/node_supervisor.sh)
	$(call require_file,./scripts/continuous_producer.sh)
	$(call require_file,./scripts/real_chain_daemon.sh)
	$(call require_file,./scripts/network_env_daemon.sh)
	$(call require_file,./scripts/network_stack.sh)
	$(call require_file,./scripts/aoxc_easy.sh)
	$(call require_file,./scripts/autonomy_sqlite_ctl.py)
	@echo "Environment check passed."

# --------------------------------------------------------------------
# Path bootstrap and cleanup
# --------------------------------------------------------------------
bootstrap-paths:
	$(call print_banner,Creating canonical AOXC directories)
	$(call ensure_dir,$(AOXC_DATA_ROOT))
	$(call ensure_dir,$(AOXC_HOME))
	$(call ensure_dir,$(AOXC_HOME_LOCAL))
	$(call ensure_dir,$(AOXC_HOME_REAL))
	$(call ensure_dir,$(AOXC_HOME_MAINNET))
	$(call ensure_dir,$(AOXC_HOME_TESTNET))
	$(call ensure_dir,$(AOXC_HOME_DEVNET))
	$(call ensure_dir,$(AOXC_HOME_LOCALNET))
	$(call ensure_dir,$(AOXC_BIN_DIR))
	$(call ensure_dir,$(AOXC_RELEASES_DIR))
	$(call ensure_dir,$(AOXC_NETWORK_BIN_ROOT))
	$(call ensure_dir,$(AOXC_LOG_ROOT))
	$(call ensure_dir,$(AOXC_REAL_LOG_DIR))
	$(call ensure_dir,$(AOXC_MAINNET_LOG_DIR))
	$(call ensure_dir,$(AOXC_TESTNET_LOG_DIR))
	$(call ensure_dir,$(AOXC_DEVNET_LOG_DIR))
	$(call ensure_dir,$(AOXC_LOCALNET_LOG_DIR))
	$(call ensure_dir,$(AOXC_DATA_ROOT)/.test)
	@echo "AOXC path bootstrap complete."

bootstrap-desktop-paths:
	$(call print_banner,Creating AOXC desktop testnet directories)
	$(call ensure_dir,$(AOXC_DESKTOP_ROOT))
	$(call ensure_dir,$(AOXC_HOME_DESKTOP_TESTNET))
	$(call ensure_dir,$(AOXC_DESKTOP_BIN_DIR))
	$(call ensure_dir,$(AOXC_DESKTOP_LOG_DIR))
	@echo "AOXC desktop testnet path bootstrap complete."

bootstrap-env-paths: bootstrap-paths
	$(call print_banner,Creating runtime subdirectories for the selected AOXC environment)
	$(call assert_supported_env,$(AOXC_ENV))
	$(call ensure_dir,$(call runtime_home))
	$(call ensure_dir,$(call runtime_identity_dir))
	$(call ensure_dir,$(call runtime_state_dir))
	$(call ensure_dir,$(call runtime_config_dir))
	$(call ensure_dir,$(call runtime_operator_dir))
	$(call ensure_dir,$(call runtime_log_dir))
	@echo "AOXC runtime subdirectories created for environment: $(AOXC_ENV)"

clean-home:
	$(call print_banner,Removing effective AOXC home)
	@rm -rf "$(AOXC_HOME)"
	@echo "Removed: $(AOXC_HOME)"

clean-logs:
	$(call print_banner,Removing AOXC logs)
	@rm -rf "$(AOXC_LOG_ROOT)"
	@echo "Removed: $(AOXC_LOG_ROOT)"

clean-env-home:
	$(call print_banner,Removing selected environment runtime home)
	$(call assert_supported_env,$(AOXC_ENV))
	@rm -rf "$(call runtime_home)"
	@echo "Removed: $(call runtime_home)"

clean-env-logs:
	$(call print_banner,Removing selected environment log directory)
	$(call assert_supported_env,$(AOXC_ENV))
	@rm -rf "$(call runtime_log_dir)"
	@echo "Removed: $(call runtime_log_dir)"

clean-env-identity:
	$(call print_banner,Removing selected environment identity directory)
	$(call assert_supported_env,$(AOXC_ENV))
	@rm -rf "$(call runtime_identity_dir)"
	@echo "Removed: $(call runtime_identity_dir)"

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

build-release-mainnet: build-release bootstrap-paths
	$(call print_banner,Producing mainnet-scoped AOXC binary)
	@mkdir -p "$(AOXC_NETWORK_BIN_ROOT)/mainnet"
	@cp target/release/aoxc "$(AOXC_NETWORK_BIN_ROOT)/mainnet/aoxc-$(RELEASE_TAG)-mainnet"
	@chmod +x "$(AOXC_NETWORK_BIN_ROOT)/mainnet/aoxc-$(RELEASE_TAG)-mainnet"
	@ln -sfn "$(AOXC_NETWORK_BIN_ROOT)/mainnet/aoxc-$(RELEASE_TAG)-mainnet" "$(AOXC_NETWORK_BIN_ROOT)/mainnet/aoxc-current"

build-release-testnet: build-release bootstrap-paths
	$(call print_banner,Producing testnet-scoped AOXC binary)
	@mkdir -p "$(AOXC_NETWORK_BIN_ROOT)/testnet"
	@cp target/release/aoxc "$(AOXC_NETWORK_BIN_ROOT)/testnet/aoxc-$(RELEASE_TAG)-testnet"
	@chmod +x "$(AOXC_NETWORK_BIN_ROOT)/testnet/aoxc-$(RELEASE_TAG)-testnet"
	@ln -sfn "$(AOXC_NETWORK_BIN_ROOT)/testnet/aoxc-$(RELEASE_TAG)-testnet" "$(AOXC_NETWORK_BIN_ROOT)/testnet/aoxc-current"

build-release-devnet: build-release bootstrap-paths
	$(call print_banner,Producing devnet-scoped AOXC binary)
	@mkdir -p "$(AOXC_NETWORK_BIN_ROOT)/devnet"
	@cp target/release/aoxc "$(AOXC_NETWORK_BIN_ROOT)/devnet/aoxc-$(RELEASE_TAG)-devnet"
	@chmod +x "$(AOXC_NETWORK_BIN_ROOT)/devnet/aoxc-$(RELEASE_TAG)-devnet"
	@ln -sfn "$(AOXC_NETWORK_BIN_ROOT)/devnet/aoxc-$(RELEASE_TAG)-devnet" "$(AOXC_NETWORK_BIN_ROOT)/devnet/aoxc-current"

build-release-matrix: build-release-mainnet build-release-testnet build-release-devnet
	$(call print_banner,Completed release matrix build)

package-bin: build-release bootstrap-paths
	$(call print_banner,Installing release AOXC CLI into canonical bin directory)
	@cp target/release/aoxc "$(AOXC_BIN_PATH)"
	@chmod +x "$(AOXC_BIN_PATH)"
	@echo "Installed: $(AOXC_BIN_PATH)"

release-binary-list:
	$(call print_banner,Printing configured release binary names)
	@printf "%s\n" $(RELEASE_BINARIES)

package-all-bin: build-release-all bootstrap-paths
	$(call print_banner,Installing all release AOXC binaries into canonical bin directory)
	@for bin in $(RELEASE_BINARIES); do \
		test -f "target/release/$$bin" || { echo "Missing built binary: target/release/$$bin"; exit 1; }; \
		cp "target/release/$$bin" "$(AOXC_BIN_DIR)/$$bin"; \
		chmod +x "$(AOXC_BIN_DIR)/$$bin"; \
	done
	@echo "Installed binaries into: $(AOXC_BIN_DIR)"

package-versioned-bin: build-release-all bootstrap-paths
	$(call print_banner,Installing release binaries into versioned bundle)
	@mkdir -p "$(RELEASE_BUNDLE_BIN_DIR)"
	@for bin in $(RELEASE_BINARIES); do \
		test -f "target/release/$$bin" || { echo "Missing built binary: target/release/$$bin"; exit 1; }; \
		cp "target/release/$$bin" "$(RELEASE_BUNDLE_BIN_DIR)/$$bin"; \
		chmod +x "$(RELEASE_BUNDLE_BIN_DIR)/$$bin"; \
	done
	@$(MAKE) manifest > "$(RELEASE_BUNDLE_MANIFEST)"
	@cd "$(RELEASE_BUNDLE_DIR)" && $(SHA256SUM) bin/* > "$(RELEASE_BUNDLE_CHECKSUMS)"
	@echo "Versioned release bundle created at: $(RELEASE_BUNDLE_DIR)"

package-versioned-archive: package-versioned-bin
	$(call print_banner,Creating versioned release archive)
	@mkdir -p "$(AOXC_RELEASES_DIR)"
	@cd "$(AOXC_RELEASES_DIR)" && $(TAR) -czf "$(notdir $(RELEASE_ARCHIVE_PATH))" "$(RELEASE_BUNDLE_NAME)"
	@echo "Archive created at: $(RELEASE_ARCHIVE_PATH)"

package-network-versioned-bin: build-release bootstrap-paths
	$(call print_banner,Installing per-network versioned AOXC CLI copies)
	@for env in mainnet testnet devnet; do \
		mkdir -p "$(AOXC_NETWORK_BIN_ROOT)/$$env"; \
		cp target/release/aoxc "$(AOXC_NETWORK_BIN_ROOT)/$$env/aoxc-$(RELEASE_TAG)-$$env"; \
		chmod +x "$(AOXC_NETWORK_BIN_ROOT)/$$env/aoxc-$(RELEASE_TAG)-$$env"; \
		ln -sfn "$(AOXC_NETWORK_BIN_ROOT)/$$env/aoxc-$(RELEASE_TAG)-$$env" "$(AOXC_NETWORK_BIN_ROOT)/$$env/aoxc-current"; \
	done
	@echo "Per-network versioned binaries installed under: $(AOXC_NETWORK_BIN_ROOT)"

package-desktop-testnet: build-release-all bootstrap-desktop-paths
	$(call print_banner,Packaging desktop testnet binaries and profile layout)
	@mkdir -p "$(AOXC_DESKTOP_BIN_DIR)"
	@cp target/release/aoxc "$(AOXC_DESKTOP_BIN_DIR)/aoxc"
	@cp target/release/aoxchub "$(AOXC_DESKTOP_BIN_DIR)/aoxchub"
	@cp target/release/aoxckit "$(AOXC_DESKTOP_BIN_DIR)/aoxckit"
	@chmod +x "$(AOXC_DESKTOP_BIN_DIR)/aoxc" "$(AOXC_DESKTOP_BIN_DIR)/aoxchub" "$(AOXC_DESKTOP_BIN_DIR)/aoxckit"
	@echo "Desktop binaries installed under: $(AOXC_DESKTOP_BIN_DIR)"
	@echo "Use AOXC_HOME=$(AOXC_HOME_DESKTOP_TESTNET) for desktop-testnet runtime isolation."

publish-release: package-versioned-archive db-release-sqlite
	$(call print_banner,Release publication evidence completed)
	@echo "Release archive: $(RELEASE_ARCHIVE_PATH)"

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
# Database / autonomy memory
# --------------------------------------------------------------------
db-init-sqlite:
	$(call print_banner,Initializing sqlite-backed AOXC operator state)
	@$(CARGO) run -p aoxcmd -- db-init --backend sqlite
	@$(PYTHON) ./scripts/autonomy_sqlite_ctl.py init

db-status-sqlite:
	$(call print_banner,Printing sqlite-backed AOXC operator state)
	@$(CARGO) run -p aoxcmd -- db-status --backend sqlite
	@$(PYTHON) ./scripts/autonomy_sqlite_ctl.py status

db-event-sqlite:
	$(call print_banner,Recording autonomous event into sqlite operator memory)
	@$(PYTHON) ./scripts/autonomy_sqlite_ctl.py event --env "$${AOXC_ENV:-devnet}" --action "$${ACTION:-heartbeat}" --status "$${STATUS:-ok}" --detail "$${DETAIL:-make-db-event-sqlite}"

db-release-sqlite:
	$(call print_banner,Recording release publication metadata in sqlite operator memory)
	@$(PYTHON) ./scripts/autonomy_sqlite_ctl.py release --version "$(RELEASE_TAG)" --artifact "$(RELEASE_ARCHIVE_PATH)"

db-history-sqlite:
	$(call print_banner,Recent autonomous sqlite event history)
	@$(PYTHON) ./scripts/autonomy_sqlite_ctl.py history --limit "$${LIMIT:-30}"

# --------------------------------------------------------------------
# Informational CLI surfaces
# --------------------------------------------------------------------
version:
	$(CARGO) run -p aoxcmd -- version

manifest:
	$(CARGO) run -p aoxcmd -- build-manifest

policy:
	$(CARGO) run -p aoxcmd -- node-connection-policy

## --------------------------------------------------------------------
# Canonical environment lifecycle
# --------------------------------------------------------------------

env-print:
	$(call print_banner,Printing resolved AOXC environment paths)
	$(call assert_supported_env,$(AOXC_ENV))
	@printf "AOXC_ENV=%s\n" "$(AOXC_ENV)"
	@printf "AOXC_ENV_SOURCE_DIR=%s\n" "$(AOXC_ENV_SOURCE_DIR)"
	@printf "AOXC_RUNTIME_HOME=%s\n" "$(call runtime_home)"
	@printf "AOXC_RUNTIME_IDENTITY_DIR=%s\n" "$(call runtime_identity_dir)"
	@printf "AOXC_RUNTIME_STATE_DIR=%s\n" "$(call runtime_state_dir)"
	@printf "AOXC_RUNTIME_CONFIG_DIR=%s\n" "$(call runtime_config_dir)"
	@printf "AOXC_RUNTIME_OPERATOR_DIR=%s\n" "$(call runtime_operator_dir)"
	@printf "AOXC_RUNTIME_LOG_DIR=%s\n" "$(call runtime_log_dir)"
	@printf "AOXC_DEFAULT_IDENTITY_DIR=%s\n" "$(call default_identity_dir)"
	@printf "ENV_MANIFEST_SOURCE=%s\n" "$(AOXC_ENV_SOURCE_DIR)/$(ENV_MANIFEST_FILE)"
	@printf "ENV_GENESIS_SOURCE=%s\n" "$(AOXC_ENV_SOURCE_DIR)/$(ENV_GENESIS_FILE)"
	@printf "ENV_GENESIS_SHA256_SOURCE=%s\n" "$(AOXC_ENV_SOURCE_DIR)/$(ENV_GENESIS_SHA256_FILE)"

env-refresh-genesis-sha256:
	$(call print_banner,Refreshing canonical genesis digest sidecar)
	$(call assert_supported_env,$(AOXC_ENV))
	$(call require_command,sha256sum)
	$(call require_dir,$(AOXC_ENV_SOURCE_DIR))
	$(call require_file,$(AOXC_ENV_SOURCE_DIR)/$(ENV_GENESIS_FILE))
	@cd "$(AOXC_ENV_SOURCE_DIR)" && $(SHA256SUM) "$(ENV_GENESIS_FILE)" > "$(ENV_GENESIS_SHA256_FILE)"
	@echo "Refreshed: $(AOXC_ENV_SOURCE_DIR)/$(ENV_GENESIS_SHA256_FILE)"

env-refresh-all-genesis-sha256:
	$(call print_banner,Refreshing canonical genesis digest sidecars for mainnet, testnet, and devnet)
	@for env in mainnet testnet devnet; do \
		$(MAKE) --no-print-directory env-refresh-genesis-sha256 AOXC_ENV="$$env" || exit 1; \
	done
	@echo "Refreshed genesis digest sidecars for: mainnet testnet devnet"

env-source-check:
	$(call print_banner,Validating canonical environment bundle)
	$(call assert_supported_env,$(AOXC_ENV))
	$(call require_command,sha256sum)
	$(call require_dir,$(AOXC_ENV_SOURCE_DIR))
	$(call require_file,$(AOXC_ENV_SOURCE_DIR)/$(ENV_MANIFEST_FILE))
	$(call require_file,$(AOXC_ENV_SOURCE_DIR)/$(ENV_GENESIS_FILE))
	$(call require_file,$(AOXC_ENV_SOURCE_DIR)/$(ENV_VALIDATORS_FILE))
	$(call require_file,$(AOXC_ENV_SOURCE_DIR)/$(ENV_BOOTNODES_FILE))
	$(call require_file,$(AOXC_ENV_SOURCE_DIR)/$(ENV_CERTIFICATE_FILE))
	$(call require_file,$(AOXC_ENV_SOURCE_DIR)/$(ENV_PROFILE_FILE))
	$(call require_file,$(AOXC_ENV_SOURCE_DIR)/$(ENV_RELEASE_POLICY_FILE))
	@$(MAKE) --no-print-directory env-refresh-genesis-sha256 AOXC_ENV="$(AOXC_ENV)"
	@cd "$(AOXC_ENV_SOURCE_DIR)" && $(SHA256SUM) -c "$(ENV_GENESIS_SHA256_FILE)"
	@echo "Canonical environment bundle is valid for: $(AOXC_ENV)"

env-source-check-all:
	$(call print_banner,Validating canonical environment bundles for mainnet, testnet, and devnet)
	@for env in mainnet testnet devnet; do \
		$(MAKE) --no-print-directory env-source-check AOXC_ENV="$$env" || exit 1; \
	done
	@echo "Canonical environment bundles are valid for: mainnet testnet devnet"

env-install: env-source-check bootstrap-env-paths
	$(call print_banner,Installing canonical environment bundle into runtime identity directory)
	$(call assert_supported_env,$(AOXC_ENV))
	@mkdir -p "$(call runtime_identity_dir)"
	@cp "$(AOXC_ENV_SOURCE_DIR)/$(ENV_MANIFEST_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_MANIFEST_FILE)"
	@cp "$(AOXC_ENV_SOURCE_DIR)/$(ENV_GENESIS_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_FILE)"
	@cp "$(AOXC_ENV_SOURCE_DIR)/$(ENV_VALIDATORS_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_VALIDATORS_FILE)"
	@cp "$(AOXC_ENV_SOURCE_DIR)/$(ENV_BOOTNODES_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_BOOTNODES_FILE)"
	@cp "$(AOXC_ENV_SOURCE_DIR)/$(ENV_CERTIFICATE_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_CERTIFICATE_FILE)"
	@cp "$(AOXC_ENV_SOURCE_DIR)/$(ENV_PROFILE_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_PROFILE_FILE)"
	@cp "$(AOXC_ENV_SOURCE_DIR)/$(ENV_RELEASE_POLICY_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_RELEASE_POLICY_FILE)"
	@cp "$(AOXC_ENV_SOURCE_DIR)/$(ENV_GENESIS_SHA256_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_SHA256_FILE)"
	@$(SHA256SUM) "$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_FILE)" | awk '{print $$1}' > "$(call runtime_identity_dir)/$(ENV_RUNTIME_FINGERPRINT_FILE)"
	@{ \
		echo "env=$(AOXC_ENV)"; \
		echo "source_dir=$(AOXC_ENV_SOURCE_DIR)"; \
		echo "runtime_home=$(call runtime_home)"; \
		echo "identity_dir=$(call runtime_identity_dir)"; \
		echo "installed_at_utc=$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
		echo "manifest_file=$(ENV_RUNTIME_MANIFEST_FILE)"; \
		echo "genesis_file=$(ENV_RUNTIME_GENESIS_FILE)"; \
		echo "validators_file=$(ENV_RUNTIME_VALIDATORS_FILE)"; \
		echo "bootnodes_file=$(ENV_RUNTIME_BOOTNODES_FILE)"; \
		echo "certificate_file=$(ENV_RUNTIME_CERTIFICATE_FILE)"; \
		echo "profile_file=$(ENV_RUNTIME_PROFILE_FILE)"; \
		echo "release_policy_file=$(ENV_RUNTIME_RELEASE_POLICY_FILE)"; \
		echo "genesis_sha256_file=$(ENV_RUNTIME_GENESIS_SHA256_FILE)"; \
		echo "genesis_fingerprint_file=$(ENV_RUNTIME_FINGERPRINT_FILE)"; \
	} > "$(call runtime_identity_dir)/$(ENV_RUNTIME_INSTALL_RECEIPT)"
	@echo "$(AOXC_ENV)" > "$(call runtime_identity_dir)/$(ENV_RUNTIME_ACTIVE_FILE)"
	@echo "Installed canonical runtime identity for: $(AOXC_ENV)"
	@echo "Runtime identity directory: $(call runtime_identity_dir)"

env-verify: env-source-check
	$(call print_banner,Verifying runtime identity materialization)
	$(call assert_supported_env,$(AOXC_ENV))
	$(call require_dir,$(call runtime_identity_dir))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_MANIFEST_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_SHA256_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_VALIDATORS_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_BOOTNODES_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_CERTIFICATE_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_PROFILE_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_RELEASE_POLICY_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_FINGERPRINT_FILE))
	@cmp -s "$(AOXC_ENV_SOURCE_DIR)/$(ENV_MANIFEST_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_MANIFEST_FILE)" || { echo "Manifest mismatch between source and runtime"; exit 1; }
	@cmp -s "$(AOXC_ENV_SOURCE_DIR)/$(ENV_GENESIS_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_FILE)" || { echo "Genesis mismatch between source and runtime"; exit 1; }
	@cmp -s "$(AOXC_ENV_SOURCE_DIR)/$(ENV_GENESIS_SHA256_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_SHA256_FILE)" || { echo "Genesis checksum sidecar mismatch between source and runtime"; exit 1; }
	@cmp -s "$(AOXC_ENV_SOURCE_DIR)/$(ENV_VALIDATORS_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_VALIDATORS_FILE)" || { echo "Validators mismatch between source and runtime"; exit 1; }
	@cmp -s "$(AOXC_ENV_SOURCE_DIR)/$(ENV_BOOTNODES_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_BOOTNODES_FILE)" || { echo "Bootnodes mismatch between source and runtime"; exit 1; }
	@cmp -s "$(AOXC_ENV_SOURCE_DIR)/$(ENV_CERTIFICATE_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_CERTIFICATE_FILE)" || { echo "Certificate mismatch between source and runtime"; exit 1; }
	@cmp -s "$(AOXC_ENV_SOURCE_DIR)/$(ENV_PROFILE_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_PROFILE_FILE)" || { echo "Profile mismatch between source and runtime"; exit 1; }
	@cmp -s "$(AOXC_ENV_SOURCE_DIR)/$(ENV_RELEASE_POLICY_FILE)" "$(call runtime_identity_dir)/$(ENV_RUNTIME_RELEASE_POLICY_FILE)" || { echo "Release policy mismatch between source and runtime"; exit 1; }
	@cd "$(call runtime_identity_dir)" && $(SHA256SUM) -c "$(ENV_RUNTIME_GENESIS_SHA256_FILE)"
	@ACTUAL_FINGERPRINT="$$(sha256sum "$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_FILE)" | awk '{print $$1}')"; \
	STORED_FINGERPRINT="$$(cat "$(call runtime_identity_dir)/$(ENV_RUNTIME_FINGERPRINT_FILE)")"; \
	[ "$$ACTUAL_FINGERPRINT" = "$$STORED_FINGERPRINT" ] || { echo "Runtime fingerprint drift detected"; exit 1; }
	@ACTIVE_ENV="$$(cat "$(call runtime_identity_dir)/$(ENV_RUNTIME_ACTIVE_FILE)" 2>/dev/null || true)"; \
	[ "$$ACTIVE_ENV" = "$(AOXC_ENV)" ] || { echo "Active environment marker mismatch: expected $(AOXC_ENV), found '$$ACTIVE_ENV'"; exit 1; }
	@echo "Runtime identity verification passed for: $(AOXC_ENV)"

env-sync-default: bootstrap-paths
	$(call print_banner,Syncing selected environment identity into the default runtime home for backward compatibility)
	$(call assert_supported_env,$(AOXC_ENV))
	$(call require_dir,$(call runtime_identity_dir))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_MANIFEST_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_SHA256_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_VALIDATORS_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_BOOTNODES_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_CERTIFICATE_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_PROFILE_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_RELEASE_POLICY_FILE))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_FINGERPRINT_FILE))
	@mkdir -p "$(call default_identity_dir)"
	@cp "$(call runtime_identity_dir)/$(ENV_RUNTIME_MANIFEST_FILE)" "$(call default_identity_dir)/$(ENV_RUNTIME_MANIFEST_FILE)"
	@cp "$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_FILE)" "$(call default_identity_dir)/$(ENV_RUNTIME_GENESIS_FILE)"
	@cp "$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_SHA256_FILE)" "$(call default_identity_dir)/$(ENV_RUNTIME_GENESIS_SHA256_FILE)"
	@cp "$(call runtime_identity_dir)/$(ENV_RUNTIME_VALIDATORS_FILE)" "$(call default_identity_dir)/$(ENV_RUNTIME_VALIDATORS_FILE)"
	@cp "$(call runtime_identity_dir)/$(ENV_RUNTIME_BOOTNODES_FILE)" "$(call default_identity_dir)/$(ENV_RUNTIME_BOOTNODES_FILE)"
	@cp "$(call runtime_identity_dir)/$(ENV_RUNTIME_CERTIFICATE_FILE)" "$(call default_identity_dir)/$(ENV_RUNTIME_CERTIFICATE_FILE)"
	@cp "$(call runtime_identity_dir)/$(ENV_RUNTIME_PROFILE_FILE)" "$(call default_identity_dir)/$(ENV_RUNTIME_PROFILE_FILE)"
	@cp "$(call runtime_identity_dir)/$(ENV_RUNTIME_RELEASE_POLICY_FILE)" "$(call default_identity_dir)/$(ENV_RUNTIME_RELEASE_POLICY_FILE)"
	@cp "$(call runtime_identity_dir)/$(ENV_RUNTIME_FINGERPRINT_FILE)" "$(call default_identity_dir)/$(ENV_RUNTIME_FINGERPRINT_FILE)"
	@echo "$(AOXC_ENV)" > "$(call default_identity_dir)/$(ENV_RUNTIME_ACTIVE_FILE)"
	@echo "$(AOXC_ENV)" > "$(AOXC_HOME)/$(ENV_RUNTIME_ACTIVE_FILE)"
	@echo "Default runtime home synchronized with environment: $(AOXC_ENV)"

env-activate: env-install env-verify env-sync-default
	$(call print_banner,Activating environment in runtime homes)
	$(call assert_supported_env,$(AOXC_ENV))
	@echo "$(AOXC_ENV)" > "$(call runtime_home)/$(ENV_RUNTIME_ACTIVE_FILE)"
	@echo "$(AOXC_ENV)" > "$(call runtime_identity_dir)/$(ENV_RUNTIME_ACTIVE_FILE)"
	@echo "$(AOXC_ENV)" > "$(AOXC_HOME)/$(ENV_RUNTIME_ACTIVE_FILE)"
	@echo "Activated environment: $(AOXC_ENV)"
	@echo "Active runtime home: $(call runtime_home)"
	@echo "Default compatibility home: $(AOXC_HOME)"

env-status:
	$(call print_banner,Printing runtime environment status)
	$(call assert_supported_env,$(AOXC_ENV))
	@echo "AOXC_ENV=$(AOXC_ENV)"
	@echo "SOURCE_DIR=$(AOXC_ENV_SOURCE_DIR)"
	@echo "RUNTIME_HOME=$(call runtime_home)"
	@echo "IDENTITY_DIR=$(call runtime_identity_dir)"
	@if [ -d "$(call runtime_identity_dir)" ]; then \
		echo "identity_directory_present=yes"; \
		find "$(call runtime_identity_dir)" -maxdepth 1 -type f | sort; \
	else \
		echo "identity_directory_present=no"; \
	fi
	@if [ -f "$(call runtime_identity_dir)/$(ENV_RUNTIME_INSTALL_RECEIPT)" ]; then \
		echo ""; \
		echo "[install receipt]"; \
		cat "$(call runtime_identity_dir)/$(ENV_RUNTIME_INSTALL_RECEIPT)"; \
	fi
	@if [ -f "$(call runtime_identity_dir)/$(ENV_RUNTIME_FINGERPRINT_FILE)" ]; then \
		echo ""; \
		echo "[genesis fingerprint]"; \
		cat "$(call runtime_identity_dir)/$(ENV_RUNTIME_FINGERPRINT_FILE)"; \
	fi
	@if [ -f "$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_SHA256_FILE)" ]; then \
		echo ""; \
		echo "[genesis checksum sidecar]"; \
		cat "$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_SHA256_FILE)"; \
	fi
	@if [ -f "$(AOXC_HOME)/$(ENV_RUNTIME_ACTIVE_FILE)" ]; then \
		echo ""; \
		echo "[default home active environment]"; \
		cat "$(AOXC_HOME)/$(ENV_RUNTIME_ACTIVE_FILE)"; \
	fi

env-fingerprint:
	$(call print_banner,Printing runtime genesis fingerprint)
	$(call assert_supported_env,$(AOXC_ENV))
	$(call require_file,$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_FILE))
	@sha256sum "$(call runtime_identity_dir)/$(ENV_RUNTIME_GENESIS_FILE)"

env-doctor:
	$(call print_banner,Running end-to-end environment diagnostics)
	$(call assert_supported_env,$(AOXC_ENV))
	@$(MAKE) env-print AOXC_ENV="$(AOXC_ENV)"
	@$(MAKE) env-source-check AOXC_ENV="$(AOXC_ENV)"
	@$(MAKE) env-status AOXC_ENV="$(AOXC_ENV)"
	@if [ -d "$(call runtime_identity_dir)" ]; then \
		$(MAKE) env-verify AOXC_ENV="$(AOXC_ENV)"; \
	else \
		echo "Runtime identity directory is absent; install has not been performed yet."; \
	fi
	@if [ -f "$(call default_identity_dir)/$(ENV_RUNTIME_GENESIS_FILE)" ]; then \
		echo "Default compatibility identity is present."; \
	else \
		echo "Default compatibility identity is absent."; \
	fi
	@echo "Environment diagnostics completed for: $(AOXC_ENV)"

env-doctor-all:
	$(call print_banner,Running end-to-end environment diagnostics for mainnet, testnet, and devnet)
	@for env in mainnet testnet devnet; do \
		$(MAKE) --no-print-directory env-doctor AOXC_ENV="$$env" || exit 1; \
	done
	@echo "Environment diagnostics completed for: mainnet testnet devnet"

env-reinstall:
	$(call print_banner,Reinstalling runtime identity from canonical bundle)
	@$(MAKE) clean-env-identity AOXC_ENV="$(AOXC_ENV)"
	@$(MAKE) env-install AOXC_ENV="$(AOXC_ENV)"
	@$(MAKE) env-verify AOXC_ENV="$(AOXC_ENV)"
	@$(MAKE) env-sync-default AOXC_ENV="$(AOXC_ENV)"

env-clean: clean-env-identity

env-reset:
	$(call print_banner,Resetting selected environment runtime home, identity, state, and logs)
	$(call assert_supported_env,$(AOXC_ENV))
	@rm -rf "$(call runtime_home)"
	@rm -rf "$(call runtime_log_dir)"
	@echo "Environment runtime reset complete for: $(AOXC_ENV)"

env-show-active:
	$(call print_banner,Printing active environment markers)
	$(call assert_supported_env,$(AOXC_ENV))
	@if [ -f "$(call runtime_home)/$(ENV_RUNTIME_ACTIVE_FILE)" ]; then \
		echo "runtime-home active environment: $$(cat "$(call runtime_home)/$(ENV_RUNTIME_ACTIVE_FILE)")"; \
	else \
		echo "runtime-home active environment marker is absent"; \
	fi
	@if [ -f "$(call runtime_identity_dir)/$(ENV_RUNTIME_ACTIVE_FILE)" ]; then \
		echo "runtime-identity active environment: $$(cat "$(call runtime_identity_dir)/$(ENV_RUNTIME_ACTIVE_FILE)")"; \
	else \
		echo "runtime-identity active environment marker is absent"; \
	fi
	@if [ -f "$(AOXC_HOME)/$(ENV_RUNTIME_ACTIVE_FILE)" ]; then \
		echo "default-home active environment: $$(cat "$(AOXC_HOME)/$(ENV_RUNTIME_ACTIVE_FILE)")"; \
	else \
		echo "default-home active environment marker is absent"; \
	fi

env-install-mainnet:
	@$(MAKE) env-install AOXC_ENV=mainnet

env-install-testnet:
	@$(MAKE) env-install AOXC_ENV=testnet

env-install-devnet:
	@$(MAKE) env-install AOXC_ENV=devnet

env-install-localnet:
	@$(MAKE) env-install AOXC_ENV=localnet

env-verify-mainnet:
	@$(MAKE) env-verify AOXC_ENV=mainnet

env-verify-testnet:
	@$(MAKE) env-verify AOXC_ENV=testnet

env-verify-devnet:
	@$(MAKE) env-verify AOXC_ENV=devnet

env-verify-localnet:
	@$(MAKE) env-verify AOXC_ENV=localnet

env-activate-mainnet:
	@$(MAKE) env-activate AOXC_ENV=mainnet

env-activate-testnet:
	@$(MAKE) env-activate AOXC_ENV=testnet

env-activate-devnet:
	@$(MAKE) env-activate AOXC_ENV=devnet

env-activate-localnet:
	@$(MAKE) env-activate AOXC_ENV=localnet

env-bootstrap-mainnet:
	@$(MAKE) bootstrap-env-paths AOXC_ENV=mainnet
	@$(MAKE) env-activate AOXC_ENV=mainnet

env-bootstrap-testnet:
	@$(MAKE) bootstrap-env-paths AOXC_ENV=testnet
	@$(MAKE) env-activate AOXC_ENV=testnet

env-bootstrap-devnet:
	@$(MAKE) bootstrap-env-paths AOXC_ENV=devnet
	@$(MAKE) env-activate AOXC_ENV=devnet

env-bootstrap-localnet:
	@$(MAKE) bootstrap-env-paths AOXC_ENV=localnet
	@$(MAKE) env-activate AOXC_ENV=localnet
# --------------------------------------------------------------------
# Developer bootstrap and local helpers
# --------------------------------------------------------------------
dev-bootstrap:
	$(call print_banner,Suggested AOXC bootstrap flow)
	@printf "1) make env-check\n"
	@printf "2) make bootstrap-paths\n"
	@printf "3) make package-all-bin\n"
	@printf "4) make env-activate AOXC_ENV=%s\n" "$(AOXC_ENV)"
	@printf "5) make ops-doctor AOXC_ENV=%s\n" "$(AOXC_ENV)"
	@printf "6) make ops-auto-start AOXC_ENV=%s\n" "$(AOXC_ENV)"

run-local: package-bin bootstrap-paths
	$(call print_banner,Running local packaged node helper)
	@AOXC_HOME="$(AOXC_HOME_LOCAL)" ./scripts/run-local.sh

supervise-local: package-bin bootstrap-paths
	$(call print_banner,Running local supervisor helper)
	@AOXC_HOME="$(AOXC_HOME_LOCAL)" ./scripts/node_supervisor.sh

audit-install: package-all-bin
	$(call print_banner,Printing installed AOXC binaries)
	@ls -lah "$(AOXC_BIN_DIR)"

produce-loop:
	$(call print_banner,Running continuous producer helper)
	@AOXC_HOME="$(AOXC_HOME_LOCAL)" ./scripts/continuous_producer.sh

# --------------------------------------------------------------------
# Local real-chain loop
# --------------------------------------------------------------------
real-chain-prep: bootstrap-paths package-bin
	$(call print_banner,Preparing real-chain home and logs)
	@mkdir -p "$(AOXC_HOME_REAL)" "$(AOXC_REAL_LOG_DIR)"
	@echo "Prepared real-chain runtime home: $(AOXC_HOME_REAL)"

real-chain-run: real-chain-prep
	$(call print_banner,Running local real-chain daemon loop)
	@AOXC_HOME="$(AOXC_HOME_REAL)" AOXC_LOG_DIR="$(AOXC_REAL_LOG_DIR)" ./scripts/real_chain_daemon.sh run

real-chain-run-once: real-chain-prep
	$(call print_banner,Running one bounded local real-chain daemon cycle)
	@AOXC_HOME="$(AOXC_HOME_REAL)" AOXC_LOG_DIR="$(AOXC_REAL_LOG_DIR)" ./scripts/real_chain_daemon.sh once

real-chain-health:
	$(call print_banner,Probing local real-chain health)
	@AOXC_HOME="$(AOXC_HOME_REAL)" AOXC_LOG_DIR="$(AOXC_REAL_LOG_DIR)" ./scripts/real_chain_daemon.sh health

real-chain-tail:
	$(call print_banner,Tailing local real-chain logs)
	@AOXC_HOME="$(AOXC_HOME_REAL)" AOXC_LOG_DIR="$(AOXC_REAL_LOG_DIR)" ./scripts/real_chain_daemon.sh tail

# --------------------------------------------------------------------
# Network daemon orchestration
# --------------------------------------------------------------------
net-mainnet-start: package-bin env-activate-mainnet
	$(call print_banner,Bootstrapping and starting mainnet daemon)
	@AOXC_ENV=mainnet AOXC_HOME="$(AOXC_HOME_MAINNET)" AOXC_LOG_DIR="$(AOXC_MAINNET_LOG_DIR)" ./scripts/network_env_daemon.sh start

net-mainnet-once: package-bin env-activate-mainnet
	$(call print_banner,Running one mainnet daemon cycle)
	@AOXC_ENV=mainnet AOXC_HOME="$(AOXC_HOME_MAINNET)" AOXC_LOG_DIR="$(AOXC_MAINNET_LOG_DIR)" ./scripts/network_env_daemon.sh once

net-mainnet-status:
	$(call print_banner,Showing mainnet daemon status)
	@AOXC_ENV=mainnet AOXC_HOME="$(AOXC_HOME_MAINNET)" AOXC_LOG_DIR="$(AOXC_MAINNET_LOG_DIR)" ./scripts/network_env_daemon.sh status

net-mainnet-stop:
	$(call print_banner,Stopping mainnet daemon)
	@AOXC_ENV=mainnet AOXC_HOME="$(AOXC_HOME_MAINNET)" AOXC_LOG_DIR="$(AOXC_MAINNET_LOG_DIR)" ./scripts/network_env_daemon.sh stop

net-testnet-start: package-bin env-activate-testnet
	$(call print_banner,Bootstrapping and starting testnet daemon)
	@AOXC_ENV=testnet AOXC_HOME="$(AOXC_HOME_TESTNET)" AOXC_LOG_DIR="$(AOXC_TESTNET_LOG_DIR)" ./scripts/network_env_daemon.sh start

net-testnet-once: package-bin env-activate-testnet
	$(call print_banner,Running one testnet daemon cycle)
	@AOXC_ENV=testnet AOXC_HOME="$(AOXC_HOME_TESTNET)" AOXC_LOG_DIR="$(AOXC_TESTNET_LOG_DIR)" ./scripts/network_env_daemon.sh once

net-testnet-status:
	$(call print_banner,Showing testnet daemon status)
	@AOXC_ENV=testnet AOXC_HOME="$(AOXC_HOME_TESTNET)" AOXC_LOG_DIR="$(AOXC_TESTNET_LOG_DIR)" ./scripts/network_env_daemon.sh status

net-testnet-stop:
	$(call print_banner,Stopping testnet daemon)
	@AOXC_ENV=testnet AOXC_HOME="$(AOXC_HOME_TESTNET)" AOXC_LOG_DIR="$(AOXC_TESTNET_LOG_DIR)" ./scripts/network_env_daemon.sh stop

net-devnet-start: package-bin env-activate-devnet
	$(call print_banner,Bootstrapping and starting devnet daemon)
	@AOXC_ENV=devnet AOXC_HOME="$(AOXC_HOME_DEVNET)" AOXC_LOG_DIR="$(AOXC_DEVNET_LOG_DIR)" ./scripts/network_env_daemon.sh start

net-devnet-once: package-bin env-activate-devnet
	$(call print_banner,Running one devnet daemon cycle)
	@AOXC_ENV=devnet AOXC_HOME="$(AOXC_HOME_DEVNET)" AOXC_LOG_DIR="$(AOXC_DEVNET_LOG_DIR)" ./scripts/network_env_daemon.sh once

net-devnet-status:
	$(call print_banner,Showing devnet daemon status)
	@AOXC_ENV=devnet AOXC_HOME="$(AOXC_HOME_DEVNET)" AOXC_LOG_DIR="$(AOXC_DEVNET_LOG_DIR)" ./scripts/network_env_daemon.sh status

net-devnet-stop:
	$(call print_banner,Stopping devnet daemon)
	@AOXC_ENV=devnet AOXC_HOME="$(AOXC_HOME_DEVNET)" AOXC_LOG_DIR="$(AOXC_DEVNET_LOG_DIR)" ./scripts/network_env_daemon.sh stop

net-dual-start: package-bin env-activate-mainnet env-activate-testnet
	$(call print_banner,Starting dual stack: testnet and mainnet)
	@./scripts/network_stack.sh start-dual

net-dual-once: package-bin env-activate-mainnet env-activate-testnet
	$(call print_banner,Running one cycle on dual stack)
	@./scripts/network_stack.sh once-dual

net-dual-status:
	$(call print_banner,Showing dual stack status)
	@./scripts/network_stack.sh status-dual

net-dual-stop:
	$(call print_banner,Stopping dual stack)
	@./scripts/network_stack.sh stop-dual

net-dual-restart: net-dual-stop net-dual-start
	$(call print_banner,Dual stack restarted)

# --------------------------------------------------------------------
# Easy operator surfaces
# --------------------------------------------------------------------
ops-help:
	$(call print_banner,AOXC operator quick start)
	@printf "make ops-doctor AOXC_ENV=devnet\n"
	@printf "make ops-auto-start AOXC_ENV=devnet\n"
	@printf "make ops-status-devnet\n"
	@printf "make ops-logs-devnet\n"

ops-doctor:
	$(call print_banner,Running environment readiness checks)
	@$(MAKE) env-check
	@$(MAKE) bootstrap-paths
	@$(MAKE) env-doctor AOXC_ENV="$(AOXC_ENV)"

ops-auto-prepare:
	$(call print_banner,Preparing AOXC environment automatically)
	@$(MAKE) bootstrap-paths
	@$(MAKE) env-activate AOXC_ENV="$(AOXC_ENV)"
	@echo "Automatic environment preparation completed for: $(AOXC_ENV)"

ops-auto-bootstrap: ops-auto-prepare
	$(call print_banner,Starting selected AOXC environment automatically)
	@case "$(AOXC_ENV)" in \
		mainnet) $(MAKE) net-mainnet-start ;; \
		testnet) $(MAKE) net-testnet-start ;; \
		devnet) $(MAKE) net-devnet-start ;; \
		localnet) echo "No dedicated localnet daemon target exists in this Makefile."; exit 1 ;; \
		local-dev) echo "No dedicated local-dev daemon target exists in this Makefile."; exit 1 ;; \
		*) echo "Unsupported AOXC_ENV: $(AOXC_ENV)"; exit 1 ;; \
	esac

ops-start-mainnet:
	@$(MAKE) net-mainnet-start

ops-start-testnet:
	@$(MAKE) net-testnet-start

ops-start-devnet:
	@$(MAKE) net-devnet-start

ops-start-dual:
	@$(MAKE) net-dual-start

ops-auto-start: ops-auto-bootstrap

ops-auto-once:
	$(call print_banner,Running one cycle on the selected AOXC environment)
	@case "$(AOXC_ENV)" in \
		mainnet) $(MAKE) net-mainnet-once ;; \
		testnet) $(MAKE) net-testnet-once ;; \
		devnet) $(MAKE) net-devnet-once ;; \
		localnet) echo "No dedicated localnet daemon target exists in this Makefile."; exit 1 ;; \
		local-dev) echo "No dedicated local-dev daemon target exists in this Makefile."; exit 1 ;; \
		*) echo "Unsupported AOXC_ENV: $(AOXC_ENV)"; exit 1 ;; \
	esac

ops-stop-mainnet:
	@$(MAKE) net-mainnet-stop

ops-stop-testnet:
	@$(MAKE) net-testnet-stop

ops-stop-devnet:
	@$(MAKE) net-devnet-stop

ops-stop-dual:
	@$(MAKE) net-dual-stop

ops-status-mainnet:
	@$(MAKE) net-mainnet-status

ops-status-testnet:
	@$(MAKE) net-testnet-status

ops-status-devnet:
	@$(MAKE) net-devnet-status

ops-status-dual:
	@$(MAKE) net-dual-status

ops-restart-mainnet:
	@$(MAKE) net-mainnet-stop
	@$(MAKE) net-mainnet-start

ops-restart-testnet:
	@$(MAKE) net-testnet-stop
	@$(MAKE) net-testnet-start

ops-restart-devnet:
	@$(MAKE) net-devnet-stop
	@$(MAKE) net-devnet-start

ops-restart-dual:
	@$(MAKE) net-dual-restart

ops-logs-mainnet:
	$(call print_banner,Tailing mainnet logs)
	@AOXC_ENV=mainnet AOXC_HOME="$(AOXC_HOME_MAINNET)" AOXC_LOG_DIR="$(AOXC_MAINNET_LOG_DIR)" ./scripts/network_env_daemon.sh tail

ops-logs-testnet:
	$(call print_banner,Tailing testnet logs)
	@AOXC_ENV=testnet AOXC_HOME="$(AOXC_HOME_TESTNET)" AOXC_LOG_DIR="$(AOXC_TESTNET_LOG_DIR)" ./scripts/network_env_daemon.sh tail

ops-logs-devnet:
	$(call print_banner,Tailing devnet logs)
	@AOXC_ENV=devnet AOXC_HOME="$(AOXC_HOME_DEVNET)" AOXC_LOG_DIR="$(AOXC_DEVNET_LOG_DIR)" ./scripts/network_env_daemon.sh tail

ops-dashboard:
	$(call print_banner,AOXC multi-environment dashboard)
	@printf "\n[mainnet]\n"
	@$(MAKE) --no-print-directory net-mainnet-status || true
	@printf "\n[testnet]\n"
	@$(MAKE) --no-print-directory net-testnet-status || true
	@printf "\n[devnet]\n"
	@$(MAKE) --no-print-directory net-devnet-status || true

ops-flow-mainnet:
	$(call print_banner,Executing full automatic mainnet operational flow)
	@$(MAKE) env-check
	@$(MAKE) package-bin
	@$(MAKE) env-activate-mainnet
	@$(MAKE) db-event-sqlite AOXC_ENV=mainnet ACTION=flow STATUS=started DETAIL=ops-flow-mainnet
	@$(MAKE) net-mainnet-start
	@$(MAKE) db-event-sqlite AOXC_ENV=mainnet ACTION=flow STATUS=completed DETAIL=ops-flow-mainnet

ops-flow-testnet:
	$(call print_banner,Executing full automatic testnet operational flow)
	@$(MAKE) env-check
	@$(MAKE) package-bin
	@$(MAKE) env-activate-testnet
	@$(MAKE) db-event-sqlite AOXC_ENV=testnet ACTION=flow STATUS=started DETAIL=ops-flow-testnet
	@$(MAKE) net-testnet-start
	@$(MAKE) db-event-sqlite AOXC_ENV=testnet ACTION=flow STATUS=completed DETAIL=ops-flow-testnet

ops-flow-devnet:
	$(call print_banner,Executing full automatic devnet operational flow)
	@$(MAKE) env-check
	@$(MAKE) package-bin
	@$(MAKE) env-activate-devnet
	@$(MAKE) db-event-sqlite AOXC_ENV=devnet ACTION=flow STATUS=started DETAIL=ops-flow-devnet
	@$(MAKE) net-devnet-start
	@$(MAKE) db-event-sqlite AOXC_ENV=devnet ACTION=flow STATUS=completed DETAIL=ops-flow-devnet

ops-autonomy-blueprint:
	$(call print_banner,Printing autonomous system delivery blueprint)
	@printf "1. Validate operator environment\n"
	@printf "2. Bootstrap AOXC paths\n"
	@printf "3. Package binaries\n"
	@printf "4. Install and verify canonical environment identity\n"
	@printf "5. Sync selected identity into default runtime home\n"
	@printf "6. Start environment-scoped daemon loop\n"
	@printf "7. Record operational events into sqlite memory\n"
	@printf "8. Expose operator dashboard and restart controls\n"

# --------------------------------------------------------------------
# UI surfaces
# --------------------------------------------------------------------
ui-mainnet: build-release-all bootstrap-desktop-paths
	$(call print_banner,Running AOXCHub with mainnet profile)
	@AOXC_ENV=mainnet AOXC_HOME="$(AOXC_HOME_MAINNET)" "$(PWD)/target/release/aoxchub"

ui-testnet: build-release-all bootstrap-desktop-paths
	$(call print_banner,Running AOXCHub with testnet profile)
	@AOXC_ENV=testnet AOXC_HOME="$(AOXC_HOME_DESKTOP_TESTNET)" "$(PWD)/target/release/aoxchub"

ui-devnet: build-release-all bootstrap-desktop-paths
	$(call print_banner,Running AOXCHub with devnet label and testnet-safe defaults)
	@AOXC_ENV=devnet AOXC_HOME="$(AOXC_HOME_DESKTOP_TESTNET)" "$(PWD)/target/release/aoxchub"

alpha:
	$(call print_banner,AOXC alpha target)
	@echo "No alpha-specific workflow is defined beyond the standard operator and packaging surfaces."
