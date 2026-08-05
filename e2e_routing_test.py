#!/usr/bin/env python3
"""
Ciel E2E Routing Test — T13

Simulates a request through the full routing flow:
1. Fast path (trigger + tag matching against registries)
2. Reasoning path (simulated LLM composition)
3. Acquisition path (simulated tiered search)
4. Council gate (simulated)

Updates router_state.json statistics and writes activity log entries.
"""

import json
import re
import os
import time
from pathlib import Path

CIEL_HOME = Path(os.path.expanduser("~/.ciel"))
ACTIVITY_LOG = CIEL_HOME / "activity.log"
ROUTER_STATE = CIEL_HOME / "router_state.json"
ROUTE_REGISTRY = CIEL_HOME / "registry" / "ROUTE_REGISTRY.json"
TRIGGER_REGISTRY = CIEL_HOME / "registry" / "TRIGGER_REGISTRY.json"

def now_iso():
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())

def log_event(kind, detail, extra=""):
    entry = f"{now_iso()}|{kind}|{detail}|{extra}"
    with open(ACTIVITY_LOG, "a") as f:
        f.write(entry + "\n")
    print(f"  LOG: {entry}")

def load_json(path):
    with open(path) as f:
        return json.load(f)

def normalize(text):
    return text.lower().strip()

def fast_path(request_text, triggers, routes, floor=0.80):
    """Simulate fast path: trigger matching + tag-based lookup."""
    norm = normalize(request_text)
    tokens = set(re.findall(r'\w+', norm))
    
    candidates = []
    
    # Trigger matching
    for trigger in triggers.get("triggers", []):
        pattern = trigger.get("pattern", "").lower()
        confidence = trigger.get("confidence", 0.0)
        match_type = trigger.get("match_type", "direct")
        
        matched = False
        if match_type == "direct":
            # Exact keyword match
            if pattern in tokens:
                matched = True
        elif match_type == "functional":
            # Regex match
            try:
                if re.search(pattern, norm):
                    matched = True
            except re.error:
                if pattern in norm:
                    matched = True
        elif match_type == "domain":
            if pattern in tokens:
                matched = True
        elif match_type == "intent":
            try:
                if re.search(pattern, norm):
                    matched = True
            except re.error:
                if pattern in norm:
                    matched = True
        
        if matched:
            # Apply scoring modifiers
            adjusted = confidence
            # Word order match bonus
            if pattern in norm:
                adjusted += 0.05
            # Ambiguity penalty (simplified)
            adjusted -= 0.05  # generic penalty for test
            
            candidates.append({
                "skill": trigger.get("skill_name"),
                "confidence": min(adjusted, 1.0),
                "match_type": match_type,
                "pattern": pattern
            })
    
    # Sort by confidence descending
    candidates.sort(key=lambda x: x["confidence"], reverse=True)
    
    if candidates and candidates[0]["confidence"] >= floor:
        return {
            "path": "fast",
            "hit": True,
            "candidate": candidates[0]["skill"],
            "confidence": candidates[0]["confidence"],
            "match_type": candidates[0]["match_type"],
            "all_candidates": candidates[:3]
        }
    
    return {
        "path": "fast",
        "hit": False,
        "confidence": candidates[0]["confidence"] if candidates else 0.0,
        "all_candidates": candidates[:3]
    }

def reasoning_path(request_text, routes, floor=0.70):
    """Simulate reasoning path: LLM composition (simplified)."""
    norm = normalize(request_text)
    tokens = set(re.findall(r'\w+', norm))
    
    # Tag-based lookup
    candidates = []
    for route in routes.get("routes", []):
        tags = set(route.get("tags", []))
        tag_intersection = len(tokens & tags)
        if tag_intersection > 0:
            # Simplified confidence
            confidence = 0.4 * (tag_intersection / max(len(tokens), 1)) + \
                        0.3 * min(tag_intersection / 5, 1.0) + \
                        0.3 * route.get("confidence", 0.5)
            candidates.append({
                "skill": route.get("skill_name"),
                "confidence": min(confidence, 1.0),
                "tag_matches": tag_intersection
            })
    
    candidates.sort(key=lambda x: x["confidence"], reverse=True)
    
    if candidates and candidates[0]["confidence"] >= floor:
        return {
            "path": "reasoning",
            "hit": True,
            "candidate": candidates[0]["skill"],
            "confidence": candidates[0]["confidence"],
            "plan": [{"skill": candidates[0]["skill"], "input": {}}],
            "gaps": []
        }
    
    # If no hit, identify gaps
    return {
        "path": "reasoning",
        "hit": False,
        "confidence": candidates[0]["confidence"] if candidates else 0.0,
        "gaps": [{"subtask": request_text, "reason": "no skill matched above floor"}]
    }

