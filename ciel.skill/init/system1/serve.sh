#!/usr/bin/env bash
# laya-serve launcher — sources ~/.ciel/system1/env and execs the venv binary.
# Install target: ~/.ciel/system1/serve.sh (chmod +x).
# Keep resident via your platform supervisor:
#   macOS: launchd agent  (init/launchd/com.ciel.system1.plist)
#   Linux: systemd user unit (init/systemd/ciel-system1.service)
set -euo pipefail
SYSTEM1_DIR="${HOME}/.ciel/system1"
set -a
# shellcheck source=/dev/null
. "${SYSTEM1_DIR}/env"
set +a
exec "${SYSTEM1_DIR}/venv/bin/laya-serve"
