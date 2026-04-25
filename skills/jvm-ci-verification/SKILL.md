---
name: jvm-ci-verification
version: 1.0.0
format: skill/1.0
description: The formal verification loop for JVM (Java/Kotlin) projects using Maven or Gradle.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(verify|check|test|lint).*(maven|gradle|mvn|jvm)"
    confidence: 0.9
  - pattern: "npx verify-jvm"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: JVM CI Verification (The Gate)

This skill dictates the "Pre-PR" verification protocol for all Java and Kotlin projects. It ensures that only high-quality, verified code enters the shared repository.

## The Verification Loop

### Phase 1: Clean Build
- **Maven**: `mvn clean verify -DskipTests` (Parallel: `-T 4`).
- **Gradle**: `./gradlew clean assemble -x test`.

### Phase 2: Quality Analysis
- **Lint**: `spotless:check` or `detekt`.
- **Static**: `spotbugs` or `pmd`.
- **Rule**: If static analysis fails, the loop terminates immediately.

### Phase 3: Tests & Coverage
- **Threshold**: Hard gate at **80% line coverage** (verified via JaCoCo/Kover).
- **Command**: `mvn test` or `./gradlew koverVerify`.

### Phase 4: Security Scan
- **Dependencies**: `org.owasp:dependency-check` to find CVEs.
- **Secrets**: `grep` check for `sk-`, `api_key`, or `password` patterns in `src/`.

## Report Generation
The Orchestrator MUST produce a `VERIFICATION_REPORT.md` before finalizing a branch:
- **Build**: PASS/FAIL.
- **Analysis**: Tool output summaries.
- **Coverage**: % and delta from baseline.
- **Security**: CVE count.

## Anti-Patterns
- **Test-Skipping**: Running `mvn install -DskipTests` to bypass the quality gate.
- **Local-Only H2**: Passing tests on H2 but ignoring production Postgres specificities.
- **Unpinned Analysis**: Using "LATEST" versions of static analysis plugins that produce inconsistent results.
