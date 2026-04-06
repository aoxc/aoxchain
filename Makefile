# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
#
# --------------------------------------------------------------------
# AOXC Makefile
# Single-runtime, portable, redb-first operator surface
# --------------------------------------------------------------------
# Architectural policy:
# - A single AOXC runtime root is used per host.
# - No environment fan-out exists in this Makefile.
# - Network/profile selection is intentionally delegated to repository
#   configuration and canonical runtime source material.
# - redb is treated as the canonical embedded database backend.
# - Python-based sqlite autonomy helpers are intentionally removed.
# - The Makefile provides one auditable operational surface for:
#     build, packaging, runtime installation, verification,
#     activation, daemon orchestration, and operator evidence.
#
# Support policy:
# - Linux: primary operator platform
# - macOS: best-effort support
# - Windows: GNU Make / Git Bash / MSYS2 / WSL-friendly use only
#
# Tooling expectations:
# - GNU Make
# - bash
# - cargo
# - git
# - sha256sum
# - tar

SHELL := /bin/bash
.DEFAULT_GOAL := help
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c

# --------------------------------------------------------------------
# Host platform detection
# --------------------------------------------------------------------
ifeq ($(OS),Windows_NT)
AOXC_PLATFORM := windows
else
UNAME_S := $(shell uname -s 2>/dev/null || echo unknown)
ifeq ($(UNAME_S),Linux)
AOXC_PLATFORM := linux
else ifeq ($(UNAME_S),Darwin)
AOXC_PLATFORM := macos
else
AOXC_PLATFORM := unknown
endif
endif

# --------------------------------------------------------------------
# Core tools
# --------------------------------------------------------------------
CARGO ?= cargo
RUSTFMT ?= rustfmt
BASH ?= bash
SHA256SUM ?= sha256sum
TAR ?= tar
DATE ?= date
MKDIR ?= mkdir
RM ?= rm
CP ?= cp
CAT ?= cat
FIND ?= find
TEE ?= tee
AWK ?= awk
LS ?= ls
SED ?= sed
CMP ?= cmp
CONTAINER_ENGINE ?= auto
CONTAINER_IMAGE ?= aoxchain-node:local
CONTAINER_COMPOSE_FILE ?= docker-compose.yaml

# --------------------------------------------------------------------
# Workspace quality flags
# --------------------------------------------------------------------
CLIPPY_FLAGS ?= --workspace --all-targets --all-features
TEST_FLAGS ?= --workspace
CHECK_FLAGS ?= --workspace

# --------------------------------------------------------------------
# Portable AOXC root resolution
# --------------------------------------------------------------------
ifndef AOXC_ROOT
ifeq ($(AOXC_PLATFORM),windows)
ifdef LOCALAPPDATA
AOXC_ROOT := $(LOCALAPPDATA)/AOXC
else ifdef APPDATA
AOXC_ROOT := $(APPDATA)/AOXC
else ifdef USERPROFILE
AOXC_ROOT := $(USERPROFILE)/AppData/Local/AOXC
else
$(error Unable to resolve AOXC_ROOT on Windows. Set AOXC_ROOT explicitly.)
endif
else
ifdef XDG_STATE_HOME
AOXC_ROOT := $(XDG_STATE_HOME)/aoxc
else ifdef XDG_DATA_HOME
AOXC_ROOT := $(XDG_DATA_HOME)/aoxc
else ifdef HOME
AOXC_ROOT := $(HOME)/.aoxc
else
$(error Unable to resolve AOXC_ROOT. Set AOXC_ROOT explicitly.)
endif
endif
endif

# --------------------------------------------------------------------
# Runtime policy
# --------------------------------------------------------------------
AOXC_DB_BACKEND ?= redb
AOXC_BOOTSTRAP_PROFILE ?= validation
AOXC_VALIDATOR_NAME ?= validator-01
AOXC_VALIDATOR_PASSWORD ?= ChangeMe#2026
AOXC_VALIDATOR_ID ?= aoxc-val-custom-001
AOXC_CONSENSUS_PUBLIC_KEY ?= replace_with_consensus_public_key_hex
AOXC_NETWORK_PUBLIC_KEY ?= replace_with_network_public_key_hex
AOXC_VALIDATOR_BALANCE ?= 50000000
AOXC_NEW_ACCOUNT_ID ?= AOXC_USER_0001
AOXC_NEW_ACCOUNT_BALANCE ?= 1000000
AOXC_NEW_ACCOUNT_ROLE ?= user

# --------------------------------------------------------------------
# AOXC-Q persistent testnet supervisor defaults
# --------------------------------------------------------------------
AOXC_Q_SCRIPT ?= ./scripts/aoxc-q-v0.2.0.start.sh
AOXC_Q_HOME ?= /mnt/xdbx/aoxc
AOXC_Q_ENV ?= testnet
AOXC_Q_PROFILE ?= testnet
AOXC_Q_MODE ?= local
AOXC_Q_NODES ?= 7
AOXC_Q_ROUNDS ?= 200
AOXC_Q_SLEEP_MIN_SECS ?= 1
AOXC_Q_SLEEP_MAX_SECS ?= 6
AOXC_Q_FORCE ?= 0

# --------------------------------------------------------------------
# Executable suffix
# --------------------------------------------------------------------
AOXC_EXE_SUFFIX :=
ifeq ($(AOXC_PLATFORM),windows)
AOXC_EXE_SUFFIX := .exe
endif

# --------------------------------------------------------------------
# Canonical single-runtime path contract
# --------------------------------------------------------------------
AOXC_BIN_ROOT ?= $(AOXC_ROOT)/bin
AOXC_BIN_CURRENT_DIR ?= $(AOXC_BIN_ROOT)/current
AOXC_BIN_VERSIONED_DIR ?= $(AOXC_BIN_ROOT)/versioned

AOXC_BIN_PATH ?= $(AOXC_BIN_CURRENT_DIR)/aoxc$(AOXC_EXE_SUFFIX)
AOXCHUB_BIN_PATH ?= $(AOXC_BIN_CURRENT_DIR)/aoxchub$(AOXC_EXE_SUFFIX)
AOXCKIT_BIN_PATH ?= $(AOXC_BIN_CURRENT_DIR)/aoxckit$(AOXC_EXE_SUFFIX)

AOXC_RELEASES_DIR ?= $(AOXC_ROOT)/releases
AOXC_LOG_ROOT ?= $(AOXC_ROOT)/logs
AOXC_LOG_DIR ?= $(AOXC_LOG_ROOT)
AOXC_RUNTIME_ROOT ?= $(AOXC_ROOT)/runtime
AOXC_RUNTIME_IDENTITY_DIR ?= $(AOXC_RUNTIME_ROOT)/identity
AOXC_RUNTIME_STATE_DIR ?= $(AOXC_RUNTIME_ROOT)/state
AOXC_RUNTIME_CONFIG_DIR ?= $(AOXC_RUNTIME_ROOT)/config
AOXC_RUNTIME_OPERATOR_DIR ?= $(AOXC_RUNTIME_ROOT)/operator
AOXC_RUNTIME_DB_DIR ?= $(AOXC_RUNTIME_ROOT)/db

AOXC_AUDIT_ROOT ?= $(AOXC_ROOT)/audit
AOXC_ARTIFACTS_ROOT ?= $(AOXC_ROOT)/artifacts
AOXC_CACHE_ROOT ?= $(AOXC_ROOT)/cache
AOXC_TMP_ROOT ?= $(AOXC_ROOT)/tmp
AOXC_ACTIVE_PROFILE_FILE ?= $(AOXC_ROOT)/active-profile

AOXC_DESKTOP_ROOT ?= $(AOXC_ROOT)/desktop
AOXC_DESKTOP_BIN_DIR ?= $(AOXC_DESKTOP_ROOT)/bin
AOXC_DESKTOP_LOG_DIR ?= $(AOXC_DESKTOP_ROOT)/logs
AOXC_DESKTOP_HOME ?= $(AOXC_DESKTOP_ROOT)/home

# --------------------------------------------------------------------
# Audit / evidence surfaces
# --------------------------------------------------------------------
AOXC_OPERATOR_EVENTS_FILE ?= $(AOXC_AUDIT_ROOT)/operator-events.jsonl
AOXC_RELEASE_EVENTS_FILE ?= $(AOXC_AUDIT_ROOT)/release-events.jsonl
AOXC_DB_STATUS_FILE ?= $(AOXC_AUDIT_ROOT)/db-status.latest.json
AOXC_RUNTIME_INSTALL_RECEIPT ?= $(AOXC_AUDIT_ROOT)/runtime-install.receipt
AOXC_RUNTIME_HEALTH_FILE ?= $(AOXC_AUDIT_ROOT)/runtime-health.latest.txt

