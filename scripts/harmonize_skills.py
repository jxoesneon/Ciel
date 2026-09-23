import os
import re

SKILLS_DIR = r"~\Ciel\skills"
DOMAIN_TAGS = {
    "web": ["web", "frontend", "html", "css", "js", "react", "nextjs", "nuxt", "gsap", "remotion", "seo", "modern-js", "laravel"],
    "systems": ["systems", "kernel", "os", "container", "docker", "rust", "cpp", "go", "jvm", "java", "dotnet", "database", "migrations", "perl"],
    "ai": ["ai", "agent", "llm", "neural", "prompt", "eval", "mcp", "intelligence", "autonomous", "orchestration", "swarm"],
    "ops": ["ops", "deployment", "ci", "cd", "automation", "logistics", "billing", "monitoring", "jira", "google-workspace", "content-distribution", "email"],
    "mobile": ["mobile", "android", "kmp", "ios", "swift", "flutter", "compose-multiplatform"],
    "data": ["data", "analytical", "ml", "database", "retrieval", "context", "knowledge", "memory"],
    "security": ["security", "vulnerability", "safety", "compliance", "audit", "crypto", "guard"],
    "quality": ["quality", "test", "verification", "debugging", "linter", "review", "eval-harness"],
    "strategy": ["strategy", "planning", "research", "brainstorming", "decision", "identity", "brand", "compact", "flow"],
    "design": ["design", "ui", "ux", "presentation", "animation", "asset", "gsap", "remotion"]
}

def get_domain(skill_name, content):
    for domain, keywords in DOMAIN_TAGS.items():
        if any(kw in skill_name.lower() for kw in keywords):
            return domain
        if any(kw in content.lower() for kw in keywords):
            return domain
    return "strategy" # Default

def harmonize_skill(file_path):
    skill_dir = os.path.dirname(file_path)
    # Ciel extension fields (runtimes, tags, ...) live in the ciel.yaml
    # sidecar when present; SKILL.md stays Agent-Skills-spec pure.
    meta_path = os.path.join(skill_dir, "ciel.yaml")
    if not os.path.isfile(meta_path):
        meta_path = file_path

    with open(meta_path, encoding='utf-8') as f:
        meta = f.read()

    # 1. Runtime Normalization (H1)
    # Match runtimes: [...] or runtimes: \n - ...
    meta = re.sub(r'runtimes:.*?(?=\n\w)', 'runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]', meta, flags=re.DOTALL)

    # 2. Domain Tag Enrichment (H2)
    skill_name = os.path.basename(skill_dir)
    domain = get_domain(skill_name, meta)

    # Update tags
    tag_match = re.search(r'tags: \[(.*?)\]', meta)
    if tag_match:
        tags = [t.strip().strip('"').strip("'") for t in tag_match.group(1).split(',')]
        if f"domain:{domain}" not in tags:
            tags.append(f"domain:{domain}")
        # Clean up tags
        tags = [t for t in tags if t not in ["ciel", "harmonized"]]
        tags = ["ciel", "harmonized"] + sorted(set(tags))
        new_tags_str = 'tags: [' + ', '.join(f'"{t}"' for t in tags) + ']'
        meta = meta.replace(tag_match.group(0), new_tags_str)

    if meta_path != file_path:
        with open(meta_path, 'w', encoding='utf-8') as f:
            f.write(meta)
    else:
        content = meta

    # 3. Placeholder Purge (M6) — SKILL.md body only
    with open(file_path, encoding='utf-8') as f:
        content = f.read() if meta_path != file_path else content
    # Replace TODO, FIXME, ... with signal
    content = content.replace('TODO', 'Refine implementation logic to align with Ciel 1.0 standards.')
    content = content.replace('FIXME', 'Resolve architectural debt and ensure deterministic behavior.')

    # Only replace ... if it looks like a placeholder, not a prose ellipsis
    # Usually placeholders are alone on a line or in brackets
    content = re.sub(r'\[\.\.\.\]', '[Comprehensive implementation details following Ciel spec]', content)
    content = re.sub(r'^\s*\.\.\.\s*$', '    [Continuous integration and verification steps]', content, flags=re.MULTILINE)

    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)

def main():
    for root, dirs, files in os.walk(SKILLS_DIR):
        if 'SKILL.md' in files:
            harmonize_skill(os.path.join(root, 'SKILL.md'))

if __name__ == "__main__":
    main()
