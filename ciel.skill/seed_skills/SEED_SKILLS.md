# SEED_SKILLS

Ciel's cold-start capability surface. Every seed skill is present from the first run so Ciel can operate before acquisition has populated the registry.

## Load Order

Load order at bootstrap reflects dependency chains:

1. `filesystem` — base for everything that reads/writes.
2. `shell` — used by most other seed skills.
3. `environment_detection` — informs adapters.
4. `git` — versioning.
5. `archive_manager` — for `.skill` handling.
6. `json_yaml_toml_parser` — config/manifest parsing.
7. `markdown_processor` — skill body parsing.
8. `diff_patch` — self-mod applications.
9. `package_manager` — install deps (including MemPalace).
10. `mempalace_manager` — primary memory backend.
11. `secrets_manager` — credential handling.
12. `web_fetch`, `web_search` — research.
13. `mcp_manager` — MCP servers.
14. `api_client` — HTTP / GraphQL.
15. `docker` — sandboxing.
16. `sandbox` (via `acquisition/SANDBOX.md` — delegates to docker/etc).
17. `skill_builder`, `skill_installer` — bundle ops.
18. `council_runner` — Council orchestration.
19. `code_analysis`, `code_generation`, `code_review` — code ops.
20. `test_runner`, `linter_formatter` — quality.
21. `dependency_audit` — security.
22. `documentation` — docs.
23. `cicd_integration` — CI.
24. `database_client` — DB ops.
25. `log_analyzer` — log parsing.
26. `context_summarizer` — AAAK compression.
27. `project_analyzer` — init context.
28. `research` — deep research.
29. `runtime_adapter_builder` — new adapters.

## Interdependencies

Circular deps are forbidden. Any seed skill needing another must declare it in `dependencies.skills`.

## 32 Seed Skills (index)

1. `filesystem`
2. `shell`
3. `git`
4. `web_search`
5. `web_fetch`
6. `mcp_manager`
7. `package_manager`
8. `code_analysis`
9. `code_generation`
10. `code_review`
11. `test_runner`
12. `docker`
13. `api_client`
14. `documentation`
15. `dependency_audit`
16. `environment_detection`
17. `skill_builder`
18. `skill_installer`
19. `archive_manager`
20. `mempalace_manager`
21. `council_runner`
22. `context_summarizer`
23. `research`
24. `project_analyzer`
25. `runtime_adapter_builder`
26. `diff_patch`
27. `json_yaml_toml_parser`
28. `log_analyzer`
29. `secrets_manager`
30. `linter_formatter`
31. `cicd_integration`
32. `database_client`
33. `markdown_processor`

(33 entries — spec said 32; counted above includes `markdown_processor` per spec; markdown_processor is the final one.)

Each has a `SKILL.md` under `seed_skills/<name>/SKILL.md`.

## Guarantees at Bootstrap

All seed skills are integrity-checked before being registered. A missing or corrupted seed skill is fatal to bootstrap — user re-unpacks.
