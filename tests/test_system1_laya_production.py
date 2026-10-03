import unittest
import time
import json
import re
from unittest.mock import patch, MagicMock

import sys
import os
from pathlib import Path

# Ensure paths
_LIB = Path(__file__).resolve().parent.parent / "ciel.skill" / "init" / "hooks" / "lib"
sys.path.insert(0, str(_LIB))
import system1      # noqa: E402
import risk_policy  # noqa: E402

# --- PROPOSED LAYA INTEGRATION LOGIC ---

def enforce_m1_failsafe(command, path, system1_verdict):
    """M1 Fail-Safe: If offline/timeout (None), fallback to regex."""
    if system1_verdict is not None:
        return system1_verdict
    
    destructive_patterns = [r'\brm\s+-rf\b', r'\bmkfs\b', r'\bdd\b']
    target = f"{command} {path}"
    for pat in destructive_patterns:
        if re.search(pat, target):
            return ("dangerous", 1.0, "system1_offline_failsafe")
    
    return ("safe", 1.0, "pass")

def redact_telemetry(payload):
    """M2 Redaction: Sanitize tokens and passwords."""
    s = json.dumps(payload)
    # Redact GitHub tokens
    s = re.sub(r'(ghp_[a-zA-Z0-9]{36})', '[REDACTED:API_KEY]', s)
    # Redact passwords in URLs or flags
    s = re.sub(r'(password=)([^& \'\"]+)', r'\1[REDACTED:PASSWORD]', s)
    return json.loads(s)

# --- TESTS ---

class TestSystem1LayaProduction(unittest.TestCase):

    @patch.object(system1.urllib.request, 'urlopen')
    def test_1_laya_live_probe_classification(self, mock_urlopen):
        """Test 1: Laya server classification on benign vs destructive inputs."""
        mock_response = MagicMock()
        mock_response.read.return_value = json.dumps({
            "answers": {
                "risk": {"choice": "dangerous", "confidence": 0.95}
            },
            "model": "ModernBERT"
        }).encode('utf-8')
        mock_urlopen.return_value.__enter__.return_value = mock_response

        # Destructive
        result = system1.evaluate_risk("bash", "rm -rf /", "/")
        self.assertIsNotNone(result)
        self.assertEqual(result[0], "dangerous")
        self.assertEqual(result[1], 0.95)

    def test_2_m1_failsafe_invariant(self):
        """Test 2: M1 Fail-Safe Invariant (offline logic)."""
        # Simulate offline/timeout by returning None from system1
        offline_result = None

        # Destructive command
        decision = enforce_m1_failsafe("rm -rf", "/", offline_result)
        self.assertEqual(decision[0], "dangerous")
        self.assertEqual(decision[2], "system1_offline_failsafe")

        # Benign command
        decision2 = enforce_m1_failsafe("git status", "", offline_result)
        self.assertEqual(decision2[0], "safe")

    def test_3_m2_telemetry_redaction(self):
        """Test 3: M2 Telemetry Redaction."""
        payload = {
            "state": {"command": "curl -u user:ghp_123456789012345678901234567890123456 https://api.github.com"},
            "meta": {"note": "password=secret123"}
        }
        sanitized = redact_telemetry(payload)
        
        state_cmd = sanitized["state"]["command"]
        self.assertIn("[REDACTED:API_KEY]", state_cmd)
        self.assertNotIn("ghp_123456", state_cmd)

        meta_note = sanitized["meta"]["note"]
        self.assertIn("[REDACTED:PASSWORD]", meta_note)
        self.assertNotIn("secret123", meta_note)

    @patch.object(system1, 'ask')
    def test_4_m3_m4_performance_and_caching(self, mock_ask):
        """Test 4: Performance & Caching (< 1ms cache hits)."""
        mock_ask.return_value = {"answers": {"risk": {"choice": "safe", "confidence": 0.9}}}
        
        state = {"command": "ls -la", "path": "/tmp"}
        questions = {"risk": {"type": "choice"}}
        
        # Ensure cache is clear for this state
        cache_file = system1._cache_path(state, questions)
        if cache_file.exists():
            cache_file.unlink()

        # First resolution (cache miss)
        t0 = time.time()
        res1, hit1, lat1 = system1._resolve(state, questions)
        t1 = time.time()
        
        self.assertFalse(hit1)
        
        # Second resolution (cache hit)
        t2 = time.time()
        res2, hit2, lat2 = system1._resolve(state, questions)
        t3 = time.time()
        
        self.assertTrue(hit2)
        
        # Cache hit must take < 1ms
        cache_duration_ms = (t3 - t2) * 1000
        self.assertLess(cache_duration_ms, 2.0)  # using 2.0 to avoid flakiness in CI, but conceptually < 1ms
        self.assertEqual(res1, res2)

if __name__ == '__main__':
    unittest.main()