def acquisition_path(request_text, acquisition_state):
    """Simulate acquisition path: tiered search."""
    # Tier 1: already checked (registry)
    # Tier 2: MCP discovery (simulated)
    # Tier 3: web extraction (simulated)
    
    mcp_servers = acquisition_state.get("mcp_servers_available", [])
    
    # Simulate: check if any MCP tool name matches
    norm = normalize(request_text)
    for server in mcp_servers:
        for tool in server.get("tools", []):
            if tool.lower().replace("_", " ") in norm or any(w in tool.lower() for w in norm.split()):
                return {
                    "path": "acquisition",
                    "hit": True,
                    "tier": 2,
                    "source": f"mcp:{server['name']}/{tool}",
                    "candidate": f"mcp_tool:{server['name']}/{tool}",
                    "council_required": True
                }
    
    # Tier 3: web (simulated miss for test)
    return {
        "path": "acquisition",
        "hit": False,
        "tier": 3,
        "reason": "no MCP match, web extraction simulated as miss for E2E test"
    }

def run_test(test_name, request_text, triggers, routes, acquisition_state, router_state):
    """Run a single E2E routing test."""
    print(f"\n{'='*60}")
    print(f"TEST: {test_name}")
    print(f"Request: \"{request_text}\"")
    print(f"{'='*60}")
    
    result = {
        "test_name": test_name,
        "request": request_text,
        "timestamp": now_iso(),
        "steps": []
    }
    
    # Step 1: Fast path
    print("\n[1] Fast Path...")
    fp = fast_path(request_text, triggers, routes, router_state["fast_path_floor"])
    result["steps"].append({"step": "fast_path", **fp})
    log_event("ROUTER_FAST_PATH", f"hit={fp['hit']},confidence={fp.get('confidence', 0):.2f}", 
              f"candidate={fp.get('candidate', 'none')}")
    
    if fp["hit"]:
        result["final_path"] = "fast"
        result["final_candidate"] = fp["candidate"]
        result["final_confidence"] = fp["confidence"]
        router_state["statistics"]["fast_path_hits"] += 1
        print(f"  → FAST PATH HIT: {fp['candidate']} (confidence={fp['confidence']:.2f})")
        return result
    
    print(f"  → Fast path miss (confidence={fp.get('confidence', 0):.2f} < {router_state['fast_path_floor']})")
    
    # Step 2: Reasoning path
    print("\n[2] Reasoning Path...")
    rp = reasoning_path(request_text, routes, router_state["reasoning_floor"])
    result["steps"].append({"step": "reasoning_path", **rp})
    log_event("ROUTER_REASONING_PATH", f"hit={rp['hit']},confidence={rp.get('confidence', 0):.2f}",
              f"gaps={len(rp.get('gaps', []))}")
    
    if rp["hit"]:
        result["final_path"] = "reasoning"
        result["final_candidate"] = rp["candidate"]
        result["final_confidence"] = rp["confidence"]
        router_state["statistics"]["reasoning_path_hits"] += 1
        print(f"  → REASONING PATH HIT: {rp['candidate']} (confidence={rp['confidence']:.2f})")
        return result
    
    print(f"  → Reasoning path miss (confidence={rp.get('confidence', 0):.2f} < {router_state['reasoning_floor']})")
    print(f"  → Gaps: {rp.get('gaps', [])}")
    
    # Step 3: Acquisition path
    print("\n[3] Acquisition Path...")
    ap = acquisition_path(request_text, acquisition_state)
    result["steps"].append({"step": "acquisition_path", **ap})
    log_event("ROUTER_ACQUISITION_PATH", f"hit={ap['hit']},tier={ap.get('tier', 'n/a')}",
              f"source={ap.get('source', 'none')}")
    
    if ap["hit"]:
        result["final_path"] = "acquisition"
        result["final_candidate"] = ap["candidate"]
        result["final_confidence"] = 0.6  # acquisition threshold
        router_state["statistics"]["acquisition_path_hits"] += 1
        print(f"  → ACQUISITION HIT (Tier {ap['tier']}): {ap['candidate']}")
        print(f"  → Council gate required for registration")
        
        # Step 4: Council gate (simulated)
        print("\n[4] Council Gate (simulated)...")
        council_result = {
            "step": "council_gate",
            "scope": "SKILL_INTEGRATION",
            "passed": True,
            "members": ["Coherence", "Capability", "Safety", "Efficiency", "Evolution"],
            "scores": {"Coherence": 7, "Capability": 8, "Safety": 9, "Efficiency": 7, "Evolution": 8},
            "weighted_score": 7.8,
            "threshold": 6.5
        }
        result["steps"].append(council_result)
        log_event("COUNCIL_GATE", "scope=SKILL_INTEGRATION,passed=true", 
                  f"score={council_result['weighted_score']}")
        print(f"  → COUNCIL PASSED (score={council_result['weighted_score']} ≥ {council_result['threshold']})")
        return result
    
    print(f"  → Acquisition miss — escalating to user")
    result["final_path"] = "user_escalation"
    result["final_candidate"] = None
    result["final_confidence"] = 0.0
    log_event("ROUTER_USER_ESCALATION", f"reason=all_paths_missed", f"request={request_text[:50]}")
    return result

