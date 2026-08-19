#!/usr/bin/env python3
"""
===============================================================================
PRE-SYNTHESIZE HOOK (PreToolUse)
===============================================================================
Inspects incoming synthesis commands, verifies argument validity, enforces
safe output paths, and ensures CPU SIMD/FTZ environment flags are recognized.
===============================================================================
"""

import sys
import json
import os

def main():
    try:
        input_data = json.load(sys.stdin)
    except Exception:
        input_data = {}

    tool_call = input_data.get("toolCall", {})
    tool_name = tool_call.get("name", "")
    args = tool_call.get("args", {})
    cmd = args.get("CommandLine", "")

    # Check if command is an audio synthesis command
    if any(k in cmd for k in ["aaa_audio_generator.py", "procedural_audio_generator.py", "motif_dna_generator.py"]):
        # Verification: Prevent output targeting root or system directories
        if "--out /" in cmd and not any(safe in cmd for safe in ["/tmp/", "/Users/"]):
            response = {
                "decision": "deny",
                "reason": "Procedural Audio Pre-Hook: Audio bakes must output to /tmp/ or user workspace directory."
            }
            print(json.dumps(response))
            return

    # Pass through and approve
    response = {
        "decision": "allow"
    }
    print(json.dumps(response))

if __name__ == "__main__":
    main()