# --------------------------------------------------------------------
# Canonical runtime source contract
# These files are expected to be maintained by the repository workflow.
# Single-system policy:
# - Runtime lifecycle remains one-path.
# - Network identity is selected by AOXC_NETWORK_KIND.
# --------------------------------------------------------------------
AOXC_NETWORK_KIND ?= mainnet
AOXC_RUNTIME_SOURCE_ROOT ?= configs/environments/$(AOXC_NETWORK_KIND)

SRC_MANIFEST_FILE ?= manifest.v1.json
SRC_GENESIS_FILE ?= genesis.v1.json
SRC_GENESIS_SHA256_FILE ?= genesis.v1.sha256
SRC_VALIDATORS_FILE ?= validators.json
SRC_BOOTNODES_FILE ?= bootnodes.json
SRC_CERTIFICATE_FILE ?= certificate.json
SRC_PROFILE_FILE ?= profile.toml
SRC_RELEASE_POLICY_FILE ?= release-policy.toml

RUNTIME_MANIFEST_FILE ?= manifest.json
RUNTIME_GENESIS_FILE ?= genesis.json
RUNTIME_GENESIS_SHA256_FILE ?= genesis.sha256
RUNTIME_VALIDATORS_FILE ?= validators.json
RUNTIME_BOOTNODES_FILE ?= bootnodes.json
RUNTIME_CERTIFICATE_FILE ?= certificate.json
RUNTIME_PROFILE_FILE ?= profile.toml
RUNTIME_RELEASE_POLICY_FILE ?= release-policy.toml
RUNTIME_FINGERPRINT_FILE ?= genesis.fingerprint.sha256

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
RELEASE_ARCHIVE_BASENAME ?= $(RELEASE_BUNDLE_NAME)-portable
RELEASE_ARCHIVE_PATH ?= $(AOXC_RELEASES_DIR)/$(RELEASE_ARCHIVE_BASENAME).tar.gz
RELEASE_BINARIES ?= aoxc aoxchub aoxckit

AOXC_VERSIONED_BIN_PATH ?= $(AOXC_BIN_VERSIONED_DIR)/aoxc-$(RELEASE_TAG)$(AOXC_EXE_SUFFIX)
AOXCHUB_VERSIONED_BIN_PATH ?= $(AOXC_BIN_VERSIONED_DIR)/aoxchub-$(RELEASE_TAG)$(AOXC_EXE_SUFFIX)
AOXCKIT_VERSIONED_BIN_PATH ?= $(AOXC_BIN_VERSIONED_DIR)/aoxckit-$(RELEASE_TAG)$(AOXC_EXE_SUFFIX)

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
	@$(MKDIR) -p "$(1)"
endef

define require_command
	@command -v "$(1)" >/dev/null 2>&1 || { echo "Missing required command: $(1)"; exit 1; }
endef

define iso_utc_now
$$(TZ=UTC $(DATE) +%Y-%m-%dT%H:%M:%SZ)
endef

# --------------------------------------------------------------------
# Modular make surfaces
# --------------------------------------------------------------------
-include $(MK_DIR)/container.mk

# --------------------------------------------------------------------
# Phony targets
# --------------------------------------------------------------------
.PHONY: \
	help paths env-check bootstrap-paths bootstrap-desktop-paths \
	clean-root clean-logs clean-runtime clean-bin clean-audit \
	build build-release build-release-all build-release-matrix \
	package-bin package-all-bin package-versioned-bin package-versioned-archive publish-release \
	release-binary-list install-bin package-desktop repo-release-prepare repo-release-validate \
	test test-lib test-workspace test-inventory check fmt clippy audit code-size-gate quality quality-quick quality-release ci \
	db-init db-status db-event db-release db-history db-health \
	version manifest policy \
	runtime-print runtime-refresh-genesis-sha256 runtime-source-check runtime-install runtime-verify runtime-activate runtime-status runtime-fingerprint runtime-doctor runtime-reinstall runtime-reset runtime-show-active \
	runtime-bundle-compat-check docker-check podman-check container-check container-build container-config container-up container-down production-full \
	phase1-full quantum-readiness-gate quantum-full \
	aoxc-full-4nodes aoxc-full-4nodes-docker \
	ops-help ops-doctor ops-prepare ops-start ops-once ops-stop ops-status ops-restart ops-logs ops-flow \
	demo localnet devnet testnet testnet-gate testnet-readiness-gate aoxcvm-phase3-gate aoxcvm-production-closure-gate reset doctor audit-chain logs down restart \
	network-create network-start network-stop network-status genesis-build chain-status \
	aoxc-q-up aoxc-q-provision aoxc-q-start aoxc-q-stop aoxc-q-restart aoxc-q-status \
	chain-help chain-init chain-add-account chain-add-validator chain-start-persistent \
	ui alpha

# --------------------------------------------------------------------
# Help / diagnostics
# --------------------------------------------------------------------
help:
	@printf "\nAOXC single-runtime operator targets\n\n"
	@printf "Portable root policy\n"
	@printf "  AOXC_PLATFORM            : %s\n" "$(AOXC_PLATFORM)"
	@printf "  AOXC_ROOT                : %s\n" "$(AOXC_ROOT)"
	@printf "  AOXC_BIN_PATH            : %s\n" "$(AOXC_BIN_PATH)"
	@printf "  AOXC_RUNTIME_ROOT        : %s\n" "$(AOXC_RUNTIME_ROOT)"
	@printf "  AOXC_NETWORK_KIND        : %s\n" "$(AOXC_NETWORK_KIND)"
	@printf "  AOXC_RUNTIME_SOURCE_ROOT : %s\n\n" "$(AOXC_RUNTIME_SOURCE_ROOT)"

	@printf "Build and quality\n"
	@printf "  make build\n"
	@printf "  make build-release\n"
	@printf "  make build-release-all\n"
	@printf "  make fmt\n"
	@printf "  make check\n"
	@printf "  make test\n"
	@printf "  make test-inventory\n"
	@printf "  make clippy\n"
	@printf "  make audit\n"
	@printf "  make code-size-gate\n"
	@printf "  make repo-hygiene-gate\n"
	@printf "  make quality\n\n"
	@printf "  make production-full\n\n"
	@printf "  make phase1-full\n\n"
	@printf "  make quantum-readiness-gate\n"
	@printf "  make aoxcvm-production-closure-gate\n"
	@printf "  make quantum-full\n\n"

	@printf "Quick operator workflows\n"
	@printf "  make demo\n"
	@printf "  make localnet\n"
	@printf "  make devnet\n"
	@printf "  make testnet\n"
	@printf "  make testnet-gate\n"
	@printf "  make testnet-readiness-gate\n"
	@printf "  make aoxc-q-up AOXC_Q_MODE=<local|public> AOXC_Q_NODES=<n> AOXC_Q_FORCE=1\n"
	@printf "  make aoxc-q-status AOXC_Q_MODE=<local|public> AOXC_Q_NODES=<n>\n"
	@printf "  make aoxc-q-stop AOXC_Q_MODE=<local|public> AOXC_Q_NODES=<n>\n"
	@printf "  make doctor\n"
	@printf "  make audit-chain\n"
	@printf "  make reset\n\n"

	@printf "Packaging\n"
	@printf "  make package-bin\n"
	@printf "  make package-all-bin\n"
	@printf "  make package-versioned-bin\n"
	@printf "  make package-versioned-archive\n"
	@printf "  make publish-release\n\n"
	@printf "  make repo-release-prepare\n"
	@printf "  make repo-release-validate\n\n"

	@printf "Runtime lifecycle\n"
	@printf "  make runtime-print\n"
	@printf "  make runtime-source-check\n"
	@printf "  make runtime-bundle-compat-check\n"
	@printf "  make runtime-install\n"
	@printf "  make runtime-verify\n"
	@printf "  make runtime-activate\n"
	@printf "  make runtime-status\n"
	@printf "  make runtime-fingerprint\n"
	@printf "  make runtime-doctor\n"
	@printf "  make runtime-reinstall\n"
	@printf "  make runtime-reset\n\n"
	@printf "Container runtime\n"
	@printf "  make container-check CONTAINER_ENGINE=docker\n"
	@printf "  make container-check CONTAINER_ENGINE=podman\n"
	@printf "  make container-build\n"
	@printf "  make container-config\n"
	@printf "  make container-up\n"
	@printf "  make container-down\n\n"
	@printf "Full multi-node provision\n"
	@printf "  make aoxc-full-4nodes\n"
	@printf "  make aoxc-full-4nodes-docker\n\n"

	@printf "Database and audit\n"
	@printf "  make db-init\n"
	@printf "  make db-status\n"
	@printf "  make db-event\n"
	@printf "  make db-release\n"
	@printf "  make db-history\n"
	@printf "  make db-health\n\n"

	@printf "Operations\n"
	@printf "  make ops-help\n"
	@printf "  make ops-doctor\n"
	@printf "  make ops-prepare\n"
	@printf "  make ops-start\n"
	@printf "  make ops-once\n"
	@printf "  make ops-stop\n"
	@printf "  make ops-status\n"
	@printf "  make ops-restart\n"
	@printf "  make ops-logs\n"
	@printf "  make ops-flow\n\n"

	@printf "Persistent chain bootstrap\n"
	@printf "  make chain-help\n"
	@printf "  make chain-init AOXC_BOOTSTRAP_PROFILE=validation AOXC_VALIDATOR_NAME=validator-01 AOXC_VALIDATOR_PASSWORD='StrongPass#2026'\n"
	@printf "  make chain-add-account AOXC_NEW_ACCOUNT_ID=AOXC_USER_0001 AOXC_NEW_ACCOUNT_BALANCE=1000000 AOXC_NEW_ACCOUNT_ROLE=user\n"
	@printf "  make chain-add-validator AOXC_VALIDATOR_ID=aoxc-val-custom-001 AOXC_CONSENSUS_PUBLIC_KEY=<hex> AOXC_NETWORK_PUBLIC_KEY=<hex> AOXC_VALIDATOR_BALANCE=50000000\n"
	@printf "  make chain-start-persistent\n\n"

