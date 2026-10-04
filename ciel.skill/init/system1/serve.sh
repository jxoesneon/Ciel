#!/usr/bin/env bash
# ~/.ciel/system1/serve.sh — Local System-1 (Laya / Jev) Service Scaffold
#
# Supports booting real laya-serve (when installed in venv) or zero-dependency
# local mock runner (mock_server.py) for development, testing, and offline modes.
#
# M3-EFFICIENCY:
# - LAYA_MODELS=typed-decisions (default): restricts RAM to < 1.5 GB RSS by keeping
#   a single calibrated ModernBERT model in memory rather than multi-model residency.
# - LAYA_MAX_LOADED=1: prevents concurrent checkpoint accumulation under Termux.
# - LAYA_THREADS=2 or 4: bounds torch intra-op CPU concurrency on mobile cores.

set -euo pipefail

SYSTEM1_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$SYSTEM1_DIR/env"
PID_FILE="$SYSTEM1_DIR/laya.pid"
MOCK_SCRIPT="$SYSTEM1_DIR/mock_server.py"
VENV_LAYA="$SYSTEM1_DIR/venv/bin/laya-serve"

# Ensure ~/.ciel/system1/env exists with secure 600 permissions
init_env() {
  if [[ ! -f "$ENV_FILE" ]]; then
    echo "[serve.sh] Initializing $ENV_FILE with default configuration..."
    local rand_key
    rand_key=$(python3 -c 'import secrets; print(secrets.token_hex(24))')
    cat <<EOF >"$ENV_FILE"
# Local Laya / System-1 configuration (machine-local, chmod 600)
# M3-EFFICIENCY: Single ModernBERT model resident (< 1.5 GB RSS)
LAYA_HOST=127.0.0.1
LAYA_PORT=8765
LAYA_PRELOAD=1
LAYA_MODELS=typed-decisions
LAYA_MAX_LOADED=1
# Intra-op CPU threads: capped to 2 or 4 for mobile Termux
LAYA_THREADS=4
LAYA_API_KEY=${rand_key}
CIEL_SYSTEM1_MODE=active
CIEL_SYSTEM1_MODEL=typed-decisions
EOF
    # Native MiniLM embedder when the ciel-dev binary is installed —
    # replaces the venv python + system1_embed.py semantic leg.
    if [[ -x "$SYSTEM1_DIR/../bin/ciel-dev" ]]; then
      echo "CIEL_SYSTEM1_EMBED_BIN=$SYSTEM1_DIR/../bin/ciel-dev" >>"$ENV_FILE"
    fi
    # Termux workaround: preload libpython for embedded torch runtimes —
    # only written when the library actually exists on this machine.
    local termux_py="/data/data/com.termux/files/usr/lib/libpython3.14.so"
    if [[ -f "$termux_py" ]]; then
      echo "LD_PRELOAD=$termux_py" >>"$ENV_FILE"
    fi
    chmod 600 "$ENV_FILE"
    echo "[serve.sh] Created $ENV_FILE (mode 600)"
  fi
}

init_env

# Source the env file
# shellcheck source=/dev/null
set -a
source "$ENV_FILE"
set +a

LAYA_HOST="${LAYA_HOST:-127.0.0.1}"
LAYA_PORT="${LAYA_PORT:-8765}"
LAYA_MODELS="${LAYA_MODELS:-typed-decisions}"
LAYA_MAX_LOADED="${LAYA_MAX_LOADED:-1}"
LAYA_THREADS="${LAYA_THREADS:-4}"
LAYA_API_KEY="${LAYA_API_KEY:-}"
export LAYA_HOST LAYA_PORT LAYA_API_KEY CIEL_SYSTEM1_KEY="${CIEL_SYSTEM1_KEY:-$LAYA_API_KEY}" LAYA_MODELS LAYA_MAX_LOADED LAYA_THREADS CIEL_SYSTEM1_MODEL="${CIEL_SYSTEM1_MODEL:-typed-decisions}"

is_port_listening() {
  python3 -c "
import socket, sys
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.settimeout(0.5)
res = s.connect_ex(('$LAYA_HOST', int('$LAYA_PORT')))
s.close()
sys.exit(0 if res == 0 else 1)
" 2>/dev/null
}

wait_for_health() {
  local max_retries="${1:-60}"
  local count=0
  while ((count < max_retries)); do
    if curl -s -m 1 "http://$LAYA_HOST:$LAYA_PORT/health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.5
    ((count++))
  done
  return 1
}

