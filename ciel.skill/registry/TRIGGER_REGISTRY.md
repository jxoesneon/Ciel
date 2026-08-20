# TRIGGER_REGISTRY

| Trigger | Target ID | Confidence |
| --- | --- | --- |
| `(compare\|benchmark\|evaluate).*(agent\|aider\|claude code\|gemini\|codex)` | `agent-eval/SKILL.md` | 0.9 |
| `which (agent\|model) is better for` | `agent-eval/SKILL.md` | 0.9 |
| `(create\|design\|build\|optimize).*(agent\|harness\|action space\|tool definition)` | `agent-harness-construction/SKILL.md` | 0.9 |
| `how should I (structure\|format) (tools\|observations)` | `agent-harness-construction/SKILL.md` | 0.9 |
| `(agent\|self).*(debugging\|introspection\|looping\|stuck)` | `agent-introspection-debugging/SKILL.md` | 0.9 |
| `why is the agent (failing\|repeating\|drifting)` | `agent-introspection-debugging/SKILL.md` | 0.9 |
| `(agent\|autonomous).*(payment\|pay\|wallet\|spend\|budget)` | `agent-payment-x402/SKILL.md` | 0.9 |
| `x402` | `agent-payment-x402/SKILL.md` | 0.9 |
| `(retrieve\|gather\|refine).*(context\|codebase\|file)` | `agent-retrieval-and-context/SKILL.md` | 0.9 |
| `iterative retrieval` | `agent-retrieval-and-context/SKILL.md` | 0.9 |
| `(sort\|trim\|curate).*skills` | `agent-sort/SKILL.md` | 0.9 |
| `remove.*unnecessary.*(skills\|rules)` | `agent-sort/SKILL.md` | 0.9 |
| `DAILY vs LIBRARY` | `agent-sort/SKILL.md` | 0.9 |
| `optimize.*workspace.*context` | `agent-sort/SKILL.md` | 0.9 |
| `(orchestrate\|harness\|manage).*(agent\|fleet\|gan\|gsd\|headless)` | `agent-systems-and-harnesses/SKILL.md` | 0.9 |
| `generator-evaluator loop` | `agent-systems-and-harnesses/SKILL.md` | 0.9 |
| `(agentic\|autonomous\|agent-led).*engineering` | `agentic-engineering/SKILL.md` | 0.9 |
| `route.*model.*tier` | `agentic-engineering/SKILL.md` | 0.9 |
| `eval-first.*` | `agentic-engineering/SKILL.md` | 0.9 |
| `task.*decomposition` | `agentic-engineering/SKILL.md` | 0.9 |
| `ai-first.*engineering` | `ai-first-engineering/SKILL.md` | 0.9 |
| `agent-friendly.*architecture` | `ai-first-engineering/SKILL.md` | 0.9 |
| `raise.*testing.*bar` | `ai-first-engineering/SKILL.md` | 0.9 |
| `process.*shift` | `ai-first-engineering/SKILL.md` | 0.9 |
| `ai-regression.*testing` | `ai-regression-testing/SKILL.md` | 0.9 |
| `catch.*blind.*spot` | `ai-regression-testing/SKILL.md` | 0.9 |
| `sandbox.*production.*parity` | `ai-regression-testing/SKILL.md` | 0.9 |
| `bug-check.*` | `ai-regression-testing/SKILL.md` | 0.9 |
| `(design\|optimize\|query).*(clickhouse\|olap\|analytics\|mergetree)` | `analytical-data-ops/SKILL.md` | 0.9 |
| `materialized view` | `analytical-data-ops/SKILL.md` | 0.9 |
| `(structure\|setup\|architect).*(android\|kmp\|kotlin multiplatform).*(project\|module)` | `android-kmp-architecture/SKILL.md` | 0.9 |
| `android clean architecture` | `android-kmp-architecture/SKILL.md` | 0.9 |
| `(gsap\|animation).*(react\|vue\|svelte\|mounted\|cleanup)` | `animation-framework-integration/SKILL.md` | 0.9 |
| `useGSAP` | `animation-framework-integration/SKILL.md` | 0.9 |
| `(optimize\|speed up).*(animation\|gsap\|fps\|60fps)` | `animation-performance-utils/SKILL.md` | 0.9 |
| `gsap.utils` | `animation-performance-utils/SKILL.md` | 0.9 |
| `(design\|build\|architect).*(api\|backend\|endpoint\|rest)` | `api-backend-development/SKILL.md` | 0.9 |
| `backend patterns` | `api-backend-development/SKILL.md` | 0.9 |
| `(record\|ADR).*decision` | `architecture-decision-records/SKILL.md` | 0.9 |
| `why did we.*(choose\|use\|select)` | `architecture-decision-records/SKILL.md` | 0.9 |
| `architectural.*trade-off` | `architecture-decision-records/SKILL.md` | 0.9 |
| `consequences.*of.*(pattern\|framework)` | `architecture-decision-records/SKILL.md` | 0.9 |
| `(write\|draft\|polished).*(article\|blog\|essay\|newsletter\|tutorial)` | `article-writing/SKILL.md` | 0.9 |
| `write a guide for` | `article-writing/SKILL.md` | 0.9 |
| `(create\|design\|generate).*(logo\|banner\|icon\|header\|card)` | `asset-design/SKILL.md` | 0.9 |
| `design a (facebook\|twitter\|linkedin) cover` | `asset-design/SKILL.md` | 0.9 |
| `(audit\|inventory\|list).*(automation\|job\|workflow\|hook\|connector)` | `automation-audit-ops/SKILL.md` | 0.9 |
| `what (automations\|hooks) are (live\|broken)` | `automation-audit-ops/SKILL.md` | 0.9 |
| `(setup\|run\|create).*(autonomous\|continuous).*(loop\|pipeline\|workflow)` | `autonomous-loops/SKILL.md` | 0.9 |
| `orchestrate (multiple\|parallel) (agents\|tasks)` | `autonomous-loops/SKILL.md` | 0.9 |
| `(orchestrate\|dispatch\|schedule).*(agent\|fleet\|cron\|mission)` | `autonomous-orchestration/SKILL.md` | 0.9 |
| `multi-agent workflow` | `autonomous-orchestration/SKILL.md` | 0.9 |
| `(audit\|analyze\|remediate).*(billing\|burn\|cost\|revenue\|stripe)` | `billing-and-burn-ops/SKILL.md` | 0.9 |
| `ecc tools cost audit` | `billing-and-burn-ops/SKILL.md` | 0.9 |
| `(billing\|refund\|dashboard\|monitor).*(stripe\|grafana\|signoz\|customer)` | `billing-and-monitoring/SKILL.md` | 0.9 |
| `customer billing ops` | `billing-and-monitoring/SKILL.md` | 0.9 |
| `(brainstorm\|design\|spec\|requirements).*(for\|new\|feature)` | `brainstorming/SKILL.md` | 0.9 |
| `how should we.*(build\|implement\|architecture)` | `brainstorming/SKILL.md` | 0.9 |
| `let's explore.*(idea\|concept\|approach)` | `brainstorming/SKILL.md` | 0.9 |
| `I want to.*(add\|create).*(but\|not sure)` | `brainstorming/SKILL.md` | 0.9 |
| `(define\|update\|audit).*(brand\|voice\|identity\|tone)` | `brand-identity/SKILL.md` | 0.9 |
| `brand guidelines` | `brand-identity/SKILL.md` | 0.9 |
| `(create\|store\|archive\|save).*(artifact\|plan\|audit\|transient)` | `ciel-artifact-management/SKILL.md` | 0.9 |
| `planning mode` | `ciel-artifact-management/SKILL.md` | 0.9 |
| `(generate\|update\|validate).*(docs\|documentation\|readme)` | `ciel-documentation-engine/SKILL.md` | 0.9 |
| `semantic indexing` | `ciel-documentation-engine/SKILL.md` | 0.9 |
| `(build\|review\|enforce).*(react\|hook\|pattern\|rule\|hookify)` | `ciel-frontend-and-hooks/SKILL.md` | 0.9 |
| `hookify rule` | `ciel-frontend-and-hooks/SKILL.md` | 0.9 |
| `(escalate\|ask\|approve\|confirm\|decide).*(user\|host\|human)` | `ciel-hitl-protocol/SKILL.md` | 0.9 |
| `high risk operation` | `ciel-hitl-protocol/SKILL.md` | 0.9 |
| `(configure\|manage\|split).*(ciel\|cost\|dmux\|parallel\|ecc)` | `ciel-internal-ops/SKILL.md` | 0.9 |
| `configure-ecc` | `ciel-internal-ops/SKILL.md` | 0.9 |
| `(save\|search\|sync\|explore).*(knowledge\|memory\|session\|structure\|ast)` | `ciel-knowledge-and-memory/SKILL.md` | 0.9 |
| `smart explore` | `ciel-knowledge-and-memory/SKILL.md` | 0.9 |
| `(manage\|audit\|configure).*(skill\|worktree\|workspace\|visa)` | `ciel-meta-and-utility/SKILL.md` | 0.9 |
| `using git worktrees` | `ciel-meta-and-utility/SKILL.md` | 0.9 |
| `(audit\|configure\|call).*(ui\|state\|api\|ciel\|ecc)` | `ciel-meta-operations/SKILL.md` | 0.9 |
| `click path audit` | `ciel-meta-operations/SKILL.md` | 0.9 |
| `(design\|build\|optimize).*(pytorch\|tensor\|postgres\|sql\|ml)` | `ciel-ml-and-data-patterns/SKILL.md` | 0.9 |
| `device-agnostic code` | `ciel-ml-and-data-patterns/SKILL.md` | 0.9 |
| `(route\|classify\|orchestrate\|build team).*(task\|domain\|agent)` | `ciel-orchestration-and-routing/SKILL.md` | 0.9 |
| `master router` | `ciel-orchestration-and-routing/SKILL.md` | 0.9 |
| `(scaffold\|create\|init\|new).*(project\|module\|service\|boilerplate)` | `ciel-project-scaffolder/SKILL.md` | 0.9 |
| `standardize project` | `ciel-project-scaffolder/SKILL.md` | 0.9 |
| `(verify\|check\|test\|lint).*(completion\|evidence\|tdd\|terminal)` | `ciel-quality-and-verification/SKILL.md` | 0.9 |
| `verification before completion` | `ciel-quality-and-verification/SKILL.md` | 0.9 |
| `(review\|audit\|adversarial\|challenge\|critique).*(code\|pr\|security\|design)` | `ciel-review-and-adversarial/SKILL.md` | 0.9 |
| `request.*review` | `ciel-review-and-adversarial/SKILL.md` | 0.9 |
| `(debug\|fix\|trace\|error\|crash\|fail).*(root cause\|log\|stack)` | `ciel-root-cause-debugger/SKILL.md` | 0.9 |
| `analyze logs` | `ciel-root-cause-debugger/SKILL.md` | 0.9 |
| `(audit\|design\|style).*(solidity\|amm\|design system\|token\|css)` | `ciel-security-and-design/SKILL.md` | 0.9 |
| `design system` | `ciel-security-and-design/SKILL.md` | 0.9 |
| `(create\|edit\|review).*(skill\|persona\|soul\|reproduce)` | `ciel-skill-engineering/SKILL.md` | 0.9 |
| `node init_skill.cjs` | `ciel-skill-engineering/SKILL.md` | 0.9 |
| `(swarm\|parallel\|coordinate\|distribute\|decompose).*(agents\|tasks\|workers)` | `ciel-swarm-orchestration/SKILL.md` | 0.9 |
| `invoke subagents` | `ciel-swarm-orchestration/SKILL.md` | 0.9 |
| `(design\|build\|review).*(swift\|ios\|concurrency\|actor\|glass)` | `ciel-swift-and-ios-standards/SKILL.md` | 0.9 |
| `approachable concurrency` | `ciel-swift-and-ios-standards/SKILL.md` | 0.9 |
| `(onboard\|walk through\|explain\|understand).*(codebase\|repo\|project)` | `codebase-onboarding/SKILL.md` | 0.9 |
| `generate (claude.md\|starter config)` | `codebase-onboarding/SKILL.md` | 0.9 |
| `(code\|coding).*(standard\|convention\|quality)` | `coding-standards/SKILL.md` | 0.9 |
| `how should I.*(write\|structure\|format).*(code\|function\|component)` | `coding-standards/SKILL.md` | 0.9 |
| `(review\|refactor).*(code\|file).*(quality\|readability)` | `coding-standards/SKILL.md` | 0.9 |
| `(build\|design).*(compose multiplatform\|jetpack compose).*(ui\|interface)` | `compose-multiplatform/SKILL.md` | 0.9 |
| `compose multiplatform patterns` | `compose-multiplatform/SKILL.md` | 0.9 |
| `(deploy\|docker\|ci/cd\|k8s).*(container\|rolling\|canary\|github action)` | `container-and-deployment/SKILL.md` | 0.9 |
| `deployment patterns` | `container-and-deployment/SKILL.md` | 0.9 |
| `(post\|tweet\|distribute\|optimize).*(social\|x\|linkedin\|threads\|bluesky)` | `content-distribution-ops/SKILL.md` | 0.9 |
| `content engine` | `content-distribution-ops/SKILL.md` | 0.9 |
| `cache (expensive\|file\|processing\|pdf\|ocr\|parsing)` | `content-hash-cache-pattern/SKILL.md` | 0.9 |
| `content-hash caching` | `content-hash-cache-pattern/SKILL.md` | 0.9 |
| `(save\|resume\|model\|tour).*(context\|voice\|memory\|walkthrough)` | `context-and-memory-ops/SKILL.md` | 0.9 |
| `context keeper` | `context-and-memory-ops/SKILL.md` | 0.9 |
| `(check\|audit\|view).*(context\|token).*(budget\|usage\|overhead)` | `context-budget/SKILL.md` | 0.9 |
| `/context-budget` | `context-budget/SKILL.md` | 0.9 |
| `(start\|run\|select).*(loop\|pattern\|stack)` | `continuous-agent-loop/SKILL.md` | 0.9 |
| `how should I (run\|execute) this (autonomous\|continuous) task` | `continuous-agent-loop/SKILL.md` | 0.9 |
| `(instinct\|learning\|evolution).*(status\|evolve\|promote)` | `continuous-learning-v2/SKILL.md` | 0.9 |
| `how is (ciel\|the agent) evolving` | `continuous-learning-v2/SKILL.md` | 0.9 |
| `convene.*council` | `council/SKILL.md` | 0.9 |
| `council.*deliberation` | `council/SKILL.md` | 0.9 |
| `evaluate.*(decision\|choice\|path)` | `council/SKILL.md` | 0.9 |
| `second opinion` | `council/SKILL.md` | 0.9 |
| `(design\|build\|review).*(cpp\|c\+\+\|cmake\|gtest)` | `cpp-development/SKILL.md` | 0.9 |
| `cpp core guidelines` | `cpp-development/SKILL.md` | 0.9 |
| `(build\|secure\|audit).*(trading\|bot\|wallet\|solidity\|transaction)` | `crypto-and-trading-security/SKILL.md` | 0.9 |
| `spend limit guard` | `crypto-and-trading-security/SKILL.md` | 0.9 |
| `(classify\|export\|import\|customs).*(hs code\|tariff\|compliance\|border)` | `customs-compliance/SKILL.md` | 0.9 |
| `denied party screening` | `customs-compliance/SKILL.md` | 0.9 |
| `execute.*plan` | `do/SKILL.md` | 0.9 |
| `do.*(implementation\|tasks)` | `do/SKILL.md` | 0.9 |
| `start.*(phase\|execution)` | `do/SKILL.md` | 0.9 |
| `(create\|alter\|migrate\|update).*(table\|schema\|database\|column)` | `database-migrations/SKILL.md` | 0.9 |
| `database migrations` | `database-migrations/SKILL.md` | 0.9 |
| `(dispatch\|run\|execute).*(parallel\|concurrent).*(agents\|tasks)` | `dispatching-parallel-agents/SKILL.md` | 0.9 |
| `(multiple\|several) (independent\|unrelated) (failures\|bugs\|tasks)` | `dispatching-parallel-agents/SKILL.md` | 0.9 |
| `(design\|build\|review).*(django\|drf\|manage.py\|python web)` | `django-development/SKILL.md` | 0.9 |
| `django patterns` | `django-development/SKILL.md` | 0.9 |
| `(process\|ocr\|convert\|animate).*(pdf\|docx\|nutrient\|manim\|explainer)` | `document-and-video-intelligence/SKILL.md` | 0.9 |
| `technical explainer` | `document-and-video-intelligence/SKILL.md` | 0.9 |
| `(design\|build\|review).*(dotnet\|.net\|c#\|csharp)` | `dotnet-development/SKILL.md` | 0.9 |
| `dotnet patterns` | `dotnet-development/SKILL.md` | 0.9 |
| `(test\|design\|build).*(e2e\|playwright\|frontend\|ui\|visual)` | `e2e-and-visual-verification/SKILL.md` | 0.9 |
| `page object model` | `e2e-and-visual-verification/SKILL.md` | 0.9 |
| `(read\|send\|triage\|draft).*(email\|mail\|inbox)` | `email-ops/SKILL.md` | 0.9 |
| `reply to (the\|this) email` | `email-ops/SKILL.md` | 0.9 |
| `(procure\|analyze\|negotiate).*(energy\|electricity\|gas\|ppa\|utility)` | `energy-procurement/SKILL.md` | 0.9 |
| `demand charge mitigation` | `energy-procurement/SKILL.md` | 0.9 |
| `(evaluate\|benchmark\|test).*(agent\|model\|workflow)` | `eval-harness/SKILL.md` | 0.9 |
| `setup (eval\|edd\|eval-driven)` | `eval-harness/SKILL.md` | 0.9 |
| `(convention\|style\|standard\|commit).*(rule\|guide\|pattern)` | `everything-claude-code-conventions/SKILL.md` | 0.9 |
| `how should I.*(name\|format\|structure)` | `everything-claude-code-conventions/SKILL.md` | 0.9 |
| `what are the.*(repo\|project).*standards` | `everything-claude-code-conventions/SKILL.md` | 0.9 |
| `(start\|begin\|load).*(plan\|implementation\|epic)` | `executing-plans/SKILL.md` | 0.9 |
| `execute (the\|this) plan` | `executing-plans/SKILL.md` | 0.9 |
| `(find\|search\|lookup\|is there).*skill` | `find-skills/SKILL.md` | 0.9 |
| `how do I.*(react\|testing\|design\|deploy)` | `find-skills/SKILL.md` | 0.9 |
| `can you do.*(specialized\|expert)` | `find-skills/SKILL.md` | 0.9 |
| `npx skills.*` | `find-skills/SKILL.md` | 0.9 |
| `(finish\|complete).*(branch\|epic\|development\|feature)` | `finishing-a-development-branch/SKILL.md` | 0.9 |
| `ready to (merge\|pr\|push)` | `finishing-a-development-branch/SKILL.md` | 0.9 |
| `(build\|review\|refactor).*(flutter\|dart).*(feature\|widget\|code)` | `flutter-development/SKILL.md` | 0.9 |
| `flutter patterns` | `flutter-development/SKILL.md` | 0.9 |
| `(design\|build\|review).*(go\|golang\|goroutine)` | `go-development/SKILL.md` | 0.9 |
| `go test` | `go-development/SKILL.md` | 0.9 |
| `(manage\|edit\|clean\|summarize\|gws).*(google\|drive\|gdoc\|gsheet\|gmail\|calendar)` | `google-workspace-management/SKILL.md` | 0.9 |
| `gws auth setup` | `google-workspace-management/SKILL.md` | 0.9 |
| `(animate\|build).*(gsap\|timeline\|tween\|ease\|svg\|morph)` | `gsap-animation-suite/SKILL.md` | 0.9 |
| `gsap.to` | `gsap-animation-suite/SKILL.md` | 0.9 |
| `(healthcare\|hipaa\|phi\|pii\|baa).*(compliance\|patient\|clinician)` | `healthcare-compliance/SKILL.md` | 0.9 |
| `is this (healthcare\|hipaa) compliant` | `healthcare-compliance/SKILL.md` | 0.9 |
| `(build\|design).*(emr\|ehr\|patient encounter\|cdss\|interactions\|dosing)` | `healthcare-systems/SKILL.md` | 0.9 |
| `healthcare safety patterns` | `healthcare-systems/SKILL.md` | 0.9 |
| `(fundraise\|investor).*(outreach\|email\|pitch\|deck\|memo\|model)` | `investor-relations/SKILL.md` | 0.9 |
| `reply to (vc\|investor)` | `investor-relations/SKILL.md` | 0.9 |
| `(design\|build\|review).*(java\|spring boot\|springboot)` | `java-springboot-development/SKILL.md` | 0.9 |
| `spring security` | `java-springboot-development/SKILL.md` | 0.9 |
| `(jira\|atlassian).*(ticket\|issue\|sprint\|jql)` | `jira-integration/SKILL.md` | 0.9 |
| `get (requirements\|ac) from (ticket\|jira)` | `jira-integration/SKILL.md` | 0.9 |
| `(verify\|check\|test\|lint).*(maven\|gradle\|mvn\|jvm)` | `jvm-ci-verification/SKILL.md` | 0.9 |
| `npx verify-jvm` | `jvm-ci-verification/SKILL.md` | 0.9 |
| `(build\|setup).*(ktor\|exposed\|hikari\|flyway)` | `jvm-server-frameworks/SKILL.md` | 0.9 |
| `ktor patterns` | `jvm-server-frameworks/SKILL.md` | 0.9 |
| `(design\|build\|review).*(kotlin\|coroutine\|flow)` | `kotlin-development/SKILL.md` | 0.9 |
| `idiomatic kotlin` | `kotlin-development/SKILL.md` | 0.9 |
| `(design\|build\|review).*(laravel\|php\|artisan)` | `laravel-development/SKILL.md` | 0.9 |
| `laravel patterns` | `laravel-development/SKILL.md` | 0.9 |
| `(manage\|negotiate\|resolve).*(carrier\|freight\|shipment\|delay\|claim)` | `logistics-management/SKILL.md` | 0.9 |
| `freight exceptions` | `logistics-management/SKILL.md` | 0.9 |
| `make.*plan.*(for\|implement\|build)` | `make-plan/SKILL.md` | 0.9 |
| `how should we.*(execute\|start).*(implementation\|this)` | `make-plan/SKILL.md` | 0.9 |
| `create.*implementation.*steps` | `make-plan/SKILL.md` | 0.9 |
| `(research\|analyze).*(market\|competitor\|investor\|category\|trend)` | `market-research/SKILL.md` | 0.9 |
| `tam/sam/som` | `market-research/SKILL.md` | 0.9 |
| `(build\|create\|add).*mcp.*(server\|tool\|resource)` | `mcp-server-patterns/SKILL.md` | 0.9 |
| `mcp.*integration` | `mcp-server-patterns/SKILL.md` | 0.9 |
| `@modelcontextprotocol/sdk` | `mcp-server-patterns/SKILL.md` | 0.9 |
| `(remember\|save\|persist\|store).*memory` | `mempalace-rs/SKILL.md` | 0.9 |
| `what did we.*(last\|previous\|before)` | `mempalace-rs/SKILL.md` | 0.9 |
| `search.*(history\|memory\|palace)` | `mempalace-rs/SKILL.md` | 0.9 |
| `knowledge graph\|AAAK\|temporal fact` | `mempalace-rs/SKILL.md` | 0.9 |
| `(read\|check).*(message\|text\|dm\|code\|otp)` | `messages-ops/SKILL.md` | 0.9 |
| `look in (imessage\|twitter\|x) dms` | `messages-ops/SKILL.md` | 0.9 |
| `(build\|setup\|run).*(bun\|runtime\|hashing\|keccak)` | `modern-js-runtimes/SKILL.md` | 0.9 |
| `bun install` | `modern-js-runtimes/SKILL.md` | 0.9 |
| `(design\|build\|review).*(nest.js\|nestjs\|decorator\|provider)` | `nestjs-backend-development/SKILL.md` | 0.9 |
| `nest generate` | `nestjs-backend-development/SKILL.md` | 0.9 |
| `(search\|generate).*(exa\|neural\|fal.ai\|image\|video\|audio)` | `neural-intelligence-generation/SKILL.md` | 0.9 |
| `exa web search` | `neural-intelligence-generation/SKILL.md` | 0.9 |
| `(design\|build\|review).*(next.js\|nextjs\|turbopack\|app router)` | `nextjs-development/SKILL.md` | 0.9 |
| `next dev --turbopack` | `nextjs-development/SKILL.md` | 0.9 |
| `(design\|build\|review).*(nuxt\|vue\|nitro\|hydration)` | `nuxt-development/SKILL.md` | 0.9 |
| `nuxt route rules` | `nuxt-development/SKILL.md` | 0.9 |
| `(open source\|sanitize\|scan\|audit).*(repo\|project\|github\|secret)` | `opensource-and-repo-ops/SKILL.md` | 0.9 |
| `repo scan html report` | `opensource-and-repo-ops/SKILL.md` | 0.9 |
| `(plan\|forecast\|schedule).*(production\|demand\|inventory\|bottleneck)` | `operations-planning/SKILL.md` | 0.9 |
| `drum-buffer-rope` | `operations-planning/SKILL.md` | 0.9 |
| `(blueprint\|plan\|roadmap).*for` | `orchestration/SKILL.md` | 0.9 |
| `orchestrate.*` | `orchestration/SKILL.md` | 0.9 |
| `break down.*into steps` | `orchestration/SKILL.md` | 0.9 |
| `complex.*task` | `orchestration/SKILL.md` | 0.9 |
| `(benchmark\|qa\|watch\|monitor).*(performance\|lcp\|regression\|deploy)` | `performance-and-qa-ops/SKILL.md` | 0.9 |
| `canary watch` | `performance-and-qa-ops/SKILL.md` | 0.9 |
| `(design\|build\|review).*(perl\|cpan\|cpanfile)` | `perl-development/SKILL.md` | 0.9 |
| `modern perl` | `perl-development/SKILL.md` | 0.9 |
| `(create\|build).*(presentation\|deck\|slides\|pitch)` | `presentation-design/SKILL.md` | 0.9 |
| `convert (pptx\|powerpoint) to (html\|slides)` | `presentation-design/SKILL.md` | 0.9 |
| `(create\|define\|translate).*(capability\|contract\|constraints)` | `product-capability/SKILL.md` | 0.9 |
| `turn (prd\|intent) into (srs\|contract)` | `product-capability/SKILL.md` | 0.9 |
| `(analyze\|diagnose\|review).*(product\|idea\|founder lens)` | `product-lens/SKILL.md` | 0.9 |
| `should I build (this\|that)` | `product-lens/SKILL.md` | 0.9 |
| `(triage\|backlog\|audit).*(issue\|pr\|backlog)` | `project-flow-ops/SKILL.md` | 0.9 |
| `map (github\|issues) to (linear\|tasks)` | `project-flow-ops/SKILL.md` | 0.9 |
| `(optimize\|rewrite\|branch\|compact).*(prompt\|session\|repl)` | `prompt-and-session-engineering/SKILL.md` | 0.9 |
| `prompt optimizer` | `prompt-and-session-engineering/SKILL.md` | 0.9 |
| `(design\|build\|review).*(python\|pip\|pytest\|type hint)` | `python-development/SKILL.md` | 0.9 |
| `idiomatic python` | `python-development/SKILL.md` | 0.9 |
| `(investigate\|process).*(non-conformance\|ncr\|capa\|return\|refund)` | `quality-and-returns/SKILL.md` | 0.9 |
| `root cause analysis` | `quality-and-returns/SKILL.md` | 0.9 |
| `parse (quiz\|form\|invoice\|structured text)` | `regex-vs-llm-structured-text/SKILL.md` | 0.9 |
| `(regex vs llm\|regex or llm)` | `regex-vs-llm-structured-text/SKILL.md` | 0.9 |
| `(create\|build\|code).*(video\|animation).*(remotion)` | `remotion-video-creation/SKILL.md` | 0.9 |
| `remotion` | `remotion-video-creation/SKILL.md` | 0.9 |
| `(request\|get\|perform).*(code review\|audit\|peer review)` | `requesting-code-review/SKILL.md` | 0.9 |
| `review (my\|the) code` | `requesting-code-review/SKILL.md` | 0.9 |
| `(research\|scrape\|lookup).*(deep dive\|exa\|firecrawl\|documentation)` | `research-and-intelligence/SKILL.md` | 0.9 |
| `deep research report` | `research-and-intelligence/SKILL.md` | 0.9 |
| `research.*` | `research-ops/SKILL.md` | 0.9 |
| `(look up\|search for\|find).*latest` | `research-ops/SKILL.md` | 0.9 |
| `compare.*options` | `research-ops/SKILL.md` | 0.9 |
| `market.*research` | `research-ops/SKILL.md` | 0.9 |
| `(rfc\|decompose\|unit).*(feature\|dag\|work unit\|merge queue)` | `rfc-and-unit-orchestration/SKILL.md` | 0.9 |
| `ralphinho rfc pipeline` | `rfc-and-unit-orchestration/SKILL.md` | 0.9 |
| `(distill\|extract\|update).*(rules\|principles)` | `rules-distill/SKILL.md` | 0.9 |
| `turn (skills\|patterns) into rules` | `rules-distill/SKILL.md` | 0.9 |
| `(design\|build\|review).*(rust\|cargo\|crate\|borrow checker)` | `rust-development/SKILL.md` | 0.9 |
| `rust patterns` | `rust-development/SKILL.md` | 0.9 |
| `(run\|execute).*(danger\|production\|autonomous)` | `safety-guard/SKILL.md` | 0.9 |
| `prevent (destructive\|dangerous) (operations\|commands)` | `safety-guard/SKILL.md` | 0.9 |
| `(freeze\|lock).*(directory\|folder\|writes)` | `safety-guard/SKILL.md` | 0.9 |
| `(scroll\|parallax\|pin).*(animation\|trigger\|gsap)` | `scroll-driven-animation/SKILL.md` | 0.9 |
| `scrolltrigger` | `scroll-driven-animation/SKILL.md` | 0.9 |
| `(add\|build\|implement\|create).*(feature\|functionality\|integration\|utility)` | `search-first/SKILL.md` | 0.9 |
| `how should (we\|I) solve` | `search-first/SKILL.md` | 0.9 |
| `write a (script\|wrapper\|client) for` | `search-first/SKILL.md` | 0.9 |
| `(hunt\|discover).*(vulnerability\|exploit\|bounty)` | `security-bounty-hunter/SKILL.md` | 0.9 |
| `find exploitable bugs` | `security-bounty-hunter/SKILL.md` | 0.9 |
| `(check\|audit\|review).*(security\|auth\|secrets\|vulnerability)` | `security-review/SKILL.md` | 0.9 |
| `is this (safe\|secure)` | `security-review/SKILL.md` | 0.9 |
| `(scan\|audit).*(config\|settings\|shield)` | `security-scan/SKILL.md` | 0.9 |
| `npx ecc-agentshield` | `security-scan/SKILL.md` | 0.9 |
| `(seo\|keyword\|sitemap\|metadata).*(audit\|plan\|implement)` | `seo-optimization/SKILL.md` | 0.9 |
| `improve search visibility` | `seo-optimization/SKILL.md` | 0.9 |
| `(check\|verify\|audit).*(compliance\|rule following\|skill usage)` | `skill-comply/SKILL.md` | 0.9 |
| `is the agent (following\|obeying) (the rule\|this skill)` | `skill-comply/SKILL.md` | 0.9 |
| `(find\|score\|rank).*(lead\|prospect\|social graph\|outreach)` | `social-and-lead-intelligence/SKILL.md` | 0.9 |
| `warm path discovery` | `social-and-lead-intelligence/SKILL.md` | 0.9 |
| `(when\|should I) (compact\|clear context)` | `strategic-compact/SKILL.md` | 0.9 |
| `/compact` | `strategic-compact/SKILL.md` | 0.9 |
| `execute (plan\|tasks\|implementation)` | `subagent-driven-development/SKILL.md` | 0.9 |
| `(delegate\|dispatch).*(task\|subagent)` | `subagent-driven-development/SKILL.md` | 0.9 |
| `(build\|design).*(swiftui\|ios\|macos).*(view\|interface)` | `swiftui-patterns/SKILL.md` | 0.9 |
| `swiftui patterns` | `swiftui-patterns/SKILL.md` | 0.9 |
| `(bug\|error\|failure\|crash\|unexpected\|broken).*` | `systematic-debugging/SKILL.md` | 0.9 |
| `debug.*` | `systematic-debugging/SKILL.md` | 0.9 |
| `why does.*fail` | `systematic-debugging/SKILL.md` | 0.9 |
| `fix.*(again\|it)` | `systematic-debugging/SKILL.md` | 0.9 |
| `(implement\|add\|fix).* (feature\|bug\|capability)` | `test-driven-development/SKILL.md` | 0.9 |
| `tdd\|red-green-refactor` | `test-driven-development/SKILL.md` | 0.9 |
| `write.*test.*first` | `test-driven-development/SKILL.md` | 0.9 |
| `(token budget\|depth\|length\|detailed\|brief\|tldr)` | `token-budget-advisor/SKILL.md` | 0.9 |
| `respond at (25\|50\|75\|100)%` | `token-budget-advisor/SKILL.md` | 0.9 |
| `(design\|build\|refactor).*(ui\|ux\|interface\|page\|component)` | `ui-ux-design/SKILL.md` | 0.9 |
| `recommend (style\|color\|font)` | `ui-ux-design/SKILL.md` | 0.9 |
| `verify.*(changes\|implementation\|all)` | `verification-loop/SKILL.md` | 0.9 |
| `run.*(quality-gate\|validation-loop)` | `verification-loop/SKILL.md` | 0.9 |
| `is.*(ready\|complete).*(for\|to\|PR)` | `verification-loop/SKILL.md` | 0.9 |
| `(manage\|branch\|release\|triage).*(git\|github\|ios\|foundationmodel)` | `workflow-and-platform-ops/SKILL.md` | 0.9 |
| `git flow` | `workflow-and-platform-ops/SKILL.md` | 0.9 |
| `(3d\|model\|mesh\|sculpt\|retopo\|retopology\|uv\|bake\|baking\|pbr\|materialx\|openpbr\|blender\|bpy\|maya\|houdini\|unreal\|nanite\|gltf\|fbx\|usd\|obj\|substance)` | `autonomous-3d-studio/SKILL.md` | 0.9 |
| `(hard[- ]surface\|organic\|character\|anatomy\|facs\|rigging\|skinning\|geometry[- ]nodes\|vex\|texel[- ]density\|subd\|trellis\|hunyuan3d\|rodin)` | `autonomous-3d-studio/SKILL.md` | 0.9 |
| `(post\|tweet\|thread\|search).*(x\|twitter\|social media)` | `social-media-automation/SKILL.md` | 0.9 |
| `x api` | `social-media-automation/SKILL.md` | 0.9 |
| `(integrate\|build\|add).*(connector\|provider\|integration\|adapter)` | `system-integration/SKILL.md` | 0.9 |
| `hexagonal architecture` | `system-integration/SKILL.md` | 0.9 |
| `(generate\|write).*(timeline\|journey\|history).*(report)` | `timeline-report/SKILL.md` | 0.9 |
| `what is the story of this project` | `timeline-report/SKILL.md` | 0.9 |
| `(record\|create).*(demo\|walkthrough\|tutorial).*(video)` | `ui-demo-recording/SKILL.md` | 0.9 |
| `show me how it works visually` | `ui-demo-recording/SKILL.md` | 0.9 |
| `(unify\|consolidate\|route).*(notification\|alert\|ping)` | `unified-notifications-ops/SKILL.md` | 0.9 |
| `what (is happening\|happened) while I was (away\|gone)` | `unified-notifications-ops/SKILL.md` | 0.9 |
| `(edit\|cut\|transcode\|index\|stream).*(video\|footage)` | `video-operations/SKILL.md` | 0.9 |
| `videodb` | `video-operations/SKILL.md` | 0.9 |
| `(check\|verify\|audit).*(phi\|safety\|decimal\|erc20\|patient)` | `vulnerability-and-safety-gates/SKILL.md` | 0.9 |
| `healthcare eval harness` | `vulnerability-and-safety-gates/SKILL.md` | 0.9 |