paths:
	@printf "AOXC_PLATFORM=%s\n" "$(AOXC_PLATFORM)"
	@printf "AOXC_ROOT=%s\n" "$(AOXC_ROOT)"
	@printf "AOXC_DB_BACKEND=%s\n" "$(AOXC_DB_BACKEND)"
	@printf "AOXC_BIN_ROOT=%s\n" "$(AOXC_BIN_ROOT)"
	@printf "AOXC_BIN_CURRENT_DIR=%s\n" "$(AOXC_BIN_CURRENT_DIR)"
	@printf "AOXC_BIN_VERSIONED_DIR=%s\n" "$(AOXC_BIN_VERSIONED_DIR)"
	@printf "AOXC_BIN_PATH=%s\n" "$(AOXC_BIN_PATH)"
	@printf "AOXCHUB_BIN_PATH=%s\n" "$(AOXCHUB_BIN_PATH)"
	@printf "AOXCKIT_BIN_PATH=%s\n" "$(AOXCKIT_BIN_PATH)"
	@printf "AOXC_RELEASES_DIR=%s\n" "$(AOXC_RELEASES_DIR)"
	@printf "AOXC_LOG_ROOT=%s\n" "$(AOXC_LOG_ROOT)"
	@printf "AOXC_RUNTIME_ROOT=%s\n" "$(AOXC_RUNTIME_ROOT)"
	@printf "AOXC_RUNTIME_IDENTITY_DIR=%s\n" "$(AOXC_RUNTIME_IDENTITY_DIR)"
	@printf "AOXC_RUNTIME_CONFIG_DIR=%s\n" "$(AOXC_RUNTIME_CONFIG_DIR)"
	@printf "AOXC_RUNTIME_STATE_DIR=%s\n" "$(AOXC_RUNTIME_STATE_DIR)"
	@printf "AOXC_RUNTIME_OPERATOR_DIR=%s\n" "$(AOXC_RUNTIME_OPERATOR_DIR)"
	@printf "AOXC_RUNTIME_DB_DIR=%s\n" "$(AOXC_RUNTIME_DB_DIR)"
	@printf "AOXC_AUDIT_ROOT=%s\n" "$(AOXC_AUDIT_ROOT)"
	@printf "AOXC_ARTIFACTS_ROOT=%s\n" "$(AOXC_ARTIFACTS_ROOT)"
	@printf "AOXC_CACHE_ROOT=%s\n" "$(AOXC_CACHE_ROOT)"
	@printf "AOXC_TMP_ROOT=%s\n" "$(AOXC_TMP_ROOT)"
	@printf "AOXC_ACTIVE_PROFILE_FILE=%s\n" "$(AOXC_ACTIVE_PROFILE_FILE)"
	@printf "AOXC_NETWORK_KIND=%s\n" "$(AOXC_NETWORK_KIND)"
	@printf "AOXC_RUNTIME_SOURCE_ROOT=%s\n" "$(AOXC_RUNTIME_SOURCE_ROOT)"
	@printf "RELEASE_TAG=%s\n" "$(RELEASE_TAG)"
	@printf "RELEASE_ARCHIVE_PATH=%s\n" "$(RELEASE_ARCHIVE_PATH)"

# --------------------------------------------------------------------
# Environment / tooling checks
# --------------------------------------------------------------------
env-check:
	$(call print_banner,Validating local operator environment)
	@command -v $(CARGO) >/dev/null 2>&1 || { echo "cargo not found"; exit 1; }
	@command -v git >/dev/null 2>&1 || { echo "git not found"; exit 1; }
	@command -v bash >/dev/null 2>&1 || { echo "bash not found"; exit 1; }
	@command -v sha256sum >/dev/null 2>&1 || { echo "sha256sum not found"; exit 1; }
	$(call require_file,./scripts/quality_gate.sh)
	$(call require_file,./scripts/run_runtime.sh)
	$(call require_file,./scripts/runtime_daemon.sh)
	$(call require_file,./scripts/release/generate_release_evidence.sh)
	$(call require_file,./scripts/release_artifact_certify.sh)
	$(call require_file,./scripts/READ.md)
	@echo "Environment check passed."

# --------------------------------------------------------------------
# Path bootstrap and cleanup
# --------------------------------------------------------------------
bootstrap-paths:
	$(call print_banner,Creating canonical AOXC directories)
	$(call ensure_dir,$(AOXC_ROOT))
	$(call ensure_dir,$(AOXC_BIN_ROOT))
	$(call ensure_dir,$(AOXC_BIN_CURRENT_DIR))
	$(call ensure_dir,$(AOXC_BIN_VERSIONED_DIR))
	$(call ensure_dir,$(AOXC_RELEASES_DIR))
	$(call ensure_dir,$(AOXC_LOG_ROOT))
	$(call ensure_dir,$(AOXC_RUNTIME_ROOT))
	$(call ensure_dir,$(AOXC_RUNTIME_IDENTITY_DIR))
	$(call ensure_dir,$(AOXC_RUNTIME_CONFIG_DIR))
	$(call ensure_dir,$(AOXC_RUNTIME_STATE_DIR))
	$(call ensure_dir,$(AOXC_RUNTIME_OPERATOR_DIR))
	$(call ensure_dir,$(AOXC_RUNTIME_DB_DIR))
	$(call ensure_dir,$(AOXC_AUDIT_ROOT))
	$(call ensure_dir,$(AOXC_ARTIFACTS_ROOT))
	$(call ensure_dir,$(AOXC_CACHE_ROOT))
	$(call ensure_dir,$(AOXC_TMP_ROOT))
	@echo "AOXC path bootstrap complete."

bootstrap-desktop-paths:
	$(call print_banner,Creating AOXC desktop directories)
	$(call ensure_dir,$(AOXC_DESKTOP_ROOT))
	$(call ensure_dir,$(AOXC_DESKTOP_HOME))
	$(call ensure_dir,$(AOXC_DESKTOP_BIN_DIR))
	$(call ensure_dir,$(AOXC_DESKTOP_LOG_DIR))
	@echo "AOXC desktop path bootstrap complete."

clean-root:
	$(call print_banner,Removing AOXC root)
	@$(RM) -rf "$(AOXC_ROOT)"
	@echo "Removed: $(AOXC_ROOT)"

clean-logs:
	$(call print_banner,Removing AOXC logs)
	@$(RM) -rf "$(AOXC_LOG_ROOT)"
	@echo "Removed: $(AOXC_LOG_ROOT)"

clean-runtime:
	$(call print_banner,Removing AOXC runtime root)
	@$(RM) -rf "$(AOXC_RUNTIME_ROOT)"
	@echo "Removed: $(AOXC_RUNTIME_ROOT)"

