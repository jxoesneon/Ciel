#!/usr/bin/env bash
# laya-serve launcher — sources env and execs the venv binary.
# Managed by launchd (com.ciel.system1) with KeepAlive for self-heal.
set -euo pipefail
SYSTEM1_DIR="${HOME}/.ciel/system1"
set -a
# shellcheck source=/dev/null
. "${SYSTEM1_DIR}/env"
set +a
exec "${SYSTEM1_DIR}/venv/bin/laya-serve"
