#!/usr/bin/env bash
set -euo pipefail
python3 - <<'PY'
import sys
sys.path.insert(0, ".")
from paginate import paginate
assert paginate([1, 2, 3, 4, 5], 2, 1) == [1, 2]
assert paginate([1, 2, 3, 4, 5], 2, 2) == [3, 4]
assert paginate([1, 2, 3, 4, 5], 2, 3) == [5]
assert paginate([], 3, 1) == []
PY