cmd_status() {
  echo "=== Laya / System-1 Status ==="
  echo "Endpoint: http://$LAYA_HOST:$LAYA_PORT"
  echo "Env File: $ENV_FILE"
  echo "Mode:     ${CIEL_SYSTEM1_MODE:-active}"
  if [[ -f "$PID_FILE" ]]; then
    local pid
    pid=$(cat "$PID_FILE")
    if kill -0 "$pid" 2>/dev/null; then
      echo "PID File: $PID_FILE (PID: $pid, RUNNING)"
    else
      echo "PID File: $PID_FILE (PID: $pid, STALE)"
    fi
  else
    echo "PID File: None"
  fi

  if is_port_listening; then
    echo "Port Check: LISTENING on $LAYA_HOST:$LAYA_PORT"
    local health_resp
    health_resp=$(curl -s -m 1 "http://$LAYA_HOST:$LAYA_PORT/health" 2>/dev/null || true)
    if [[ -n "$health_resp" ]]; then
      echo "Health:     $health_resp"
    else
      echo "Health:     Endpoint responding, but non-JSON /health"
    fi
  else
    echo "Port Check: NOT LISTENING"
  fi
}

cmd_start() {
  local force_mock=0
  local daemon=0

  for arg in "$@"; do
    case "$arg" in
      --mock | mock) force_mock=1 ;;
      --daemon | -d) daemon=1 ;;
    esac
  done

  if [[ "${CIEL_SYSTEM1_MOCK:-0}" == "1" ]]; then
    force_mock=1
  fi

  if is_port_listening; then
    echo "[serve.sh] Server is already running on http://$LAYA_HOST:$LAYA_PORT"
    cmd_status
    return 0
  fi

  local runner=""
  local runner_args=()

  if ((force_mock == 0)) && [[ -x "$VENV_LAYA" ]]; then
    echo "[serve.sh] Starting production Laya server via $VENV_LAYA..."
    runner="$VENV_LAYA"
    runner_args=()
  else
    if ((force_mock == 1)); then
      echo "[serve.sh] Starting mock server (--mock requested)..."
    else
      echo "[serve.sh] Real laya-serve ($VENV_LAYA) not found. Falling back to local mock server..."
    fi
    runner="python3"
    runner_args=("$MOCK_SCRIPT" "--host" "$LAYA_HOST" "--port" "$LAYA_PORT" "--pidfile" "$PID_FILE")
  fi

  if ((daemon == 1)); then
    echo "[serve.sh] Spawning background daemon..."
    python3 -c '
import os, sys
log_file = sys.argv[1]
pid_file = sys.argv[2]
cmd = sys.argv[3:]

if os.fork() > 0:
    sys.exit(0)
os.setsid()
if os.fork() > 0:
    sys.exit(0)
sys.stdout.flush()
sys.stderr.flush()
with open(os.devnull, "r") as devnull:
    os.dup2(devnull.fileno(), sys.stdin.fileno())
log_out = open(log_file, "a")
os.dup2(log_out.fileno(), sys.stdout.fileno())
os.dup2(log_out.fileno(), sys.stderr.fileno())

with open(pid_file, "w") as f:
    f.write(str(os.getpid()))

os.environ["LD_PRELOAD"] = "/data/data/com.termux/files/usr/lib/libpython3.14.so"
os.execvp(cmd[0], cmd)
' "$SYSTEM1_DIR/laya.log" "$PID_FILE" "$runner" "${runner_args[@]}"

    echo "[serve.sh] Daemon spawned, awaiting readiness..."
    if wait_for_health 60; then
      local live_pid="unknown"
      if [[ -f "$PID_FILE" ]]; then live_pid=$(cat "$PID_FILE"); fi
      echo "[serve.sh] Successfully started! http://$LAYA_HOST:$LAYA_PORT (PID: $live_pid)"
    else
      echo "[serve.sh] WARNING: Timed out waiting for health check. Check $SYSTEM1_DIR/laya.log"
      return 1
    fi
  else
    echo "[serve.sh] Running in foreground on http://$LAYA_HOST:$LAYA_PORT (Ctrl+C to stop)..."
    export LD_PRELOAD=/data/data/com.termux/files/usr/lib/libpython3.14.so
    exec "$runner" "${runner_args[@]}"
  fi
}

