---
title: "Interop test run after WP-06, WP-08, and WP-09"
type: diary
tags: [diary, session]
created: 2026-07-09
status: active
---

# Interop test run after WP-06, WP-08, and WP-09

**Date:** 2026-07-09
**Project:** dart_ipfs
**Trigger:** Run the Docker-based interop suite after the recent WP-06, WP-08, and WP-09 changes.

## What was done

1. Started Docker Desktop so the daemon was reachable (`docker info` responsive after ~13.5s).
2. In `C:/Users/josee/IPFS/test/interop` ran `docker compose up -d --build` to build/recreate the interop images:
   - `dart_ipfs:interop`
   - `helia_interop:latest`
   - Containers: `dart_ipfs_interop`, `helia_interop`, `kubo_interop`, `dart_test_runner` all started healthy.
3. Ran the interop tests inside the `test-runner` container:
   ```bash
   docker compose exec -T test-runner sh -c "cd /app && dart test --preset interop --reporter=compact test/interop"
   ```

## Result

- **Final status:** `All tests passed!`
- **Test count:** `+17` (17 passed, 0 skipped, 0 failed)
- **Exit code:** `0`
- **Elapsed test time:** ~29 seconds inside the reporter

## Test cases covered

- P0 Bitswap fetch with Kubo
  - `dart_ipfs can fetch a block from Kubo via Bitswap`
  - `Kubo can fetch a block from dart_ipfs via Bitswap`
- P1 DHT provide/find with Kubo
  - `dart_ipfs provides a CID and Kubo finds it as a provider`
  - `Kubo provides a CID and dart_ipfs finds it as a provider`
- P1 IPNS resolution with Kubo
  - `dart_ipfs publishes a signed IPNS record and Kubo resolves it`
- (Other interop tests from `test/interop/test/` included in the `+17` count)

## Blockers / notes

None. The interop stack came up cleanly, images built without errors, and the full preset passed.
