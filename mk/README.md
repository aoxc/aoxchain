# Makefile Modules

This directory contains modular make surfaces included by the repository root `Makefile`.

## Current modules

- `container.mk`: Docker/Podman runtime checks and unified container lifecycle targets.

## Integration contract

- Root `Makefile` remains the canonical entrypoint.
- Modules are loaded with `-include $(MK_DIR)/*.mk` style includes to preserve backward compatibility.
- New operational surfaces should be added as focused modules instead of extending the root file indefinitely.