cmd_stop() {
  echo "[serve.sh] Stopping Laya / System-1 server..."
  local stopped=0
  if [[ -f "$PID_FILE" ]]; then
    local pid
    pid=$(cat "$PID_FILE")
    if kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      for _ in {1..15}; do
        if ! kill -0 "$pid" 2>/dev/null; then
          break
        fi
        sleep 0.2
      done
      if kill -0 "$pid" 2>/dev/null; then
        kill -9 "$pid" 2>/dev/null || true
      fi
      echo "[serve.sh] Stopped PID $pid"
      stopped=1
    fi
    rm -f "$PID_FILE"
  fi

  # Fallback port cleanup if still listening
  if is_port_listening; then
    local fpid
    fpid=$(lsof -ti ":$LAYA_PORT" 2>/dev/null || true)
    if [[ -n "$fpid" ]]; then
      kill "$fpid" 2>/dev/null || true
      echo "[serve.sh] Killed process on port $LAYA_PORT (PID: $fpid)"
      stopped=1
    fi
  fi

  if ((stopped == 0)); then
    echo "[serve.sh] No running server found."
  else
    echo "[serve.sh] Service stopped."
  fi
}

cmd_test() {
  echo "=== Running System-1 Verification Suite ==="
  local started_by_us=0
  if ! is_port_listening; then
    echo "[serve.sh] Service not running. Starting background server for test run..."
    cmd_start --daemon
    started_by_us=1
  fi

  echo ""
  echo "1. Health Endpoint Probe:"
  curl -sS "http://$LAYA_HOST:$LAYA_PORT/health"
  echo ""

  echo ""
  echo "2. Direct POST /v1/systemone (Safe tool query):"
  curl -sS -X POST "http://$LAYA_HOST:$LAYA_PORT/v1/systemone" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $LAYA_API_KEY" \
    -d '{"state": {"tool": "read", "command": "", "path": "file.txt"}, "questions": {"risk": {"type": "choice", "instructions": "test", "criteria": {"safe": "s", "dangerous": "d"}}}}'
  echo ""

  echo ""
  echo "3. Direct POST /v1/systemone (Dangerous tool query):"
  curl -sS -X POST "http://$LAYA_HOST:$LAYA_PORT/v1/systemone" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $LAYA_API_KEY" \
    -d '{"state": {"tool": "exec", "command": "rm -rf /", "path": ""}, "questions": {"risk": {"type": "choice", "instructions": "test", "criteria": {"safe": "s", "dangerous": "d"}}}}'
  echo ""

  echo ""
  echo "4. Chunked Transfer-Encoding Probe (?chunked=1):"
  curl -sS -X POST "http://$LAYA_HOST:$LAYA_PORT/v1/systemone?chunked=1" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $LAYA_API_KEY" \
    -d '{"state": {"tool": "exec", "command": "ls -l"}, "questions": {"risk": {"type": "choice", "instructions": "test", "criteria": {"safe": "s", "dangerous": "d"}}}}'
  echo ""

  echo ""
  echo "5. End-to-End via Ciel CLI (Rust Client: ciel system1 --decide):"
  local ciel_bin="$SYSTEM1_DIR/../bin/ciel"
  if [[ -x "$ciel_bin" ]]; then
    echo '{"surface": "pre_tool_risk", "state": {"tool": "exec", "command": "rm -rf /root/.ssh"}, "questions": {"risk": {"type": "choice", "instructions": "check", "criteria": {"safe": "s", "dangerous": "d"}}}}' |
      CIEL_SYSTEM1_URL="http://$LAYA_HOST:$LAYA_PORT" \
        CIEL_SYSTEM1_KEY="$LAYA_API_KEY" \
        "$ciel_bin" system1 --decide
  else
    echo "[serve.sh] ciel binary not found at $ciel_bin, skipping CLI test."
  fi

  if ((started_by_us == 1)); then
    echo ""
    echo "[serve.sh] Stopping temporary test mock instance..."
    cmd_stop
  fi
  echo ""
  echo "=== System-1 Verification Suite Finished ==="
}

usage() {
  cat <<EOF
Usage: $0 [start|mock|stop|status|restart|test|init-env] [options]

Commands:
  start [--mock] [--daemon|-d]  Start Laya server (prefers venv/bin/laya-serve, fallback to mock)
  mock  [--daemon|-d]           Force start lightweight local mock server
  stop                          Stop the running server
  status                        Check server status and health
  restart [options]             Restart server
  test                          Run end-to-end integration and dechunking tests
  init-env                      Initialize or regenerate ~/.ciel/system1/env
EOF
}

ACTION="${1:-start}"
shift || true

case "$ACTION" in
  start) cmd_start "$@" ;;
  mock) cmd_start --mock "$@" ;;
  stop) cmd_stop ;;
  status) cmd_status ;;
  restart)
    cmd_stop
    sleep 0.5
    cmd_start "$@"
    ;;
  test) cmd_test ;;
  init-env)
    rm -f "$ENV_FILE"
    init_env
    ;;
  -h | --help | help) usage ;;
  *)
    echo "Unknown command: $ACTION"
    usage
    exit 1
    ;;
esac
