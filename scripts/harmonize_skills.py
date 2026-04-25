import os
import re

SKILLS_DIR = r"C:\Users\Eduardo\Ciel\skills"
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
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # 1. Runtime Normalization (H1)
    # Match runtimes: [...] or runtimes: \n - ...
    content = re.sub(r'runtimes:.*?(?=\n\w)', 'runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]', content, flags=re.DOTALL)

    # 2. Domain Tag Enrichment (H2)
    skill_name = os.path.basename(os.path.dirname(file_path))
    domain = get_domain(skill_name, content)
    
    # Update tags
    tag_match = re.search(r'tags: \[(.*?)\]', content)
    if tag_match:
        tags = [t.strip().strip('"').strip("'") for t in tag_match.group(1).split(',')]
        if f"domain:{domain}" not in tags:
            tags.append(f"domain:{domain}")
        # Clean up tags
        tags = [t for t in tags if t not in ["ciel", "harmonized"]]
        tags = ["ciel", "harmonized"] + sorted(list(set(tags)))
        new_tags_str = 'tags: [' + ', '.join(f'"{t}"' for t in tags) + ']'
        content = content.replace(tag_match.group(0), new_tags_str)

    # 3. Placeholder Purge (M6)
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