clean-bin:
	$(call print_banner,Removing AOXC bin root)
	@$(RM) -rf "$(AOXC_BIN_ROOT)"
	@echo "Removed: $(AOXC_BIN_ROOT)"

clean-audit:
	$(call print_banner,Removing AOXC audit root)
	@$(RM) -rf "$(AOXC_AUDIT_ROOT)"
	@echo "Removed: $(AOXC_AUDIT_ROOT)"

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

build-release-matrix: build-release-all
	$(call print_banner,Completed release matrix build)

package-bin: build-release bootstrap-paths
	$(call print_banner,Installing release AOXC CLI into current bin directory)
	@test -f "target/release/aoxc$(AOXC_EXE_SUFFIX)" || { echo "Missing release binary: target/release/aoxc$(AOXC_EXE_SUFFIX)"; exit 1; }
	@$(CP) "target/release/aoxc$(AOXC_EXE_SUFFIX)" "$(AOXC_BIN_PATH)"
	@chmod +x "$(AOXC_BIN_PATH)" 2>/dev/null || true
	@echo "Installed current AOXC CLI: $(AOXC_BIN_PATH)"

release-binary-list:
	$(call print_banner,Printing configured release binary names)
	@printf "%s\n" $(RELEASE_BINARIES)

package-all-bin: build-release-all bootstrap-paths
	$(call print_banner,Installing all release AOXC binaries into current bin directory)
	@for bin in $(RELEASE_BINARIES); do \
		test -f "target/release/$$bin$(AOXC_EXE_SUFFIX)" || { echo "Missing built binary: target/release/$$bin$(AOXC_EXE_SUFFIX)"; exit 1; }; \
		$(CP) "target/release/$$bin$(AOXC_EXE_SUFFIX)" "$(AOXC_BIN_CURRENT_DIR)/$$bin$(AOXC_EXE_SUFFIX)"; \
		chmod +x "$(AOXC_BIN_CURRENT_DIR)/$$bin$(AOXC_EXE_SUFFIX)" 2>/dev/null || true; \
	done
	@echo "Installed release binaries into: $(AOXC_BIN_CURRENT_DIR)"

