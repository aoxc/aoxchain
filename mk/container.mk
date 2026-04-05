# --------------------------------------------------------------------
# Container runtime surface (Docker / Podman)
# --------------------------------------------------------------------
CONTAINER_ENGINE ?= auto
CONTAINER_IMAGE ?= aoxchain-node:local
CONTAINER_COMPOSE_FILE ?= docker-compose.yaml

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
	$(call require_file,$(CONTAINER_COMPOSE_FILE))
	@if [ "$(CONTAINER_ENGINE)" = "auto" ]; then \
		if command -v podman >/dev/null 2>&1; then \
			$(MAKE) --no-print-directory podman-check; \
			echo "Container engine resolved to: podman"; \
		elif command -v docker >/dev/null 2>&1; then \
			$(MAKE) --no-print-directory docker-check; \
			echo "Container engine resolved to: docker"; \
		else \
			echo "No supported container engine found (docker/podman)."; \
			exit 1; \
		fi; \
	elif [ "$(CONTAINER_ENGINE)" = "docker" ]; then \
		$(MAKE) --no-print-directory docker-check; \
	elif [ "$(CONTAINER_ENGINE)" = "podman" ]; then \
		$(MAKE) --no-print-directory podman-check; \
	else \
		echo "Unsupported CONTAINER_ENGINE=$(CONTAINER_ENGINE). Use auto, docker, or podman."; \
		exit 1; \
	fi

container-build: container-check
	$(call print_banner,Building local AOXChain container image with $(CONTAINER_ENGINE))
	@if [ "$(CONTAINER_ENGINE)" = "auto" ]; then \
		if command -v podman >/dev/null 2>&1; then \
			podman build -t "$(CONTAINER_IMAGE)" .; \
		else \
			docker build -t "$(CONTAINER_IMAGE)" .; \
		fi; \
	else \
		"$(CONTAINER_ENGINE)" build -t "$(CONTAINER_IMAGE)" .; \
	fi

container-config: container-check
	$(call print_banner,Rendering compose configuration with $(CONTAINER_ENGINE))
	@if [ "$(CONTAINER_ENGINE)" = "auto" ]; then \
		if command -v podman >/dev/null 2>&1; then \
			podman compose -f "$(CONTAINER_COMPOSE_FILE)" config; \
		elif docker compose version >/dev/null 2>&1; then \
			docker compose -f "$(CONTAINER_COMPOSE_FILE)" config; \
		else \
			docker-compose -f "$(CONTAINER_COMPOSE_FILE)" config; \
		fi; \
	elif [ "$(CONTAINER_ENGINE)" = "docker" ]; then \
		if docker compose version >/dev/null 2>&1; then \
			docker compose -f "$(CONTAINER_COMPOSE_FILE)" config; \
		else \
			docker-compose -f "$(CONTAINER_COMPOSE_FILE)" config; \
		fi; \
	else \
		podman compose -f "$(CONTAINER_COMPOSE_FILE)" config; \
	fi

container-up: container-check
	$(call print_banner,Starting AOXChain compose topology with $(CONTAINER_ENGINE))
	@if [ "$(CONTAINER_ENGINE)" = "auto" ]; then \
		if command -v podman >/dev/null 2>&1; then \
			podman compose -f "$(CONTAINER_COMPOSE_FILE)" up --build; \
		elif docker compose version >/dev/null 2>&1; then \
			docker compose -f "$(CONTAINER_COMPOSE_FILE)" up --build; \
		else \
			docker-compose -f "$(CONTAINER_COMPOSE_FILE)" up --build; \
		fi; \
	elif [ "$(CONTAINER_ENGINE)" = "docker" ]; then \
		if docker compose version >/dev/null 2>&1; then \
			docker compose -f "$(CONTAINER_COMPOSE_FILE)" up --build; \
		else \
			docker-compose -f "$(CONTAINER_COMPOSE_FILE)" up --build; \
		fi; \
	else \
		podman compose -f "$(CONTAINER_COMPOSE_FILE)" up --build; \
	fi

container-down: container-check
	$(call print_banner,Stopping AOXChain compose topology with $(CONTAINER_ENGINE))
	@if [ "$(CONTAINER_ENGINE)" = "auto" ]; then \
		if command -v podman >/dev/null 2>&1; then \
			podman compose -f "$(CONTAINER_COMPOSE_FILE)" down; \
		elif docker compose version >/dev/null 2>&1; then \
			docker compose -f "$(CONTAINER_COMPOSE_FILE)" down; \
		else \
			docker-compose -f "$(CONTAINER_COMPOSE_FILE)" down; \
		fi; \
	elif [ "$(CONTAINER_ENGINE)" = "docker" ]; then \
		if docker compose version >/dev/null 2>&1; then \
			docker compose -f "$(CONTAINER_COMPOSE_FILE)" down; \
		else \
			docker-compose -f "$(CONTAINER_COMPOSE_FILE)" down; \
		fi; \
	else \
		podman compose -f "$(CONTAINER_COMPOSE_FILE)" down; \
	fi
