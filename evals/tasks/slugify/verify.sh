#!/usr/bin/env bash
set -euo pipefail
python3 - <<'PY'
import sys
sys.path.insert(0, ".")
from slug import slugify
assert slugify("Hello World!") == "hello-world"
assert slugify("  A--B__c  ") == "a-b-c"
assert slugify("") == ""
PY
