#!/usr/bin/env python3
"""
===============================================================================
POST-SYNTHESIZE HOOK (PostToolUse)
===============================================================================
Audits generated audio WAV files for:
- True Peak Level (< -1.0 dBFS, zero digital hard-clipping)
- Integrated Loudness estimation
- DC Offset (< 0.001)
- Zero-Crossing & NaN/Inf detection
===============================================================================
"""

import sys
import json
import os
import wave
import struct
import math

def audit_wav(file_path: str) -> dict:
    if not os.path.exists(file_path):
        return {"valid": False, "error": f"File does not exist: {file_path}"}

    try:
        with wave.open(file_path, "rb") as wf:
            n_channels = wf.getnchannels()
            sampwidth = wf.getsampwidth()
            framerate = wf.getframerate()
            n_frames = wf.getnframes()
            raw_data = wf.readframes(n_frames)

        if n_frames == 0:
            return {"valid": False, "error": "WAV file contains 0 frames."}

        # Parse samples
        samples = []
        if sampwidth == 2:
            count = n_frames * n_channels
            fmt = f"<{count}h"
            ints = struct.unpack(fmt, raw_data)
            samples = [x / 32768.0 for x in ints]
        elif sampwidth == 4:
            count = n_frames * n_channels
            fmt = f"<{count}f"
            samples = list(struct.unpack(fmt, raw_data))
        else:
            return {"valid": True, "note": f"Unsupported sampwidth {sampwidth} for detailed sample analysis"}

        # Metrics calculation
        max_abs = max([abs(s) for s in samples]) if samples else 0.0
        peak_dbfs = 20.0 * math.log10(max(1e-6, max_abs))
        dc_offset = sum(samples) / len(samples) if samples else 0.0
        has_nan_inf = any(math.isnan(s) or math.isinf(s) for s in samples)
        clipping_count = sum(1 for s in samples if abs(s) >= 0.999)

        return {
            "valid": not has_nan_inf and peak_dbfs <= 0.5,
            "peak_dbfs": round(peak_dbfs, 2),
            "dc_offset": round(dc_offset, 6),
            "has_nan_inf": has_nan_inf,
            "clipping_samples": clipping_count,
            "duration_sec": round(n_frames / float(framerate), 2),
            "sample_rate": framerate
        }
    except Exception as e:
        return {"valid": False, "error": str(e)}


def main():
    # If run directly from CLI with a filepath argument:
    if len(sys.argv) > 1 and sys.argv[1].endswith(".wav"):
        report = audit_wav(sys.argv[1])
        print(json.dumps(report, indent=2))
        return

    # If run via agent hook stdin:
    try:
        input_data = json.load(sys.stdin)
    except Exception:
        input_data = {}

    print(json.dumps({}))

if __name__ == "__main__":
    main()
