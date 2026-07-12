---
title: blindsight — Knowledgebase
project_note: knowledgebase
type: project-note
tags: ["project-note","knowledgebase"]
status: active
created: 2026-07-09
updated: 2026-07-09
source: "https://github.com/jxoesneon/blindsight"
---

# blindsight — Knowledgebase

Synthesized expansion from the read-only subagent exploration of the local clone.

## Summary

Blindsight is a privacy-first survey platform (homepage: [blindsightsurveys.com](https://www.blindsightsurveys.com)). The stack is FastAPI (Python 3.13) for the backend and an Nx monorepo with React 19 + Vite 7 + Tailwind 4 for five SPAs. It supports anonymous surveys, email distribution, real-time analytics, GDPR/CCPA data export/deletion, DLP, a full mail client, training/exams with certificates, LTI 1.3 integration, and an admin panel.

## Local clone

| Field | Value |
|-------|-------|
| GitHub | `jxoesneon/blindsight` |
| Local path | `C:/Users/josee/blindsight` |
| Version | 2.0.0 |
| Visibility | PRIVATE |
| License | None |
| Homepage | https://www.blindsightsurveys.com |

## Top-level structure

- `pyproject.toml` — Python 3.13, FastAPI, Pydantic, Motor, Redis; build system hatchling.
- `README.md`, `AGENTS.md`, `CODEMAP.md`, `DESIGN_SYSTEM.md`.
- `Dockerfile` — multi-stage Node 20 → Python 3.13, non-root user.
- `docker-compose.yml` / `docker-compose.prod.yml`.
- `.env.sample`, `.github/workflows/`.
- `src/` — FastAPI backend with 45+ routers and services.
- `web/` — Nx monorepo with 5 SPAs.
- `tests/`, `scripts/`, `Personas/`, `monitoring/`, `docs/`.

## Architecture

### Backend (FastAPI)

- `src/main.py` — lifespan (DB indexes, Redis subscriber, email dispatcher, webhook purge) and middleware stack.
- Middleware (outer → inner): CORS → request logging → security headers (CSP/HSTS) → global error handler → rate limiting → auth rate limit → session idle timeout → request size limit → recovery enforcement.
- `src/api/v1/api.py` — 45+ domain routers mounted under `/api/v1` with global CSRF.
- Routers: auth, surveys, responses, emails, mail, privacy, DLP, admin, exams, certificates, webhooks, subscriptions, LTI, etc.
- `src/services/` — 45+ service classes.
- `src/infrastructure/mongo/` — 30+ repository classes (Motor async).
- `src/domain/models/` — 30+ Pydantic models.
- `src/core/config.py` — env validation (SECRET_KEY, CORS, MongoDB, Redis, AWS SES, etc.).

### Frontend (Nx + React)

5 SPAs:

| SPA | Path | Dev port | Purpose |
|-----|------|----------|---------|
| Dashboard | `web/apps/dashboard` | 4200 | Marketing + authenticated app |
| Survey Renderer | `web/apps/survey-renderer` | 4203 | Public participant view |
| Mail Client | `web/apps/mail-client` | 4201 | Authenticated mail |
| Blindsight Admin | `web/apps/blindsight-admin` | 4202 | Internal admin panel |
| Marketing Site | `web/apps/marketing-site` | — | Phase 2 scaffold |

Design system: Enterprise Glassmorphism (deep dark substrate, blur/layered depth, indigo brand).

## Build / test / deploy

Backend:

```bash
uv sync
py -m uvicorn src.main:app --host 0.0.0.0 --port 8000 --reload
uv run pytest tests/ -v --cov=src --cov-report=xml --cov-fail-under=90
ruff check src/; black src/; mypy src/
```

Frontend:

```bash
cd web
npm install
npx nx dev @web/dashboard
npx nx run-many -t build --projects=@web/dashboard,@web/survey-renderer,@web/mail-client,@web/blindsight-admin,@web/marketing-site --parallel=3
npx tsc --noEmit --skipLibCheck
npx nx run-many -t lint --projects=... --parallel=3
npx nx run-many -t test --projects=... --parallel=3
```

Docker:

```bash
docker-compose up -d mongodb redis
docker-compose up -d
# production:
docker-compose -f docker-compose.prod.yml up --build
```

CI (`ci.yml`): verify-install, web checks, pytest with 90% coverage, Snyk security scan. `deploy.yml` builds GHCR image.

## Recent git state (manual snapshot)

- **Latest version:** 2.0.0 (Python target 3.13).
- **Working tree:** appears clean in the read-only snapshot; recent commits focused on hardening and test coverage.
- **Recent commits:**
  - `9bc7d4a8` chore(gitignore): ignore temp scripts, test outputs, build artifacts
  - `6e552f04` test: add and update backend test coverage across routers, services, infrastructure
  - `81e58511` feat(backend): add and update API routes, services, domain models, and infrastructure
  - `b9038092` feat(frontend): update dashboard, admin, mail-client, marketing-site, and survey-renderer SPAs
  - `940f2276` docs(project): add guides, API docs, audits, ops docs, and design system updates

## Recent resolved issues (from AGENTS.md)

- Flask dependency stack removed (2026-06-09).
- SSRF protection added to webhook URLs; CSP switched to nonce-based; session Secure flag hardened.
- Structlog JSON logging wired; MongoDB connection pooling configured.
- 303+ color contrast violations fixed; admin pagination DoS fix; auth input validation hardened.
- Rate limit middlewares activated; SlowAPI proxy-aware; CORS wildcard guard; session idle timeout logging.

## Compliance targets

- GDPR Articles 30/32
- CCPA/CPRA
- SOC2 CC6.1/CC7.2
- ISO 27001 A.12.4
- NIST 800-53 AU-6/AU-9

## Key files for deeper context

1. `AGENTS.md` — quick commands, structure, known issues, verification.
2. `CODEMAP.md` — architecture and directory map.
3. `src/main.py` — entry point and middleware stack.
4. `src/core/config.py` — environment and config.
5. `src/api/v1/api.py` — router aggregation.
6. `pyproject.toml` — dependencies and tooling.
7. `docker-compose.yml` / `docker-compose.prod.yml`.
8. `web/apps/dashboard/src/app/app.tsx` — main dashboard SPA.
9. `Personas/README.md` — persona mapping and workflows.
10. `DESIGN_SYSTEM.md` — Enterprise Glassmorphism design system.

## Related

- [[ciel/projects/blindsight/blindsight.md|blindsight overview]]
- [[ciel/projects.md|Projects index]]