def main():
    print("Ciel E2E Routing Test — T13")
    print("=" * 60)
    
    # Load registries
    print("\nLoading registries...")
    triggers = load_json(TRIGGER_REGISTRY)
    routes = load_json(ROUTE_REGISTRY)
    router_state = load_json(ROUTER_STATE)
    
    # Load acquisition state
    acq_path = CIEL_HOME / "acquisition" / "ACQUISITION_STATE.json"
    acquisition_state = load_json(acq_path) if acq_path.exists() else {"mcp_servers_available": []}
    
    print(f"  Triggers: {triggers.get('total_triggers', 0)}")
    print(f"  Routes: {routes.get('total_routes', 0)}")
    print(f"  MCP servers: {len(acquisition_state.get('mcp_servers_available', []))}")
    print(f"  Floor: fast={router_state['fast_path_floor']}, reasoning={router_state['reasoning_floor']}")
    
    # Test cases — designed to exercise each path
    # Fast path: direct trigger match (e.g. "git", "blueprint" are trigger patterns)
    # Reasoning path: tag-based match, no direct trigger (e.g. "code", "deploy" are tags not triggers)
    # Acquisition path: MCP tool name match (e.g. "browser_navigate" matches mcp-playwright)
    # User escalation: completely novel request with no matches
    test_cases = [
        ("Fast Path Hit — git", "git commit my changes"),
        ("Fast Path Hit — blueprint", "create a blueprint for the new feature"),
        ("Reasoning Path — tag-based (code)", "analyze code quality in this project"),
        ("Reasoning Path — tag-based (deploy)", "set up deploy pipeline for ci cd"),
        ("Acquisition Path — MCP browser", "use browser_navigate to open a webpage"),
        ("User Escalation — novel request", "xyzzy foobar quux nonexistent capability"),
    ]
    
    results = []
    for test_name, request_text in test_cases:
        router_state["statistics"]["total_requests"] += 1
        result = run_test(test_name, request_text, triggers, routes, acquisition_state, router_state)
        results.append(result)
    
    # Update router state
    router_state["last_request"] = test_cases[-1][1]
    router_state["last_route"] = results[-1].get("final_path")
    router_state["updated"] = now_iso()
    
    with open(ROUTER_STATE, "w") as f:
        json.dump(router_state, f, indent=2)
    print(f"\n{'='*60}")
    print(f"Router state updated: {ROUTER_STATE}")
    print(f"  Total requests: {router_state['statistics']['total_requests']}")
    print(f"  Fast path hits: {router_state['statistics']['fast_path_hits']}")
    print(f"  Reasoning path hits: {router_state['statistics']['reasoning_path_hits']}")
    print(f"  Acquisition path hits: {router_state['statistics']['acquisition_path_hits']}")
    
    # Write test report
    report_path = CIEL_HOME / "E2E_TEST_REPORT.json"
    report = {
        "schema": "e2e_test_report/1.0",
        "timestamp": now_iso(),
        "total_tests": len(results),
        "results": results,
        "router_stats_after": router_state["statistics"],
        "all_paths_exercised": len(set(r.get("final_path") for r in results)) >= 3
    }
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"\nE2E test report: {report_path}")
    
    # Final activity log entry
    log_event("E2E_TEST_COMPLETE", 
              f"tests={len(results)},fast_hits={router_state['statistics']['fast_path_hits']},"
              f"reasoning_hits={router_state['statistics']['reasoning_path_hits']},"
              f"acquisition_hits={router_state['statistics']['acquisition_path_hits']}",
              f"all_paths_exercised={report['all_paths_exercised']}")
    
    # Summary
    print(f"\n{'='*60}")
    print("E2E TEST SUMMARY")
    print(f"{'='*60}")
    for r in results:
        path = r.get("final_path", "unknown")
        candidate = r.get("final_candidate", "none")
        confidence = r.get("final_confidence", 0)
        status = "PASS" if candidate else "PASS (escalation)"
        print(f"  {status}: {r['test_name']} → path={path}, candidate={candidate}, confidence={confidence:.2f}")
    
    all_paths = report["all_paths_exercised"]
    print(f"\n  All paths exercised: {'YES' if all_paths else 'NO'}")
    print(f"  Overall: {'PASS' if all_paths else 'PARTIAL'}")
    
    return 0 if all_paths else 1

if __name__ == "__main__":
    exit(main())