package-versioned-bin: build-release-all bootstrap-paths
	$(call print_banner,Installing release binaries into versioned bundle)
	@$(MKDIR) -p "$(RELEASE_BUNDLE_BIN_DIR)"
	@for bin in $(RELEASE_BINARIES); do \
		test -f "target/release/$$bin$(AOXC_EXE_SUFFIX)" || { echo "Missing built binary: target/release/$$bin$(AOXC_EXE_SUFFIX)"; exit 1; }; \
		$(CP) "target/release/$$bin$(AOXC_EXE_SUFFIX)" "$(RELEASE_BUNDLE_BIN_DIR)/$$bin$(AOXC_EXE_SUFFIX)"; \
		chmod +x "$(RELEASE_BUNDLE_BIN_DIR)/$$bin$(AOXC_EXE_SUFFIX)" 2>/dev/null || true; \
	done
	@$(MAKE) --no-print-directory manifest > "$(RELEASE_BUNDLE_MANIFEST)"
	@cd "$(RELEASE_BUNDLE_DIR)" && $(SHA256SUM) bin/* > "$(RELEASE_BUNDLE_CHECKSUMS)"
	@$(CP) "target/release/aoxc$(AOXC_EXE_SUFFIX)" "$(AOXC_VERSIONED_BIN_PATH)"
	@chmod +x "$(AOXC_VERSIONED_BIN_PATH)" 2>/dev/null || true
	@if [ -f "target/release/aoxchub$(AOXC_EXE_SUFFIX)" ]; then $(CP) "target/release/aoxchub$(AOXC_EXE_SUFFIX)" "$(AOXCHUB_VERSIONED_BIN_PATH)"; chmod +x "$(AOXCHUB_VERSIONED_BIN_PATH)" 2>/dev/null || true; fi
	@if [ -f "target/release/aoxckit$(AOXC_EXE_SUFFIX)" ]; then $(CP) "target/release/aoxckit$(AOXC_EXE_SUFFIX)" "$(AOXCKIT_VERSIONED_BIN_PATH)"; chmod +x "$(AOXCKIT_VERSIONED_BIN_PATH)" 2>/dev/null || true; fi
	@echo "Versioned release bundle created at: $(RELEASE_BUNDLE_DIR)"

package-versioned-archive: package-versioned-bin
	$(call print_banner,Creating versioned release archive)
	@$(MKDIR) -p "$(AOXC_RELEASES_DIR)"
	@cd "$(AOXC_RELEASES_DIR)" && $(TAR) -czf "$(notdir $(RELEASE_ARCHIVE_PATH))" "$(RELEASE_BUNDLE_NAME)"
	@echo "Archive created at: $(RELEASE_ARCHIVE_PATH)"

publish-release: package-versioned-archive db-release
	$(call print_banner,Release publication evidence completed)
	@echo "Release archive: $(RELEASE_ARCHIVE_PATH)"

install-bin: package-bin

package-desktop: build-release-all bootstrap-desktop-paths
	$(call print_banner,Packaging desktop binaries)
	@for bin in $(RELEASE_BINARIES); do \
		test -f "target/release/$$bin$(AOXC_EXE_SUFFIX)" || { echo "Missing built binary: target/release/$$bin$(AOXC_EXE_SUFFIX)"; exit 1; }; \
		$(CP) "target/release/$$bin$(AOXC_EXE_SUFFIX)" "$(AOXC_DESKTOP_BIN_DIR)/$$bin$(AOXC_EXE_SUFFIX)"; \
		chmod +x "$(AOXC_DESKTOP_BIN_DIR)/$$bin$(AOXC_EXE_SUFFIX)" 2>/dev/null || true; \
	done
	@echo "Desktop binaries installed under: $(AOXC_DESKTOP_BIN_DIR)"

repo-release-prepare: build-release
	$(call print_banner,Preparing repository release directory under ./releases)
	@python3 scripts/release/prepare_repo_release.py --binary target/release/aoxc --network mainnet

repo-release-validate:
	$(call print_banner,Validating repository release directory under ./releases)
	@python3 scripts/release/validate_repo_release.py "releases/v$(RELEASE_VERSION)"

test:
	$(call print_banner,Running workspace tests)
	$(CARGO) test $(TEST_FLAGS)

test-lib:
	$(call print_banner,Running library tests)
	$(CARGO) test --workspace --lib

test-workspace: test

test-inventory:
	$(call print_banner,Generating test inventory artifacts)
	python3 scripts/testing/generate_test_inventory.py

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

code-size-gate:
	$(call print_banner,Running code file length gate)
	@./scripts/validation/code_file_length_gate.sh

repo-hygiene-gate:
	$(call print_banner,Running repository hygiene gate)
	@./scripts/validation/repository_hygiene_gate.sh

code-size-gate-full:
	$(call print_banner,Running full repository code file length gate)
	@./scripts/validation/code_file_length_gate.sh --all

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
# Database / audit surfaces
# --------------------------------------------------------------------
db-init: bootstrap-paths
	$(call print_banner,Initializing AOXC runtime database using canonical redb backend)
	@test "$(AOXC_DB_BACKEND)" = "redb" || { echo "Unsupported AOXC_DB_BACKEND: $(AOXC_DB_BACKEND). Expected redb."; exit 1; }
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" $(CARGO) run -p aoxcmd -- db-init --backend redb --format json | $(TEE) "$(AOXC_DB_STATUS_FILE)" >/dev/null
	@echo "Initialized runtime database at: $(AOXC_RUNTIME_DB_DIR)"
	@echo "Status receipt: $(AOXC_DB_STATUS_FILE)"

db-status: bootstrap-paths
	$(call print_banner,Printing AOXC runtime database status)
	@test "$(AOXC_DB_BACKEND)" = "redb" || { echo "Unsupported AOXC_DB_BACKEND: $(AOXC_DB_BACKEND). Expected redb."; exit 1; }
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" $(CARGO) run -p aoxcmd -- db-status --backend redb --format json | $(TEE) "$(AOXC_DB_STATUS_FILE)"
	@echo "Persisted db status receipt at: $(AOXC_DB_STATUS_FILE)"

db-event: bootstrap-paths
	$(call print_banner,Recording operator event)
	@TS_VALUE="$(call iso_utc_now)"; \
	ACTION_VALUE="$${ACTION:-heartbeat}"; \
	STATUS_VALUE="$${STATUS:-ok}"; \
	DETAIL_VALUE="$${DETAIL:-make-db-event}"; \
	printf '{"timestamp_utc":"%s","backend":"redb","action":"%s","status":"%s","detail":"%s"}\n' \
		"$$TS_VALUE" "$$ACTION_VALUE" "$$STATUS_VALUE" "$$DETAIL_VALUE" >> "$(AOXC_OPERATOR_EVENTS_FILE)"
	@echo "Recorded operator event in: $(AOXC_OPERATOR_EVENTS_FILE)"

db-release: bootstrap-paths
	$(call print_banner,Recording release publication evidence)
	@TS_VALUE="$(call iso_utc_now)"; \
	printf '{"timestamp_utc":"%s","backend":"redb","release_tag":"%s","artifact":"%s"}\n' \
		"$$TS_VALUE" "$(RELEASE_TAG)" "$(RELEASE_ARCHIVE_PATH)" >> "$(AOXC_RELEASE_EVENTS_FILE)"
	@echo "Recorded release evidence in: $(AOXC_RELEASE_EVENTS_FILE)"

db-history: bootstrap-paths
	$(call print_banner,Recent operator event history)
	@LIMIT_VALUE="$${LIMIT:-30}"; \
	if [ ! -f "$(AOXC_OPERATOR_EVENTS_FILE)" ]; then \
		echo "No operator event history exists yet."; \
		exit 0; \
	fi; \
	tail -n "$$LIMIT_VALUE" "$(AOXC_OPERATOR_EVENTS_FILE)"

db-health: bootstrap-paths
	$(call print_banner,Producing runtime database health receipt)
	@TS_VALUE="$(call iso_utc_now)"; \
	{ \
		echo "timestamp_utc=$$TS_VALUE"; \
		echo "backend=redb"; \
		echo "aoxc_root=$(AOXC_ROOT)"; \
		echo "runtime_root=$(AOXC_RUNTIME_ROOT)"; \
		echo "runtime_db_dir=$(AOXC_RUNTIME_DB_DIR)"; \
		echo "db_status_file=$(AOXC_DB_STATUS_FILE)"; \
		echo "operator_events_file=$(AOXC_OPERATOR_EVENTS_FILE)"; \
		echo "release_events_file=$(AOXC_RELEASE_EVENTS_FILE)"; \
		if [ -f "$(AOXC_DB_STATUS_FILE)" ]; then echo "db_status_present=yes"; else echo "db_status_present=no"; fi; \
		if [ -f "$(AOXC_OPERATOR_EVENTS_FILE)" ]; then echo "operator_events_present=yes"; else echo "operator_events_present=no"; fi; \
		if [ -f "$(AOXC_RELEASE_EVENTS_FILE)" ]; then echo "release_events_present=yes"; else echo "release_events_present=no"; fi; \
	} > "$(AOXC_RUNTIME_HEALTH_FILE)"
	@$(CAT) "$(AOXC_RUNTIME_HEALTH_FILE)"

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
# Runtime lifecycle
# --------------------------------------------------------------------
runtime-print:
	$(call print_banner,Printing resolved runtime paths)
	@printf "AOXC_ROOT=%s\n" "$(AOXC_ROOT)"
	@printf "AOXC_NETWORK_KIND=%s\n" "$(AOXC_NETWORK_KIND)"
	@printf "AOXC_RUNTIME_SOURCE_ROOT=%s\n" "$(AOXC_RUNTIME_SOURCE_ROOT)"
	@printf "AOXC_RUNTIME_ROOT=%s\n" "$(AOXC_RUNTIME_ROOT)"
	@printf "AOXC_RUNTIME_IDENTITY_DIR=%s\n" "$(AOXC_RUNTIME_IDENTITY_DIR)"
	@printf "AOXC_RUNTIME_CONFIG_DIR=%s\n" "$(AOXC_RUNTIME_CONFIG_DIR)"
	@printf "AOXC_RUNTIME_STATE_DIR=%s\n" "$(AOXC_RUNTIME_STATE_DIR)"
	@printf "AOXC_RUNTIME_OPERATOR_DIR=%s\n" "$(AOXC_RUNTIME_OPERATOR_DIR)"
	@printf "AOXC_RUNTIME_DB_DIR=%s\n" "$(AOXC_RUNTIME_DB_DIR)"
	@printf "AOXC_LOG_DIR=%s\n" "$(AOXC_LOG_DIR)"
	@printf "AOXC_ACTIVE_PROFILE_FILE=%s\n" "$(AOXC_ACTIVE_PROFILE_FILE)"
	@printf "SRC_MANIFEST=%s\n" "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_MANIFEST_FILE)"
	@printf "SRC_GENESIS=%s\n" "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_GENESIS_FILE)"
	@printf "SRC_PROFILE=%s\n" "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_PROFILE_FILE)"

runtime-refresh-genesis-sha256:
	$(call print_banner,Refreshing canonical genesis digest sidecar)
	$(call require_command,sha256sum)
	$(call require_dir,$(AOXC_RUNTIME_SOURCE_ROOT))
	$(call require_file,$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_GENESIS_FILE))
	@cd "$(AOXC_RUNTIME_SOURCE_ROOT)" && $(SHA256SUM) "$(SRC_GENESIS_FILE)" > "$(SRC_GENESIS_SHA256_FILE)"
	@echo "Refreshed: $(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_GENESIS_SHA256_FILE)"

runtime-source-check:
	$(call print_banner,Validating canonical runtime source bundle)
	$(call require_command,sha256sum)
	$(call require_dir,$(AOXC_RUNTIME_SOURCE_ROOT))
	$(call require_file,$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_MANIFEST_FILE))
	$(call require_file,$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_GENESIS_FILE))
	$(call require_file,$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_VALIDATORS_FILE))
	$(call require_file,$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_BOOTNODES_FILE))
	$(call require_file,$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_CERTIFICATE_FILE))
	$(call require_file,$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_PROFILE_FILE))
	$(call require_file,$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_RELEASE_POLICY_FILE))
	@$(MAKE) --no-print-directory runtime-refresh-genesis-sha256
	@cd "$(AOXC_RUNTIME_SOURCE_ROOT)" && $(SHA256SUM) -c "$(SRC_GENESIS_SHA256_FILE)"
	@echo "Canonical runtime source bundle is valid."

runtime-bundle-compat-check:
	$(call print_banner,Validating full environment bundle compatibility)
	@python3 scripts/validate_environment_bundle.py

docker-check:
	$(call print_banner,Validating Docker runtime prerequisites)
	$(call require_command,docker)
	@docker info >/dev/null 2>&1 || { echo "Docker daemon is not reachable. Start Docker and retry."; exit 1; }
	@if docker compose version >/dev/null 2>&1; then \
		echo "Docker Compose v2 is available."; \
	elif command -v docker-compose >/dev/null 2>&1; then \
		echo "docker-compose is available."; \
	else \
		echo "Missing Docker Compose support (docker compose / docker-compose)."; \
		exit 1; \
	fi

podman-check:
	$(call print_banner,Validating Podman runtime prerequisites)
	$(call require_command,podman)
	@podman info >/dev/null 2>&1 || { echo "Podman runtime is not reachable."; exit 1; }
	@podman compose version >/dev/null 2>&1 || { echo "Missing Podman Compose support (podman compose)."; exit 1; }

container-check:
	$(call print_banner,Validating container runtime prerequisites for $(CONTAINER_ENGINE))
	@if [ "$(CONTAINER_ENGINE)" = "docker" ]; then \
		$(MAKE) --no-print-directory docker-check; \
	elif [ "$(CONTAINER_ENGINE)" = "podman" ]; then \
		$(MAKE) --no-print-directory podman-check; \
	else \
		echo "Unsupported CONTAINER_ENGINE=$(CONTAINER_ENGINE). Use docker or podman."; \
		exit 1; \
	fi

container-build: container-check
	$(call print_banner,Building local AOXChain container image with $(CONTAINER_ENGINE))
	@"$(CONTAINER_ENGINE)" build -t "$(CONTAINER_IMAGE)" .

container-config: container-check
	$(call print_banner,Rendering compose configuration with $(CONTAINER_ENGINE))
	@"$(CONTAINER_ENGINE)" compose -f "$(CONTAINER_COMPOSE_FILE)" config

container-up: container-check
	$(call print_banner,Starting AOXChain compose topology with $(CONTAINER_ENGINE))
	@"$(CONTAINER_ENGINE)" compose -f "$(CONTAINER_COMPOSE_FILE)" up --build

container-down: container-check
	$(call print_banner,Stopping AOXChain compose topology with $(CONTAINER_ENGINE))
	@"$(CONTAINER_ENGINE)" compose -f "$(CONTAINER_COMPOSE_FILE)" down

runtime-install: runtime-source-check bootstrap-paths
	$(call print_banner,Installing canonical runtime bundle into AOXC runtime root)
	@$(CP) "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_MANIFEST_FILE)" "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_MANIFEST_FILE)"
	@$(CP) "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_GENESIS_FILE)" "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_GENESIS_FILE)"
	@$(CP) "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_VALIDATORS_FILE)" "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_VALIDATORS_FILE)"
	@$(CP) "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_BOOTNODES_FILE)" "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_BOOTNODES_FILE)"
	@$(CP) "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_CERTIFICATE_FILE)" "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_CERTIFICATE_FILE)"
	@$(CP) "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_PROFILE_FILE)" "$(AOXC_RUNTIME_CONFIG_DIR)/$(RUNTIME_PROFILE_FILE)"
	@$(CP) "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_RELEASE_POLICY_FILE)" "$(AOXC_RUNTIME_CONFIG_DIR)/$(RUNTIME_RELEASE_POLICY_FILE)"
	@cd "$(AOXC_RUNTIME_IDENTITY_DIR)" && $(SHA256SUM) "$(RUNTIME_GENESIS_FILE)" > "$(RUNTIME_GENESIS_SHA256_FILE)"
	@$(SHA256SUM) "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_GENESIS_FILE)" | $(AWK) '{print $$1}' > "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_FINGERPRINT_FILE)"
	@{ \
		echo "source_root=$(AOXC_RUNTIME_SOURCE_ROOT)"; \
		echo "runtime_root=$(AOXC_RUNTIME_ROOT)"; \
		echo "identity_dir=$(AOXC_RUNTIME_IDENTITY_DIR)"; \
		echo "config_dir=$(AOXC_RUNTIME_CONFIG_DIR)"; \
		echo "state_dir=$(AOXC_RUNTIME_STATE_DIR)"; \
		echo "operator_dir=$(AOXC_RUNTIME_OPERATOR_DIR)"; \
		echo "db_dir=$(AOXC_RUNTIME_DB_DIR)"; \
		echo "installed_at_utc=$(call iso_utc_now)"; \
		echo "manifest_file=$(RUNTIME_MANIFEST_FILE)"; \
		echo "genesis_file=$(RUNTIME_GENESIS_FILE)"; \
		echo "profile_file=$(RUNTIME_PROFILE_FILE)"; \
		echo "release_policy_file=$(RUNTIME_RELEASE_POLICY_FILE)"; \
		echo "fingerprint_file=$(RUNTIME_FINGERPRINT_FILE)"; \
	} > "$(AOXC_RUNTIME_INSTALL_RECEIPT)"
	@echo "canonical-runtime" > "$(AOXC_ACTIVE_PROFILE_FILE)"
	@echo "Installed runtime bundle into: $(AOXC_RUNTIME_ROOT)"
	@echo "Install receipt: $(AOXC_RUNTIME_INSTALL_RECEIPT)"

runtime-verify: runtime-source-check
	$(call print_banner,Verifying materialized runtime bundle)
	$(call require_dir,$(AOXC_RUNTIME_IDENTITY_DIR))
	$(call require_dir,$(AOXC_RUNTIME_CONFIG_DIR))
	$(call require_file,$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_MANIFEST_FILE))
	$(call require_file,$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_GENESIS_FILE))
	$(call require_file,$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_GENESIS_SHA256_FILE))
	$(call require_file,$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_VALIDATORS_FILE))
	$(call require_file,$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_BOOTNODES_FILE))
	$(call require_file,$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_CERTIFICATE_FILE))
	$(call require_file,$(AOXC_RUNTIME_CONFIG_DIR)/$(RUNTIME_PROFILE_FILE))
	$(call require_file,$(AOXC_RUNTIME_CONFIG_DIR)/$(RUNTIME_RELEASE_POLICY_FILE))
	$(call require_file,$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_FINGERPRINT_FILE))
	@$(CMP) -s "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_MANIFEST_FILE)" "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_MANIFEST_FILE)" || { echo "Manifest mismatch between source and runtime"; exit 1; }
	@$(CMP) -s "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_GENESIS_FILE)" "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_GENESIS_FILE)" || { echo "Genesis mismatch between source and runtime"; exit 1; }
	@$(CMP) -s "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_VALIDATORS_FILE)" "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_VALIDATORS_FILE)" || { echo "Validators mismatch between source and runtime"; exit 1; }
	@$(CMP) -s "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_BOOTNODES_FILE)" "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_BOOTNODES_FILE)" || { echo "Bootnodes mismatch between source and runtime"; exit 1; }
	@$(CMP) -s "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_CERTIFICATE_FILE)" "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_CERTIFICATE_FILE)" || { echo "Certificate mismatch between source and runtime"; exit 1; }
	@$(CMP) -s "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_PROFILE_FILE)" "$(AOXC_RUNTIME_CONFIG_DIR)/$(RUNTIME_PROFILE_FILE)" || { echo "Profile mismatch between source and runtime"; exit 1; }
	@$(CMP) -s "$(AOXC_RUNTIME_SOURCE_ROOT)/$(SRC_RELEASE_POLICY_FILE)" "$(AOXC_RUNTIME_CONFIG_DIR)/$(RUNTIME_RELEASE_POLICY_FILE)" || { echo "Release policy mismatch between source and runtime"; exit 1; }
	@cd "$(AOXC_RUNTIME_IDENTITY_DIR)" && $(SHA256SUM) -c "$(RUNTIME_GENESIS_SHA256_FILE)"
	@ACTUAL_FINGERPRINT="$$(sha256sum "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_GENESIS_FILE)" | $(AWK) '{print $$1}')"; \
	STORED_FINGERPRINT="$$(cat "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_FINGERPRINT_FILE)")"; \
	[ "$$ACTUAL_FINGERPRINT" = "$$STORED_FINGERPRINT" ] || { echo "Runtime fingerprint drift detected"; exit 1; }
	@echo "Runtime verification passed."

runtime-activate: runtime-install runtime-verify db-init db-health
	$(call print_banner,Activating runtime)
	@echo "canonical-runtime" > "$(AOXC_ACTIVE_PROFILE_FILE)"
	@echo "Activated runtime root: $(AOXC_RUNTIME_ROOT)"
	@echo "Active profile marker: $(AOXC_ACTIVE_PROFILE_FILE)"

runtime-status:
	$(call print_banner,Printing runtime status)
	@echo "AOXC_ROOT=$(AOXC_ROOT)"
	@echo "RUNTIME_ROOT=$(AOXC_RUNTIME_ROOT)"
	@echo "IDENTITY_DIR=$(AOXC_RUNTIME_IDENTITY_DIR)"
	@echo "CONFIG_DIR=$(AOXC_RUNTIME_CONFIG_DIR)"
	@echo "STATE_DIR=$(AOXC_RUNTIME_STATE_DIR)"
	@echo "OPERATOR_DIR=$(AOXC_RUNTIME_OPERATOR_DIR)"
	@echo "DB_DIR=$(AOXC_RUNTIME_DB_DIR)"
	@if [ -f "$(AOXC_RUNTIME_INSTALL_RECEIPT)" ]; then \
		echo ""; \
		echo "[runtime install receipt]"; \
		$(CAT) "$(AOXC_RUNTIME_INSTALL_RECEIPT)"; \
	fi
	@if [ -f "$(AOXC_ACTIVE_PROFILE_FILE)" ]; then \
		echo ""; \
		echo "[active profile]"; \
		$(CAT) "$(AOXC_ACTIVE_PROFILE_FILE)"; \
	fi
	@if [ -f "$(AOXC_RUNTIME_HEALTH_FILE)" ]; then \
		echo ""; \
		echo "[runtime health]"; \
		$(CAT) "$(AOXC_RUNTIME_HEALTH_FILE)"; \
	fi

runtime-fingerprint:
	$(call print_banner,Printing runtime genesis fingerprint)
	$(call require_file,$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_GENESIS_FILE))
	@sha256sum "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_GENESIS_FILE)"

runtime-doctor:
	$(call print_banner,Running end-to-end runtime diagnostics)
	@$(MAKE) --no-print-directory runtime-print
	@$(MAKE) --no-print-directory runtime-source-check
	@if [ ! -f "$(AOXC_RUNTIME_IDENTITY_DIR)/$(RUNTIME_MANIFEST_FILE)" ]; then \
		echo "Runtime bundle is not materialized; running runtime-install."; \
		$(MAKE) --no-print-directory runtime-install; \
	fi
	@if [ -d "$(AOXC_RUNTIME_IDENTITY_DIR)" ]; then \
		$(MAKE) --no-print-directory runtime-verify; \
	else \
		echo "Runtime identity directory is absent; install has not been performed yet."; \
	fi
	@$(MAKE) --no-print-directory runtime-status
	@$(MAKE) --no-print-directory db-status
	@$(MAKE) --no-print-directory db-health
	@echo "Runtime diagnostics completed."

runtime-reinstall:
	$(call print_banner,Reinstalling runtime bundle)
	@$(MAKE) --no-print-directory runtime-reset
	@$(MAKE) --no-print-directory runtime-install
	@$(MAKE) --no-print-directory runtime-verify
	@$(MAKE) --no-print-directory db-init
	@$(MAKE) --no-print-directory db-health

runtime-reset:
	$(call print_banner,Resetting runtime, logs, db state, and receipts)
	@$(RM) -rf "$(AOXC_RUNTIME_ROOT)"
	@$(RM) -rf "$(AOXC_LOG_ROOT)"
	@$(RM) -f "$(AOXC_ACTIVE_PROFILE_FILE)"
	@$(RM) -f "$(AOXC_DB_STATUS_FILE)"
	@$(RM) -f "$(AOXC_RUNTIME_INSTALL_RECEIPT)"
	@$(RM) -f "$(AOXC_RUNTIME_HEALTH_FILE)"
	@echo "Runtime reset completed."

runtime-show-active:
	$(call print_banner,Printing active runtime marker)
	@if [ -f "$(AOXC_ACTIVE_PROFILE_FILE)" ]; then \
		echo "active-profile: $$(cat "$(AOXC_ACTIVE_PROFILE_FILE)")"; \
	else \
		echo "active profile marker is absent"; \
	fi

# --------------------------------------------------------------------
# Full four-node provision surface
# --------------------------------------------------------------------
AOXC_FULL_ROOT ?= $(AOXC_ROOT)-full-4nodes
AOXC_FULL_NETWORK_KIND ?= localnet
AOXC_FULL_ROUNDS ?= 3
CONTAINER_ENGINE ?= docker
CONTAINER_IMAGE ?= aoxchain-node:local
CONTAINER_COMPOSE_FILE ?= docker-compose.yaml

aoxc-full-4nodes:
	$(call print_banner,Provisioning full AOXC four-node layout)
	@AOXC_FULL_ROOT="$(AOXC_FULL_ROOT)" \
	AOXC_FULL_NETWORK_KIND="$(AOXC_FULL_NETWORK_KIND)" \
	AOXC_FULL_ROUNDS="$(AOXC_FULL_ROUNDS)" \
	./scripts/aoxc_full_4nodes.sh --force

aoxc-full-4nodes-docker:
	$(call print_banner,Provisioning full AOXC four-node layout with Docker assets)
	@AOXC_FULL_ROOT="$(AOXC_FULL_ROOT)" \
	AOXC_FULL_NETWORK_KIND="$(AOXC_FULL_NETWORK_KIND)" \
	AOXC_FULL_ROUNDS="$(AOXC_FULL_ROUNDS)" \
	AOXC_FULL_WITH_DOCKER_ASSETS=1 \
	./scripts/aoxc_full_4nodes.sh --force --with-docker-assets

production-full:
	$(call print_banner,Executing production-grade validation flow (runtime + release + docker layout))
	@$(MAKE) --no-print-directory runtime-source-check
	@$(MAKE) --no-print-directory runtime-bundle-compat-check
	@$(MAKE) --no-print-directory runtime-activate
	@$(MAKE) --no-print-directory quality-release
	@$(MAKE) --no-print-directory docker-check
	@$(MAKE) --no-print-directory aoxc-full-4nodes-docker
	@$(MAKE) --no-print-directory db-health
	@echo "Production-grade validation flow completed."

phase1-full:
	$(call print_banner,Running full phase-1 determinism validation flow)
	@$(CARGO) test -p tests phase1_full_readiness_surface_is_consistent
	@$(CARGO) test -p tests vm_phase1_execution_is_deterministic_across_replays
	@$(CARGO) test -p tests block_production_is_deterministic_for_permuted_body_sections
	@$(CARGO) test -p tests fork_choice_accepts_equal_height_siblings_with_deterministic_tiebreak
	@$(CARGO) test -p aoxcmd consensus_profile_gate_status_reports_pass_for_hybrid_testnet
	@$(CARGO) test -p aoxcunity
	@echo "Phase-1 determinism validation flow completed."

quantum-readiness-gate:
	$(call print_banner,Running AOXC quantum readiness structural gate)
	@./scripts/validation/quantum_readiness_gate.sh

quantum-full:
	$(call print_banner,Running AOXC full quantum validation flow)
	@$(MAKE) --no-print-directory phase1-full
	@$(MAKE) --no-print-directory aoxcvm-production-closure-gate
	@$(MAKE) --no-print-directory quantum-readiness-gate
	@echo "Quantum validation flow completed."

# --------------------------------------------------------------------
# Operations
# --------------------------------------------------------------------
ops-help:
	$(call print_banner,AOXC operator quick start)
	@printf "make ops-doctor\n"
	@printf "make ops-prepare\n"
	@printf "make ops-start\n"
	@printf "make ops-status\n"
	@printf "make ops-logs\n"

ops-doctor:
	$(call print_banner,Running operator readiness checks)
	@$(MAKE) --no-print-directory env-check
	@$(MAKE) --no-print-directory bootstrap-paths
	@$(MAKE) --no-print-directory runtime-doctor

ops-prepare:
	$(call print_banner,Preparing AOXC runtime automatically)
	@$(MAKE) --no-print-directory bootstrap-paths
	@$(MAKE) --no-print-directory runtime-activate
	@echo "Automatic runtime preparation completed."

ops-start: package-bin runtime-activate
	$(call print_banner,Starting AOXC runtime)
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" AOXC_LOG_DIR="$(AOXC_LOG_DIR)" ./scripts/runtime_daemon.sh start

ops-once: package-bin runtime-activate
	$(call print_banner,Running one AOXC runtime cycle)
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" AOXC_LOG_DIR="$(AOXC_LOG_DIR)" ./scripts/runtime_daemon.sh once

ops-stop:
	$(call print_banner,Stopping AOXC runtime)
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" AOXC_LOG_DIR="$(AOXC_LOG_DIR)" ./scripts/runtime_daemon.sh stop

ops-status:
	$(call print_banner,Showing AOXC runtime status)
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" AOXC_LOG_DIR="$(AOXC_LOG_DIR)" ./scripts/runtime_daemon.sh status

ops-restart:
	$(call print_banner,Restarting AOXC runtime)
	@$(MAKE) --no-print-directory ops-stop || true
	@$(MAKE) --no-print-directory ops-start

ops-logs:
	$(call print_banner,Tailing AOXC runtime logs)
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" AOXC_LOG_DIR="$(AOXC_LOG_DIR)" ./scripts/runtime_daemon.sh tail

ops-flow:
	$(call print_banner,Executing full automatic AOXC operational flow)
	@$(MAKE) --no-print-directory env-check
	@$(MAKE) --no-print-directory package-bin
	@$(MAKE) --no-print-directory runtime-activate
	@$(MAKE) --no-print-directory db-event ACTION=flow STATUS=started DETAIL=ops-flow
	@$(MAKE) --no-print-directory ops-start
	@$(MAKE) --no-print-directory db-event ACTION=flow STATUS=completed DETAIL=ops-flow

chain-help:
	$(call print_banner,AOXC persistent chain bootstrap targets)
	@printf "make chain-init AOXC_BOOTSTRAP_PROFILE=<validation|testnet|mainnet|devnet|localnet> AOXC_VALIDATOR_NAME=<name> AOXC_VALIDATOR_PASSWORD=<password>\n"
	@printf "make chain-add-account AOXC_NEW_ACCOUNT_ID=<id> AOXC_NEW_ACCOUNT_BALANCE=<amount> AOXC_NEW_ACCOUNT_ROLE=<role>\n"
	@printf "make chain-add-validator AOXC_VALIDATOR_ID=<id> AOXC_CONSENSUS_PUBLIC_KEY=<hex> AOXC_NETWORK_PUBLIC_KEY=<hex> AOXC_VALIDATOR_BALANCE=<amount>\n"
	@printf "make chain-start-persistent\n"

chain-init: package-bin bootstrap-paths
	$(call print_banner,Initializing persistent AOXC chain state)
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" "$(AOXC_BIN_PATH)" config-init --profile "$(AOXC_BOOTSTRAP_PROFILE)" --json-logs
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" "$(AOXC_BIN_PATH)" address-create --name "$(AOXC_VALIDATOR_NAME)" --profile "$(AOXC_BOOTSTRAP_PROFILE)" --password "$(AOXC_VALIDATOR_PASSWORD)"
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" "$(AOXC_BIN_PATH)" genesis-init --profile "$(AOXC_BOOTSTRAP_PROFILE)"
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" "$(AOXC_BIN_PATH)" genesis-validate
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" "$(AOXC_BIN_PATH)" node-bootstrap

chain-add-account: package-bin bootstrap-paths
	$(call print_banner,Adding account to persistent genesis)
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" "$(AOXC_BIN_PATH)" genesis-add-account --account-id "$(AOXC_NEW_ACCOUNT_ID)" --balance "$(AOXC_NEW_ACCOUNT_BALANCE)" --role "$(AOXC_NEW_ACCOUNT_ROLE)"
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" "$(AOXC_BIN_PATH)" genesis-validate

chain-add-validator: package-bin bootstrap-paths
	$(call print_banner,Adding validator to persistent genesis)
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" "$(AOXC_BIN_PATH)" genesis-add-validator --validator-id "$(AOXC_VALIDATOR_ID)" --consensus-public-key "$(AOXC_CONSENSUS_PUBLIC_KEY)" --network-public-key "$(AOXC_NETWORK_PUBLIC_KEY)" --balance "$(AOXC_VALIDATOR_BALANCE)"
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" "$(AOXC_BIN_PATH)" genesis-validate

chain-start-persistent: chain-init
	$(call print_banner,Starting persistent AOXC runtime)
	@$(MAKE) --no-print-directory ops-start

# --------------------------------------------------------------------
# High-level operator shortcuts
# --------------------------------------------------------------------
demo:
	$(call print_banner,Running AOXC demo flow)
	@./scripts/chain_demo.sh

localnet:
	$(call print_banner,Preparing localnet operator flow)
	@$(MAKE) --no-print-directory network-create AOXC_BOOTSTRAP_PROFILE=localnet
	@$(MAKE) --no-print-directory network-start
	@$(MAKE) --no-print-directory network-status

devnet:
	$(call print_banner,Preparing devnet operator flow)
	@$(MAKE) --no-print-directory network-create AOXC_BOOTSTRAP_PROFILE=devnet
	@$(MAKE) --no-print-directory network-start
	@$(MAKE) --no-print-directory network-status

testnet:
	$(call print_banner,Preparing testnet operator flow)
	@$(MAKE) --no-print-directory testnet-gate
	@$(MAKE) --no-print-directory network-create AOXC_BOOTSTRAP_PROFILE=testnet
	@$(MAKE) --no-print-directory network-start
	@$(MAKE) --no-print-directory doctor
	@$(MAKE) --no-print-directory audit-chain

testnet-gate:
	$(call print_banner,Running persistent testnet readiness gate)
	@./scripts/validation/persistent_testnet_gate.sh


testnet-readiness-gate:
	$(call print_banner,Running testnet readiness quality gate)
	@./scripts/validation/testnet_readiness_gate.sh

aoxcvm-phase3-gate:
	$(call print_banner,Running AOXCVM phase-3 release-grade gate)
	@./scripts/validation/aoxcvm_phase3_gate.sh

aoxcvm-production-closure-gate:
	$(call print_banner,Running AOXCVM full production closure gate)
	@./scripts/validation/aoxcvm_production_closure_gate.sh

reset:
	$(call print_banner,Resetting AOXC operator runtime)
	@$(MAKE) --no-print-directory runtime-reset

doctor:
	$(call print_banner,Running AOXC doctor surface)
	@$(MAKE) --no-print-directory ops-doctor

audit-chain:
	$(call print_banner,Running AOXC chain audit checks)
	@$(MAKE) --no-print-directory runtime-doctor
	@$(MAKE) --no-print-directory db-health

logs:
	$(call print_banner,Streaming AOXC logs)
	@$(MAKE) --no-print-directory ops-logs

down:
	$(call print_banner,Stopping AOXC runtime)
	@$(MAKE) --no-print-directory ops-stop

restart:
	$(call print_banner,Restarting AOXC runtime)
	@$(MAKE) --no-print-directory ops-restart

network-create:
	$(call print_banner,Creating AOXC network materials)
	@./scripts/chain_create.sh

network-start:
	$(call print_banner,Starting AOXC network)
	@./scripts/network_start.sh

network-stop:
	$(call print_banner,Stopping AOXC network)
	@./scripts/network_stop.sh

network-status:
	$(call print_banner,Reporting AOXC network status)
	@$(MAKE) --no-print-directory ops-status
	@$(MAKE) --no-print-directory chain-status

genesis-build:
	$(call print_banner,Validating AOXC genesis artifacts)
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" "$(AOXC_BIN_PATH)" genesis-validate
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" "$(AOXC_BIN_PATH)" genesis-hash

chain-status:
	$(call print_banner,Reporting AOXC chain status)
	@AOXC_HOME="$(AOXC_RUNTIME_ROOT)" "$(AOXC_BIN_PATH)" runtime-status

# --------------------------------------------------------------------
# AOXC-Q persistent testnet supervisor surfaces
# --------------------------------------------------------------------
aoxc-q-up:
	$(call print_banner,AOXC-Q up (provision + start))
	@$(BASH) "$(AOXC_Q_SCRIPT)" --action up --mode "$(AOXC_Q_MODE)" --home "$(AOXC_Q_HOME)" --env "$(AOXC_Q_ENV)" --profile "$(AOXC_Q_PROFILE)" --nodes "$(AOXC_Q_NODES)" --rounds "$(AOXC_Q_ROUNDS)" --sleep-min-secs "$(AOXC_Q_SLEEP_MIN_SECS)" --sleep-max-secs "$(AOXC_Q_SLEEP_MAX_SECS)" $(if $(filter 1,$(AOXC_Q_FORCE)),--force,)

aoxc-q-provision:
	$(call print_banner,AOXC-Q provision)
	@$(BASH) "$(AOXC_Q_SCRIPT)" --action provision --mode "$(AOXC_Q_MODE)" --home "$(AOXC_Q_HOME)" --env "$(AOXC_Q_ENV)" --profile "$(AOXC_Q_PROFILE)" --nodes "$(AOXC_Q_NODES)" --rounds "$(AOXC_Q_ROUNDS)" --sleep-min-secs "$(AOXC_Q_SLEEP_MIN_SECS)" --sleep-max-secs "$(AOXC_Q_SLEEP_MAX_SECS)" $(if $(filter 1,$(AOXC_Q_FORCE)),--force,)

aoxc-q-start:
	$(call print_banner,AOXC-Q start)
	@$(BASH) "$(AOXC_Q_SCRIPT)" --action start --mode "$(AOXC_Q_MODE)" --home "$(AOXC_Q_HOME)" --env "$(AOXC_Q_ENV)" --nodes "$(AOXC_Q_NODES)"

aoxc-q-stop:
	$(call print_banner,AOXC-Q stop)
	@$(BASH) "$(AOXC_Q_SCRIPT)" --action stop --mode "$(AOXC_Q_MODE)" --home "$(AOXC_Q_HOME)" --env "$(AOXC_Q_ENV)" --nodes "$(AOXC_Q_NODES)"

aoxc-q-restart:
	$(call print_banner,AOXC-Q restart)
	@$(BASH) "$(AOXC_Q_SCRIPT)" --action restart --mode "$(AOXC_Q_MODE)" --home "$(AOXC_Q_HOME)" --env "$(AOXC_Q_ENV)" --nodes "$(AOXC_Q_NODES)"

aoxc-q-status:
	$(call print_banner,AOXC-Q status)
	@$(BASH) "$(AOXC_Q_SCRIPT)" --action status --mode "$(AOXC_Q_MODE)" --home "$(AOXC_Q_HOME)" --env "$(AOXC_Q_ENV)" --nodes "$(AOXC_Q_NODES)"

# --------------------------------------------------------------------
# UI surfaces
# --------------------------------------------------------------------
ui: build-release-all bootstrap-desktop-paths
	$(call print_banner,Running AOXCHub UI)
	@AOXC_HOME="$(AOXC_DESKTOP_HOME)" "$(PWD)/target/release/aoxchub$(AOXC_EXE_SUFFIX)"

alpha:
	$(call print_banner,AOXC alpha target)
	@echo "No alpha-specific workflow is defined beyond the standard operator and packaging surfaces."
