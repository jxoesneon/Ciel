#!/usr/bin/env bash
# Post-start warmup: pay the one-time first-forward JIT cost for each
# preloaded checkpoint so the first real request is not a ~8s stall.
# Waits for /health, then fires one tiny question per model.
set -u
cd "$(dirname "$0")"
set -a; . ./env; set +a
url="http://${LAYA_HOST:-127.0.0.1}:${LAYA_PORT:-8765}"
for _ in $(seq 1 90); do
    curl -sf -m 2 "$url/health" >/dev/null 2>&1 && break
    sleep 1
done
for model in ${LAYA_MODELS//,/ }; do
    curl -sf -m 60 -o /dev/null -X POST "$url/v1/systemone" \
        -H 'content-type: application/json' \
        -H "authorization: Bearer ${LAYA_API_KEY}" \
        -d "{\"state\":{\"warmup\":\"true\"},\"questions\":{\"w\":{\"type\":\"choice\",\"instructions\":\"ok?\",\"criteria\":{\"yes\":\"y\",\"no\":\"n\"}}},\"model\":\"${model}\"}" \
        || true
done
exit 0
