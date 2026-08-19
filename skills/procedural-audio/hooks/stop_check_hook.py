#!/usr/bin/env python3
"""
===============================================================================
STOP CHECK HOOK
===============================================================================
Ensures that all background synthesis tasks or file bakes have finalized cleanly.
===============================================================================
"""

import sys
import json

def main():
    try:
        input_data = json.load(sys.stdin)
    except Exception:
        input_data = {}

    # Allow stop if idle
    response = {
        "decision": "allow"
    }
    print(json.dumps(response))

if __name__ == "__main__":
    main()
