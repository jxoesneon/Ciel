//! Port of `skills/ui-ux-pro-max/scripts/validate_data.py` — the stdlib-only
//! data-integrity guardrail. `validate()` collects every problem; `run()`
//! prints the FAILED list (exit 1) or the OK summary (exit 0).

use crate::common::csvio::Row;
use crate::common::py;
use crate::uiux::core::{
    stack_applicability, CSV_CONFIG, STACK_CONFIG, STACK_OUTPUT_COLS, STACK_SEARCH_COLS,
};
use crate::uiux::data::Catalog;
use crate::uiux::reasoning::parse_decision_rules;
use regex::Regex;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::OnceLock;

const REASONING_FILE: &str = "ui-reasoning.csv";

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap()
}

fn re_i(pattern: &str) -> Regex {
    Regex::new(&format!("(?i){}", pattern)).unwrap()
}

fn fullmatch(pattern: &str, text: &str) -> bool {
    re(&format!("\\A(?:{})\\z", pattern)).is_match(text)
}

const HEX_COLOR: &str = r"#[0-9A-Fa-f]{6}";
const STYLE_ID: &str = r"[a-z0-9]+(?:-[a-z0-9]+)*";

fn wcag_conformance() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| re_i(r"\bWCAG\s+A{2,3}\+?\b|\bWCAG\b.{0,40}\b(?:compliant|compliance)\b"))
}
fn wcag_grade() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| re_i(r"\bWCAG\s+A{1,3}\b"))
}
fn chart_text_fallback() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| re(r"table|list|summary|text"))
}
fn chart_non_color_guidance() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| re(
        r"color alone|hue alone|alone is insufficient|only distinction|only carriers?|supplementary|pattern|symbol|outline|direct (?:series |group )?label|marker shape|label every",
    ))
}
fn css_import_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Python uses a backreference (\1) for the quote char; Rust regex has no
    // backrefs, so use a quote-char alternation instead.
    RE.get_or_init(|| re_i(r"\A\s*@import\s+url\((?:'([^']+)'|\x22([^\x22]+)\x22)\);\s*\z"))
}
fn icon_import_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| re(
        r"import\s*\{\s*[A-Z][A-Za-z0-9]*(?:\s*,\s*[A-Z][A-Za-z0-9]*)*\s*\}\s*from\s*['\x22](?:@phosphor-icons/react|phosphor-react-native|@heroicons/react/24/(?:outline|solid))['\x22]",
    ))
}
fn landing_quantified_claim() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        re_i(r"\b\d+(?:\.\d+)?x\b|\b(?:increases?|reduces?)\s+(?:engagement|conversion|returns?)\b")
    })
}

const ICON_USAGE_REQUIREMENTS: &[&str] = &[
    r"aria-hidden",
    r"text alternative",
    r"accessible name",
    r"aria-(?:pressed|expanded)",
];

const CHART_RISKS: &[&str] = &["risk:low", "risk:conditional", "risk:high"];
const ICON_ROLES: &[&str] = &["decorative", "meaningful", "interactive", "guideline"];
const ICON_CONTEXTS: &[&str] = &["decorative", "meaningful", "interactive"];
const STYLE_STATUSES: &[&str] = &["active", "supplemental", "deprecated"];
const STACK_STATUSES: &[&str] = &["active", "supplemental", "deprecated", "unverified"];
const FONT_LICENSES: &[&str] = &["OFL", "APACHE2", "UFL"];
const PHOSPHOR_WEIGHTS: &[&str] = &["thin", "light", "regular", "bold", "fill", "duotone"];
const PROVENANCE_KINDS: &[&str] = &["reasoning", "style", "dataset-contract", "catalog-snapshot"];
const PROVENANCE_STATUSES: &[&str] = &["active", "supplemental", "deprecated"];
const PROVENANCE_SLAS: &[&str] = &["manual-verified", "needs-review"];
const PROVENANCE_SOURCE_TYPES: &[&str] = &["official", "derived"];
const PROVENANCE_APPLIES_TO: &[&str] = &[
    "search",
    "design-guidance",
    "design-system",
    "gallery",
    "style-search",
];
const CORE_PROVENANCE_FILES: &[&str] = &[
    "colors.csv",
    "charts.csv",
    "ux-guidelines.csv",
    "landing.csv",
    "typography.csv",
    "icons.csv",
    "motion.csv",
    "app-interface.csv",
    "react-performance.csv",
    "stacks/html-tailwind.csv",
];
const CATALOG_PROVENANCE_FILES: &[&str] = &["google-fonts.csv", "phosphor-icons-upstream.json"];
const CATALOG_PROVENANCE_IDS: &[&str] = &[
    "google-fonts-catalog-2026-08-13",
    "phosphor-icons-catalog-2.1.1",
];

const OFFICIAL_SOURCE_HOSTS: &[&str] = &[
    "carbondesignsystem.com",
    "developer.android.com",
    "developer.apple.com",
    "developers.google.com",
    "fluent2.microsoft.design",
    "github.com",
    "greensock.com",
    "gsap.com",
    "m3.material.io",
    "opensource.adobe.com",
    "react.dev",
    "s2.spectrum.adobe.com",
    "shopify.dev",
    "design-system.service.gov.uk",
    "tailwindcss.com",
    "www.w3.org",
];

fn stack_official_hosts(stack: &str) -> &'static [&'static str] {
    match stack {
        "react" => &["react.dev"],
        "nextjs" => &["nextjs.org"],
        "vue" => &["vuejs.org", "pinia.vuejs.org"],
        "svelte" => &["svelte.dev", "kit.svelte.dev"],
        "astro" => &["docs.astro.build"],
        "angular" => &["angular.dev"],
        "html-tailwind" => &["tailwindcss.com"],
        "shadcn" => &["ui.shadcn.com"],
        "nuxtjs" => &["nuxt.com"],
        "nuxt-ui" => &["ui.nuxt.com"],
        "react-native" => &["reactnative.dev", "react.dev"],
        "flutter" => &["api.flutter.dev", "docs.flutter.dev"],
        "swiftui" => &["developer.apple.com"],
        "jetpack-compose" => &["developer.android.com"],
        "avalonia" => &["docs.avaloniaui.net"],
        "uwp" | "winui" | "wpf" => &["learn.microsoft.com"],
        "uno" => &["platform.uno"],
        "javafx" => &["openjfx.io", "mkpaz.github.io", "www.w3.org"],
        "threejs" => &["threejs.org", "github.com", "www.npmjs.com", "www.w3.org"],
        "laravel" => &["laravel.com"],
        _ => &[],
    }
}

const REQUIRED_UX_GUIDANCE: &[(&str, &str)] = &[
    ("Focus Not Obscured (Minimum)", "Web"),
    ("Focus Not Obscured (Enhanced)", "Web"),
    ("Focus Appearance", "Web"),
    ("Dragging Movements", "All"),
    ("Target Size (Minimum)", "Web"),
    ("Consistent Help", "All"),
    ("Redundant Entry", "All"),
    ("Accessible Authentication (Minimum)", "All"),
    ("Auto-Rotating Content Controls", "All"),
];

/// (foreground, background, role, minimum) in Python dict order.
const COLOR_CONTRAST_PAIRS: &[(&str, &str, &str, f64)] = &[
    ("On Primary", "Primary", "normal-text", 4.5),
    ("On Secondary", "Secondary", "normal-text", 4.5),
    ("On Accent", "Accent", "normal-text", 4.5),
    ("Foreground", "Background", "normal-text", 4.5),
    ("Card Foreground", "Card", "normal-text", 4.5),
    ("Muted Foreground", "Muted", "normal-text", 4.5),
    ("On Destructive", "Destructive", "normal-text", 4.5),
    ("Ring", "Background", "non-text-focus-indicator", 3.0),
];

// ============ MINIMAL urlsplit / parse_qs / quote_plus ============

#[derive(Default)]
struct UrlParts {
    scheme: String,
    username: Option<String>,
    password: Option<String>,
    hostname: String,
    port_present: bool,
    port_valid: bool,
    path: String,
    query: String,
}

/// `urllib.parse.urlsplit` for the http(s)-oriented URLs in this dataset.
fn urlsplit(url: &str) -> UrlParts {
    let mut out = UrlParts::default();
    let mut rest = url;
    if let Some(idx) = rest.find("://") {
        out.scheme = rest[..idx].to_lowercase();
        rest = &rest[idx + 3..];
    }
    // split authority / path+query
    let (authority, tail) = match rest.find(['/', '?', '#']) {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    if out.scheme.is_empty() {
        // no scheme: the whole string is the path
        out.path = rest.to_string();
        return out;
    }
    let (userinfo, hostport) = match authority.rfind('@') {
        Some(i) => (Some(&authority[..i]), &authority[i + 1..]),
        None => (None, authority),
    };
    if let Some(ui) = userinfo {
        match ui.find(':') {
            Some(i) => {
                out.username = Some(ui[..i].to_string());
                out.password = Some(ui[i + 1..].to_string());
            }
            None => out.username = Some(ui.to_string()),
        }
    }
    let (host, port_str) = match hostport.find(':') {
        Some(i) => (&hostport[..i], Some(&hostport[i + 1..])),
        None => (hostport, None),
    };
    out.hostname = host.to_lowercase();
    if let Some(p) = port_str {
        out.port_present = true;
        out.port_valid = p.parse::<u16>().is_ok();
    }
    // tail: path?query#fragment
    let tail = tail.split('#').next().unwrap_or("");
    match tail.find('?') {
        Some(i) => {
            out.path = tail[..i].to_string();
            out.query = tail[i + 1..].to_string();
        }
        None => out.path = tail.to_string(),
    }
    out
}

fn pct_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = &s[i + 1..i + 3];
                match u8::from_str_radix(hex, 16) {
                    Ok(v) => out.push(v),
                    Err(_) => out.push(bytes[i]),
                }
                i += 3;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `urllib.parse.parse_qs(qs)` — '&' separator, percent-decoding, and the
/// default keep_blank_values=False behavior.
fn parse_qs(qs: &str) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for pair in qs.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = match pair.find('=') {
            Some(i) => (&pair[..i], &pair[i + 1..]),
            None => continue,
        };
        if v.is_empty() {
            continue;
        }
        out.entry(pct_decode(k)).or_default().push(pct_decode(v));
    }
    out
}

/// `urllib.parse.quote_plus` — space -> '+', everything outside
/// [A-Za-z0-9_.~-] percent-encoded uppercase.
fn quote_plus(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b' ' => out.push('+'),
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

// ============ HELPERS ============

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = values.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 1 {
        values[n / 2]
    } else {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    }
}

fn split_field(value: &str, delimiter: char) -> Vec<String> {
    value
        .split(delimiter)
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

fn valid_date(value: &str) -> bool {
    py::valid_date_not_future(value)
}

fn catalog_date(value: &Value) -> bool {
    match value.as_str() {
        Some(s) => match py::parse_iso_date(s) {
            Some((y, _, _)) => y > 1970 && py::valid_date_not_future(s),
            None => false,
        },
        None => false,
    }
}

fn valid_confidence(value: &Value) -> bool {
    if value.is_null() {
        return true;
    }
    if value.is_boolean() {
        return false;
    }
    match value.as_f64() {
        Some(v) => v.is_finite() && (0.0..=1.0).contains(&v),
        None => false,
    }
}

/// Strict `_relative_luminance` — raises an error message for invalid hex.
fn strict_luminance(value: &str) -> Result<f64, String> {
    if !fullmatch(HEX_COLOR, value) {
        return Err(format!("invalid hex color '{}'", value));
    }
    let mut channels = [0f64; 3];
    for (i, ch) in channels.iter_mut().enumerate() {
        *ch = f64::from(
            u8::from_str_radix(&value[i * 2 + 1..i * 2 + 3], 16)
                .map_err(|_| format!("invalid hex color '{}'", value))?,
        ) / 255.0;
    }
    let linear: Vec<f64> = channels
        .iter()
        .map(|c| {
            if *c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        })
        .collect();
    Ok(0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2])
}

fn contrast_ratio(foreground: &str, background: &str) -> Result<f64, String> {
    let first = strict_luminance(foreground)?;
    let second = strict_luminance(background)?;
    Ok((first.max(second) + 0.05) / (first.min(second) + 0.05))
}

/// `(?<!\d)[1-9]00(?!\d)` — a standalone 3-digit run matching [1-9]00.
fn font_weights(text: &str) -> BTreeSet<String> {
    let bytes = text.as_bytes();
    let mut out = BTreeSet::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let run = &text[start..i];
            if run.len() == 3
                && run.as_bytes()[0] != b'0'
                && run.as_bytes()[1] == b'0'
                && run.as_bytes()[2] == b'0'
            {
                out.insert(run.to_string());
            }
        } else {
            i += 1;
        }
    }
    out
}

// ============ CHECKS ============

fn check_file(
    catalog: &Catalog,
    label: &str,
    rel: &str,
    search_cols: &[&str],
    output_cols: &[&str],
    problems: &mut Vec<String>,
) -> (Vec<String>, Vec<Row>) {
    let filepath = catalog.path(rel);
    if !filepath.exists() {
        problems.push(format!("[{}] missing file: {}", label, filepath.display()));
        return (vec![], vec![]);
    }
    let table = match catalog.table(rel) {
        Ok(t) => t,
        Err(e) => {
            problems.push(format!("[{}] failed to parse {}: {}", label, rel, e));
            return (vec![], vec![]);
        }
    };
    let header_set: HashSet<&str> = table.headers.iter().map(|s| s.as_str()).collect();
    let mut referenced: BTreeSet<&str> = BTreeSet::new();
    for c in search_cols.iter().chain(output_cols.iter()) {
        referenced.insert(c);
    }
    for col in referenced {
        if !header_set.contains(col) {
            problems.push(format!(
                "[{}] {}: expected column '{}' not found in header",
                label, rel, col
            ));
        }
    }
    if header_set.contains("No") {
        let mut seen: HashMap<String, usize> = HashMap::new();
        for (i, row) in table.rows.iter().enumerate() {
            let key = row.get("No").cloned().unwrap_or_default();
            let line = i + 2;
            if let Some(prev) = seen.get(&key) {
                problems.push(format!(
                    "[{}] {}: duplicate 'No' value '{}' on rows {} and {}",
                    label, rel, key, prev, line
                ));
            } else {
                seen.insert(key, line);
            }
        }
    } else if label.starts_with("stack:") {
        problems.push(format!(
            "[{}] {}: missing 'No' index column present in other stack files (schema drift -- harmless for search, but inconsistent with the rest of data/stacks/)",
            label, rel
        ));
    }
    (table.headers, table.rows)
}

fn check_style_contract(
    styles: &[Row],
    products: &[Row],
    reasoning: &[Row],
    problems: &mut Vec<String>,
    replacement_targets: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    let mut ids: HashSet<String> = HashSet::new();
    let mut lookup: HashMap<String, String> = HashMap::new();
    let mut parents: Vec<(String, String)> = Vec::new();
    let mut by_id: HashMap<String, Row> = HashMap::new();
    for row in styles {
        let style_id = row.get("Style ID").cloned().unwrap_or_default();
        if !fullmatch(STYLE_ID, &style_id) {
            problems.push(format!("[style] invalid Style ID '{}'", style_id));
        }
        if ids.contains(&style_id) {
            problems.push(format!("[style] duplicate Style ID '{}'", style_id));
        }
        ids.insert(style_id.clone());
        by_id.insert(style_id.clone(), row.clone());
        let status = row.get("Status").cloned().unwrap_or_default();
        let parent = row.get("Parent Style ID").cloned().unwrap_or_default();
        if !STYLE_STATUSES.contains(&status.as_str()) {
            problems.push(format!("[style:{}] invalid Status '{}'", style_id, status));
        }
        let replacement_domain = row.get("Replacement Domain").cloned().unwrap_or_default();
        let replacement_id = row.get("Replacement ID").cloned().unwrap_or_default();
        if status == "supplemental" && parent.is_empty() {
            problems.push(format!(
                "[style:{}] supplemental rows require Parent Style ID",
                style_id
            ));
        }
        if status == "deprecated" {
            let has_parent = !parent.is_empty();
            let has_redirect = !replacement_domain.is_empty() && !replacement_id.is_empty();
            if has_parent == has_redirect {
                problems.push(format!(
                    "[style:{}] deprecated rows require exactly one parent or redirect",
                    style_id
                ));
            }
            if has_redirect
                && !replacement_targets
                    .get(&replacement_domain)
                    .map(|s| s.contains(&replacement_id))
                    .unwrap_or(false)
            {
                problems.push(format!(
                    "[style:{}] invalid {} redirect '{}'",
                    style_id, replacement_domain, replacement_id
                ));
            }
        } else if !replacement_domain.is_empty() || !replacement_id.is_empty() {
            problems.push(format!(
                "[style:{}] only deprecated rows may redirect",
                style_id
            ));
        }
        parents.push((style_id.clone(), parent));
        let mut keys = vec![
            style_id.clone(),
            row.get("Style Category").cloned().unwrap_or_default(),
        ];
        keys.extend(split_field(
            row.get("Aliases").map(|s| s.as_str()).unwrap_or(""),
            '|',
        ));
        for key in keys {
            let folded = key.to_lowercase();
            if let Some(prev) = lookup.get(&folded) {
                if *prev != style_id {
                    problems.push(format!(
                        "[style] ambiguous identity '{}' -> {}, {}",
                        key, prev, style_id
                    ));
                }
            }
            lookup.insert(folded, style_id.clone());
        }
    }
    let parent_map: HashMap<String, String> = parents.iter().cloned().collect();
    for (style_id, parent) in &parents {
        if !parent.is_empty() && (!ids.contains(parent) || parent == style_id) {
            problems.push(format!("[style:{}] invalid parent '{}'", style_id, parent));
        }
        let mut seen: HashSet<String> = [style_id.clone()].into_iter().collect();
        let mut current = parent.clone();
        while !current.is_empty() {
            if seen.contains(&current) {
                problems.push(format!(
                    "[style:{}] parent cycle through '{}'",
                    style_id, current
                ));
                break;
            }
            seen.insert(current.clone());
            current = parent_map.get(&current).cloned().unwrap_or_default();
        }
        if !parent.is_empty()
            && by_id
                .get(parent)
                .and_then(|r| r.get("Status"))
                .map(|s| s.as_str())
                != Some("active")
        {
            problems.push(format!(
                "[style:{}] parent must target an active style",
                style_id
            ));
        }
    }
    let mut references: Vec<String> = Vec::new();
    for row in products {
        references.extend(split_field(
            row.get("Primary Style Recommendation")
                .map(|s| s.as_str())
                .unwrap_or(""),
            '+',
        ));
        references.extend(split_field(
            row.get("Secondary Styles")
                .map(|s| s.as_str())
                .unwrap_or(""),
            ',',
        ));
    }
    for row in reasoning {
        references.extend(split_field(
            row.get("Style_Priority").map(|s| s.as_str()).unwrap_or(""),
            '+',
        ));
    }
    let mut sorted_refs: Vec<String> = references
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    sorted_refs.sort();
    for reference in sorted_refs {
        match lookup.get(&reference.to_lowercase()) {
            None => problems.push(format!("[style] unresolved reference '{}'", reference)),
            Some(resolved_id) => {
                if let Some(row) = styles
                    .iter()
                    .find(|r| r.get("Style ID").map(|s| s.as_str()) == Some(resolved_id.as_str()))
                {
                    if row.get("Status").map(|s| s.as_str()) == Some("deprecated") {
                        problems.push(format!(
                            "[style] reference targets deprecated style '{}'",
                            reference
                        ));
                    }
                }
            }
        }
    }

    let performance_levels = ["cost:low", "cost:moderate", "cost:high"];
    let accessibility_levels = ["risk:low", "risk:conditional", "risk:high"];
    let mode_levels = ["supported", "conditional", "not-recommended"];
    let mut prompt_lengths: HashMap<String, Vec<f64>> = HashMap::new();
    for row in styles {
        let style_id = row.get("Style ID").cloned().unwrap_or_default();
        let perf = row
            .get("Performance")
            .map(|s| s.split('|').next().unwrap_or("").to_string())
            .unwrap_or_default();
        if !performance_levels.contains(&perf.as_str()) {
            problems.push(format!(
                "[style:{}] invalid Performance vocabulary",
                style_id
            ));
        }
        let acc = row
            .get("Accessibility")
            .map(|s| s.split('|').next().unwrap_or("").to_string())
            .unwrap_or_default();
        if !accessibility_levels.contains(&acc.as_str()) {
            problems.push(format!(
                "[style:{}] invalid Accessibility vocabulary",
                style_id
            ));
        }
        let claim_text = row.values().cloned().collect::<Vec<_>>().join(" ");
        if wcag_conformance().is_match(&claim_text) {
            problems.push(format!(
                "[style:{}] accessibility conformance guarantee",
                style_id
            ));
        }
        let fw = row
            .get("Framework Compatibility")
            .cloned()
            .unwrap_or_default();
        if re(r"\d+/10").is_match(&fw) {
            problems.push(format!(
                "[style:{}] framework score is unsupported",
                style_id
            ));
        }
        let fw_lower = fw.to_lowercase();
        if ["all frameworks", "excellent", "performant", "lightweight"]
            .iter()
            .any(|t| fw_lower.contains(t))
        {
            problems.push(format!(
                "[style:{}] unsupported framework guarantee",
                style_id
            ));
        }
        for field in ["Light Mode \u{2713}", "Dark Mode \u{2713}"] {
            let v = row.get(field).cloned().unwrap_or_default();
            if !mode_levels.contains(&v.as_str()) {
                problems.push(format!("[style:{}] invalid {} vocabulary", style_id, field));
            }
        }
        let pm = row.get("Preferred Mode").cloned().unwrap_or_default();
        if !["auto", "light", "dark"].contains(&pm.as_str()) {
            problems.push(format!("[style:{}] invalid Preferred Mode", style_id));
        }
        let length = row
            .get("AI Prompt Keywords")
            .map(|s| s.split_whitespace().count())
            .unwrap_or(0) as f64;
        prompt_lengths
            .entry(row.get("Type").cloned().unwrap_or_default())
            .or_default()
            .push(length);
        if length > 40.0 {
            problems.push(format!("[style:{}] AI prompt exceeds 40 words", style_id));
        }
    }
    if let (Some(general), Some(mobile)) = (
        prompt_lengths.get_mut("General").map(|v| median(v)),
        prompt_lengths.get_mut("Mobile").map(|v| median(v)),
    ) {
        if mobile > general * 1.25 {
            problems.push("[style] mobile prompt median exceeds 1.25x general median".to_string());
        }
    }
    ids
}

fn check_reasoning_contract(
    products: &[Row],
    colors: &[Row],
    reasoning: &[Row],
    style_ids: &HashSet<String>,
    patterns: &HashSet<String>,
    problems: &mut Vec<String>,
) {
    let semantic_sets: &[(&str, &[Row], &str)] = &[
        ("products", products, "Product Type"),
        ("colors", colors, "Product Type"),
        ("reasoning", reasoning, "UI_Category"),
    ];
    for (label, rows, key) in semantic_sets {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for row in *rows {
            *counts
                .entry(row.get(*key).cloned().unwrap_or_default())
                .or_insert(0) += 1;
        }
        let mut duplicates: Vec<String> = counts
            .iter()
            .filter(|(_, c)| **c > 1)
            .map(|(v, _)| v.clone())
            .collect();
        duplicates.sort();
        if rows.len() != 192 {
            problems.push(format!(
                "[reasoning] {} must contain exactly 192 rows; got {}",
                label,
                rows.len()
            ));
        }
        if !duplicates.is_empty() {
            problems.push(format!(
                "[reasoning] duplicate {} labels: {}",
                label,
                duplicates.join(", ")
            ));
        }
    }
    let names = |rows: &[Row], key: &str| -> HashSet<String> {
        rows.iter()
            .map(|r| r.get(key).cloned().unwrap_or_default())
            .collect()
    };
    let product_names = names(products, "Product Type");
    let color_names = names(colors, "Product Type");
    let reasoning_names = names(reasoning, "UI_Category");
    if product_names != color_names {
        problems.push("[reasoning] products/colors product labels differ".to_string());
    }
    let mut missing: Vec<String> = product_names
        .difference(&reasoning_names)
        .cloned()
        .collect();
    let mut extra: Vec<String> = reasoning_names
        .difference(&product_names)
        .cloned()
        .collect();
    missing.sort();
    extra.sort();
    if !missing.is_empty() {
        problems.push(format!(
            "[reasoning] missing exact product rows: {}",
            missing.join(", ")
        ));
    }
    if !extra.is_empty() {
        problems.push(format!(
            "[reasoning] unknown product rows: {}",
            extra.join(", ")
        ));
    }
    for row in reasoning {
        let category = row.get("UI_Category").cloned().unwrap_or_default();
        let rules =
            match parse_decision_rules(row.get("Decision_Rules").map(|s| s.as_str()).unwrap_or(""))
            {
                Ok(r) => r,
                Err(e) => {
                    problems.push(format!("[reasoning:{}] {}", category, e));
                    continue;
                }
            };
        for actions in rules.values() {
            for action in actions.as_array().cloned().unwrap_or_default() {
                let text = action.as_str().unwrap_or("");
                let (prefix, value) = text.split_once(':').unwrap_or(("", ""));
                if prefix == "style" && !style_ids.contains(value) {
                    problems.push(format!(
                        "[reasoning:{}] unknown style action '{}'",
                        category, value
                    ));
                }
                if prefix == "pattern" && !patterns.contains(value) {
                    problems.push(format!(
                        "[reasoning:{}] unknown pattern action '{}'",
                        category, value
                    ));
                }
            }
        }
        let pattern = row.get("Recommended_Pattern").cloned().unwrap_or_default();
        if !patterns.contains(&pattern) {
            problems.push(format!(
                "[reasoning:{}] unknown Recommended_Pattern '{}'",
                category, pattern
            ));
        }
        let confidence = row.get("Confidence").cloned().unwrap_or_default();
        if !confidence.is_empty() {
            let ok = confidence
                .parse::<f64>()
                .map(|v| (0.0..=1.0).contains(&v))
                .unwrap_or(false);
            if !ok {
                problems.push(format!(
                    "[reasoning:{}] invalid Confidence '{}'",
                    category, confidence
                ));
            }
        }
    }
}

fn check_color_contract(rows: &[Row], problems: &mut Vec<String>) {
    for row in rows {
        let product = row.get("Product Type").cloned().unwrap_or_default();
        let mut ratios: HashMap<String, f64> = HashMap::new();
        for &(foreground, background, role, minimum) in COLOR_CONTRAST_PAIRS {
            let fg = row.get(foreground).cloned().unwrap_or_default();
            let bg = row.get(background).cloned().unwrap_or_default();
            match contrast_ratio(&fg, &bg) {
                Err(e) => {
                    problems.push(format!("[color:{}] {}", product, e));
                    continue;
                }
                Ok(ratio) => {
                    if ratio + 1e-9 < minimum {
                        problems.push(format!(
                            "[color:{}] {} {}/{} contrast {:.2}:1 is below {:.1}:1",
                            product, role, foreground, background, ratio, minimum
                        ));
                    }
                    ratios.insert(foreground.to_string(), ratio);
                }
            }
        }
        if let (Some(muted), Some(fg)) = (ratios.get("Muted Foreground"), ratios.get("Foreground"))
        {
            if muted > &(fg + 1e-9) {
                problems.push(format!(
                    "[color:{}] muted text contrast exceeds primary text contrast",
                    product
                ));
            }
        }
        let value = row
            .get("Destructive")
            .cloned()
            .unwrap_or_default()
            .trim_start_matches('#')
            .to_string();
        if value.len() >= 6 {
            if let (Ok(red), Ok(green), Ok(blue)) = (
                u8::from_str_radix(&value[0..2], 16),
                u8::from_str_radix(&value[2..4], 16),
                u8::from_str_radix(&value[4..6], 16),
            ) {
                let (r, g, b) = (red as f64, green as f64, blue as f64);
                if g > r * 1.1 && g > b * 1.1 {
                    problems.push(format!(
                        "[color:{}] Destructive token is success green",
                        product
                    ));
                }
            }
        }
    }
}

fn check_chart_contract(rows: &[Row], problems: &mut Vec<String>) {
    for row in rows {
        let data_type = row.get("Data Type").cloned().unwrap_or_default();
        if row.get("Accessibility Grade").map(|s| s.as_str())
            != Some("deprecated: use Accessibility Risk")
        {
            problems.push(format!(
                "[chart:{}] invalid deprecated Accessibility Grade",
                data_type
            ));
        }
        let risk = row.get("Accessibility Risk").cloned().unwrap_or_default();
        if !CHART_RISKS.contains(&risk.as_str()) {
            problems.push(format!("[chart:{}] invalid Accessibility Risk", data_type));
        }
        let fallback = format!(
            "{} {}",
            row.get("Accessibility Notes").cloned().unwrap_or_default(),
            row.get("A11y Fallback").cloned().unwrap_or_default()
        )
        .to_lowercase();
        if !chart_text_fallback().is_match(&fallback) {
            problems.push(format!(
                "[chart:{}] missing text/table/list fallback",
                data_type
            ));
        }
        if !chart_non_color_guidance().is_match(&fallback) {
            problems.push(format!(
                "[chart:{}] missing non-color distinction guidance",
                data_type
            ));
        }
        if !row
            .get("Interactive Level")
            .map(|s| s.trim())
            .unwrap_or("")
            .is_empty()
            && !fallback.contains("keyboard")
        {
            problems.push(format!(
                "[chart:{}] missing keyboard interaction equivalent",
                data_type
            ));
        }
        if wcag_grade().is_match(&fallback) {
            problems.push(format!(
                "[chart:{}] fallback claims WCAG conformance",
                data_type
            ));
        }
    }
}

// ---------- typography ----------

fn font_families(url: &str) -> Vec<String> {
    parse_qs(&urlsplit(url).query)
        .get("family")
        .cloned()
        .unwrap_or_default()
}

fn font_names(family_declarations: &[String]) -> HashSet<String> {
    family_declarations
        .iter()
        .map(|d| d.split(':').next().unwrap_or("").replace('+', " "))
        .collect()
}

fn configured_font_names(config: &str) -> HashSet<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| re(r"'([^']+)'"));
    re.captures_iter(config).map(|c| c[1].to_string()).collect()
}

fn imported_weights(family_declarations: &[String]) -> BTreeSet<String> {
    let mut weights = BTreeSet::new();
    for declaration in family_declarations {
        if !declaration.contains(':') || !declaration.contains('@') {
            continue;
        }
        let axis_values = declaration.split_once(':').map(|x| x.1).unwrap_or("");
        let mut parts = axis_values.splitn(2, '@');
        let axes = parts.next().unwrap_or("");
        let values = parts.next().unwrap_or("");
        if axes.contains("wght") {
            weights.extend(font_weights(values));
        }
    }
    weights
}

fn declared_weights(notes: &str) -> BTreeSet<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| re_i(r"(?:weights?|strictly)[^.;]{0,100}"));
    let mut weights = BTreeSet::new();
    for m in re.find_iter(notes) {
        weights.extend(font_weights(m.as_str()));
    }
    weights
}

fn check_typography_contract(rows: &[Row], problems: &mut Vec<String>) {
    for row in rows {
        let pairing = row.get("Font Pairing Name").cloned().unwrap_or_default();
        let font_url = row.get("Google Fonts URL").cloned().unwrap_or_default();
        let url_families = font_families(&font_url);
        let families = font_names(&url_families);
        let configured =
            configured_font_names(row.get("Tailwind Config").map(|s| s.as_str()).unwrap_or(""));
        let named: HashSet<String> = [
            row.get("Heading Font").cloned().unwrap_or_default(),
            row.get("Body Font").cloned().unwrap_or_default(),
        ]
        .into_iter()
        .collect();
        if !named.is_subset(&families) || !named.is_subset(&configured) {
            problems.push(format!(
                "[typography:{}] named/imported/configured fonts differ",
                pairing
            ));
        }
        let css_import = row.get("CSS Import").cloned().unwrap_or_default();
        let import_match = css_import_re().captures(&css_import);
        let import_url = match import_match {
            Some(c) => c
                .get(1)
                .or_else(|| c.get(2))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default(),
            None => {
                problems.push(format!("[typography:{}] invalid CSS Import", pairing));
                continue;
            }
        };
        let import_families = font_families(&import_url);
        let mut sorted_url = url_families.clone();
        let mut sorted_import = import_families.clone();
        sorted_url.sort();
        sorted_import.sort();
        if sorted_url != sorted_import {
            problems.push(format!(
                "[typography:{}] Google URL and CSS Import differ",
                pairing
            ));
        }
        let imported = imported_weights(&url_families);
        let declared = declared_weights(row.get("Notes").map(|s| s.as_str()).unwrap_or(""));
        if !declared.is_empty() && !declared.is_subset(&imported) {
            let missing: Vec<String> = declared.difference(&imported).cloned().collect();
            problems.push(format!(
                "[typography:{}] recommended weights not imported: {}",
                pairing,
                missing.join(", ")
            ));
        }
    }
}

// ---------- catalog ----------

fn load_catalog_json(catalog: &Catalog, name: &str, problems: &mut Vec<String>) -> Value {
    let path = catalog.path(name);
    let payload = std::fs::read(&path)
        .map_err(|e| e.to_string())
        .and_then(|b| String::from_utf8(b).map_err(|e| e.to_string()))
        .and_then(|t| serde_json::from_str::<Value>(&t).map_err(|e| e.to_string()));
    let payload = match payload {
        Ok(v) => v,
        Err(e) => {
            problems.push(format!("[catalog:{}] invalid JSON: {}", name, e));
            return json!({});
        }
    };
    if !payload.is_object() || payload.get("schemaVersion") != Some(&json!(1)) {
        problems.push(format!(
            "[catalog:{}] schemaVersion 1 object is required",
            name
        ));
        return json!({});
    }
    payload
}

fn valid_google_fonts_exclusion_source(value: &Value) -> bool {
    let s = match value.as_str() {
        Some(s) => s,
        None => return false,
    };
    let parsed = urlsplit(s);
    if parsed.port_present && !parsed.port_valid {
        return false;
    }
    let has_credentials_or_port =
        parsed.username.is_some() || parsed.password.is_some() || parsed.port_present;
    if parsed.scheme != "https" || has_credentials_or_port {
        return false;
    }
    if parsed.hostname == "fonts.google.com" {
        return !parsed.path.is_empty();
    }
    parsed.hostname == "github.com"
        && (parsed.path == "/google/fonts" || parsed.path.starts_with("/google/fonts/"))
}

fn check_font_catalog(
    rows: &[Row],
    licenses: &Value,
    typography: &[Row],
    problems: &mut Vec<String>,
) {
    let families: Vec<String> = rows
        .iter()
        .map(|r| r.get("Family").cloned().unwrap_or_default())
        .collect();
    let family_set: HashSet<String> = families.iter().cloned().collect();
    if families.is_empty() || family_set.len() != families.len() || family_set.contains("") {
        problems
            .push("[catalog:google-fonts] Family values must be non-empty and unique".to_string());
    }
    let mut available: HashMap<String, HashSet<String>> = HashMap::new();
    for row in rows {
        let family = row.get("Family").cloned().unwrap_or_default();
        if !catalog_date(&json!(row.get("Date Added").cloned().unwrap_or_default()))
            || !catalog_date(&json!(row
                .get("Last Modified")
                .cloned()
                .unwrap_or_default()))
        {
            problems.push(format!(
                "[catalog:google-fonts:{}] invalid or suspicious date",
                family
            ));
        }
        let expected_url = format!("https://fonts.google.com/specimen/{}", quote_plus(&family));
        if row.get("Google Fonts URL").map(|s| s.as_str()) != Some(expected_url.as_str()) {
            problems.push(format!(
                "[catalog:google-fonts:{}] invalid specimen URL",
                family
            ));
        }
        let styles: HashSet<String> =
            split_field(row.get("Styles").map(|s| s.as_str()).unwrap_or(""), '|')
                .into_iter()
                .map(|s| s.strip_suffix('i').unwrap_or(&s).to_string())
                .collect();
        if styles.is_empty() {
            problems.push(format!(
                "[catalog:google-fonts:{}] Styles cannot be empty",
                family
            ));
        }
        available.insert(family, styles);
    }

    let source = licenses.get("source");
    let valid_source = source
        .and_then(Value::as_object)
        .map(|s| {
            s.get("repository") == Some(&json!("https://github.com/google/fonts"))
                && s.get("metadataFile") == Some(&json!("METADATA.pb"))
                && fullmatch(
                    r"[0-9a-f]{40}",
                    s.get("revision").and_then(Value::as_str).unwrap_or(""),
                )
        })
        .unwrap_or(false);
    if !valid_source {
        problems.push("[catalog:google-font-licenses] invalid source revision".to_string());
    }
    let entries = licenses.get("families");
    let excluded = licenses.get("excludedFamilies");
    if licenses.get("familyCount") != Some(&json!(rows.len()))
        || !entries.map(|e| e.is_array()).unwrap_or(false)
        || !excluded.map(|e| e.is_array()).unwrap_or(false)
    {
        problems.push("[catalog:google-font-licenses] invalid counts or arrays".to_string());
        return;
    }
    let mut licensed_names: HashSet<String> = HashSet::new();
    for item in entries
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        if !item.is_object() {
            problems
                .push("[catalog:google-font-licenses] family entry must be an object".to_string());
            continue;
        }
        let name = item.get("name").and_then(Value::as_str).unwrap_or("");
        let name_value = item.get("name").cloned().unwrap_or(Value::Null);
        if name.is_empty()
            || !name_value.is_string()
            || licensed_names.contains(name)
            || !FONT_LICENSES.contains(&item.get("license").and_then(Value::as_str).unwrap_or(""))
            || item.get("status") != Some(&json!("active"))
            || !catalog_date(item.get("date_added").unwrap_or(&Value::Null))
            || !catalog_date(item.get("verifiedAt").unwrap_or(&Value::Null))
        {
            problems.push(format!(
                "[catalog:google-font-licenses:{}] invalid active family",
                name
            ));
        }
        licensed_names.insert(name.to_string());
    }
    if licensed_names != family_set {
        problems.push(
            "[catalog:google-font-licenses] active families must match google-fonts.csv"
                .to_string(),
        );
    }
    let mut excluded_names: HashSet<String> = HashSet::new();
    for item in excluded
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let name = item.get("name").and_then(Value::as_str).unwrap_or("");
        let source_v = item.get("source").cloned().unwrap_or(Value::Null);
        if name.is_empty()
            || !item.get("name").map(|v| v.is_string()).unwrap_or(false)
            || excluded_names.contains(name)
            || family_set.contains(name)
            || item.get("status") != Some(&json!("needs-review"))
            || !item
                .get("reason")
                .map(|v| v.is_string() && !v.as_str().unwrap_or("").is_empty())
                .unwrap_or(false)
            || !valid_google_fonts_exclusion_source(&source_v)
            || !catalog_date(item.get("verifiedAt").unwrap_or(&Value::Null))
        {
            problems.push(format!(
                "[catalog:google-font-licenses:{}] invalid exclusion",
                name
            ));
        }
        excluded_names.insert(name.to_string());
    }

    for row in typography {
        let pairing = row.get("Font Pairing Name").cloned().unwrap_or_default();
        let declarations = font_families(
            row.get("Google Fonts URL")
                .map(|s| s.as_str())
                .unwrap_or(""),
        );
        for declaration in &declarations {
            let family = declaration
                .split(':')
                .next()
                .unwrap_or("")
                .replace('+', " ");
            if !available.contains_key(&family) {
                problems.push(format!(
                    "[typography:{}] font absent from approved catalog: {}",
                    pairing, family
                ));
                continue;
            }
            let mut weights = imported_weights(std::slice::from_ref(declaration));
            if weights.is_empty() {
                weights.insert("400".to_string());
            }
            let avail = &available[&family];
            if !weights.iter().all(|w| avail.contains(w)) {
                let missing: Vec<String> = weights
                    .iter()
                    .filter(|w| !avail.contains(*w))
                    .cloned()
                    .collect();
                problems.push(format!(
                    "[typography:{}] catalog lacks {} weights: {}",
                    pairing,
                    family,
                    missing.join(", ")
                ));
            }
        }
    }
}

fn check_phosphor_catalog(curated: &[Row], manifest: &Value, problems: &mut Vec<String>) {
    let source = manifest.get("source").and_then(Value::as_object);
    let imports = manifest.get("reactImports").and_then(Value::as_object);
    let empty: Map<String, Value> = Map::new();
    let source = source.unwrap_or(&empty);
    let imports = imports.unwrap_or(&empty);
    let icons = manifest.get("icons");
    if source.get("package") != Some(&json!("@phosphor-icons/core"))
        || source.get("version") != Some(&json!("2.1.1"))
    {
        problems.push("[catalog:phosphor] unpinned core package".to_string());
    }
    if source.get("reactPackage") != Some(&json!("@phosphor-icons/react"))
        || source.get("reactVersion") != Some(&json!("2.1.10"))
    {
        problems.push("[catalog:phosphor] unpinned React package".to_string());
    }
    let weights_ok = manifest
        .get("weights")
        .and_then(Value::as_array)
        .map(|w| {
            let set: HashSet<&str> = w.iter().filter_map(Value::as_str).collect();
            PHOSPHOR_WEIGHTS.iter().all(|p| set.contains(p)) && set.len() == PHOSPHOR_WEIGHTS.len()
        })
        .unwrap_or(false);
    let imports_ok = imports.len() == 2
        && imports.get("clientModule") == Some(&json!("@phosphor-icons/react"))
        && imports.get("ssrModule") == Some(&json!("@phosphor-icons/react/ssr"));
    let icons_list = icons.and_then(Value::as_array).cloned().unwrap_or_default();
    if manifest.get("status") != Some(&json!("active"))
        || !catalog_date(manifest.get("verifiedAt").unwrap_or(&Value::Null))
        || !weights_ok
        || !imports_ok
        || !icons.map(|i| i.is_array()).unwrap_or(false)
        || manifest.get("iconCount") != Some(&json!(icons_list.len()))
    {
        problems.push("[catalog:phosphor] invalid snapshot metadata".to_string());
        return;
    }
    let mut names: HashMap<String, String> = HashMap::new();
    let mut components: HashSet<String> = HashSet::new();
    for item in &icons_list {
        let name = item.get("name").and_then(Value::as_str).unwrap_or("");
        let component = item.get("component").and_then(Value::as_str).unwrap_or("");
        if name.is_empty()
            || names.contains_key(name)
            || component.is_empty()
            || components.contains(component)
            || item.get("clientImport")
                != Some(&json!(format!(
                    "import {{ {} }} from \"@phosphor-icons/react\"",
                    component
                )))
            || item.get("ssrImport")
                != Some(&json!(format!(
                    "import {{ {} }} from \"@phosphor-icons/react/ssr\"",
                    component
                )))
        {
            problems.push(format!(
                "[catalog:phosphor:{}] invalid identity or imports",
                name
            ));
        }
        names.insert(name.to_string(), component.to_string());
        components.insert(component.to_string());
    }
    let phosphor_rows: Vec<&Row> = curated
        .iter()
        .filter(|r| r.get("Library").map(|s| s.as_str()) == Some("Phosphor"))
        .collect();
    if manifest.get("curatedValidatedCount") != Some(&json!(phosphor_rows.len())) {
        problems.push("[catalog:phosphor] curated validation count is stale".to_string());
    }
    let component_re = re(r"import\s*\{\s*([A-Za-z0-9]+)");
    for row in phosphor_rows {
        let name = row.get("Icon Name").cloned().unwrap_or_default();
        let import_code = row.get("Import Code").cloned().unwrap_or_default();
        let component = component_re
            .captures(&import_code)
            .map(|c| c[1].to_string())
            .unwrap_or_default();
        if names.get(&name) != Some(&component) {
            problems.push(format!(
                "[catalog:phosphor:{}] curated icon is absent or mismatched",
                name
            ));
        }
    }
}

fn check_catalog_summary(
    catalog: &Catalog,
    summary: &Value,
    licenses: &Value,
    phosphor: &Value,
    problems: &mut Vec<String>,
) {
    if !catalog_date(summary.get("verifiedAt").unwrap_or(&Value::Null)) {
        problems.push("[catalog:summary] invalid verifiedAt".to_string());
    }
    let empty: Map<String, Value> = Map::new();
    let counts = summary
        .get("counts")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    let rows_of = |name: &str| -> Vec<Row> { catalog.rows(name).unwrap_or_default() };
    let styles = rows_of("styles.csv");
    let count_status = |want: &str| -> usize {
        styles
            .iter()
            .filter(|r| r.get("Status").map(|s| s.as_str()) == Some(want))
            .count()
    };
    let searchable = styles
        .iter()
        .filter(|r| r.get("Status").map(|s| s.as_str()) != Some("deprecated"))
        .count();
    let mut styles_counts = Map::new();
    styles_counts.insert("total".into(), json!(styles.len()));
    styles_counts.insert("searchable".into(), json!(searchable));
    styles_counts.insert("active".into(), json!(count_status("active")));
    styles_counts.insert("supplemental".into(), json!(count_status("supplemental")));
    styles_counts.insert("deprecated".into(), json!(count_status("deprecated")));
    let stack_guidelines: usize = STACK_CONFIG.iter().map(|(_, f)| rows_of(f).len()).sum();

    let mut expected: Vec<(&str, Value)> = vec![
        ("styles", Value::Object(styles_counts)),
        ("products", json!(rows_of("products.csv").len())),
        ("palettes", json!(rows_of("colors.csv").len())),
        ("reasoningProfiles", json!(rows_of(REASONING_FILE).len())),
        ("fontPairings", json!(rows_of("typography.csv").len())),
        ("googleFonts", json!(rows_of("google-fonts.csv").len())),
        ("curatedIcons", json!(rows_of("icons.csv").len())),
        (
            "upstreamPhosphorIcons",
            phosphor.get("iconCount").cloned().unwrap_or(Value::Null),
        ),
        ("uxGuidelines", json!(rows_of("ux-guidelines.csv").len())),
        ("motionPresets", json!(rows_of("motion.csv").len())),
        ("chartTypes", json!(rows_of("charts.csv").len())),
        ("stacks", json!(STACK_CONFIG.len())),
        ("stackGuidelines", json!(stack_guidelines)),
    ];
    for (key, value) in expected.drain(..) {
        if counts.get(key) != Some(&value) {
            problems.push(format!("[catalog:summary] stale count for {}", key));
        }
    }
    let snapshots = summary
        .get("snapshots")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    for name in [
        "google-fonts.csv",
        "google-font-licenses.json",
        "icons.csv",
        "phosphor-icons-upstream.json",
    ] {
        let bytes = std::fs::read(catalog.path(name)).unwrap_or_default();
        let normalized: Vec<u8> = {
            let mut v = Vec::with_capacity(bytes.len());
            let mut i = 0;
            while i < bytes.len() {
                if i + 1 < bytes.len() && bytes[i] == b'\r' && bytes[i + 1] == b'\n' {
                    v.push(b'\n');
                    i += 2;
                } else {
                    v.push(bytes[i]);
                    i += 1;
                }
            }
            v
        };
        let digest = format!("{:x}", Sha256::digest(&normalized));
        let expected_snapshot = {
            let mut m = Map::new();
            m.insert("sha256".to_string(), json!(digest));
            Value::Object(m)
        };
        if snapshots.get(name) != Some(&expected_snapshot) {
            problems.push(format!("[catalog:summary] stale snapshot for {}", name));
        }
    }
    let policy = summary.get("promotionPolicy");
    let expected_policy = json!({
        "changedFamilySetRequiresExplicitApproval": true,
        "relevanceGateRequired": true,
        "unlicensedFamiliesExcluded": true,
    });
    if policy != Some(&expected_policy) {
        problems.push("[catalog:summary] invalid promotion policy".to_string());
    }
    let mut pending: Vec<Value> = licenses
        .get("excludedFamilies")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|i| i.is_object())
        .map(|i| {
            json!({
                "family": i.get("name").cloned().unwrap_or(Value::Null),
                "reason": i.get("reason").cloned().unwrap_or(Value::Null),
            })
        })
        .collect();
    pending.sort_by(|a, b| {
        let fa = a
            .get("family")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_lowercase();
        let fb = b
            .get("family")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_lowercase();
        fa.cmp(&fb)
    });
    if summary.get("pendingCandidates") != Some(&Value::Array(pending)) {
        problems.push("[catalog:summary] pending candidates do not match exclusions".to_string());
    }
}

fn check_catalog_contract(
    catalog: &Catalog,
    domain_rows: &HashMap<String, Vec<Row>>,
    problems: &mut Vec<String>,
) {
    if catalog
        .path(".google-font-refresh.incomplete.json")
        .exists()
    {
        problems
            .push("[catalog:google-fonts] incomplete refresh marker requires review".to_string());
    }
    let licenses = load_catalog_json(catalog, "google-font-licenses.json", problems);
    let phosphor = load_catalog_json(catalog, "phosphor-icons-upstream.json", problems);
    let summary = load_catalog_json(catalog, "catalog-summary.json", problems);
    if licenses.is_object() && !licenses.as_object().unwrap().is_empty() {
        check_font_catalog(
            domain_rows
                .get("google-fonts")
                .cloned()
                .unwrap_or_default()
                .as_slice(),
            &licenses,
            domain_rows
                .get("typography")
                .cloned()
                .unwrap_or_default()
                .as_slice(),
            problems,
        );
    }
    if phosphor.is_object() && !phosphor.as_object().unwrap().is_empty() {
        check_phosphor_catalog(
            domain_rows
                .get("icons")
                .cloned()
                .unwrap_or_default()
                .as_slice(),
            &phosphor,
            problems,
        );
    }
    if summary.is_object() && !summary.as_object().unwrap().is_empty() {
        check_catalog_summary(catalog, &summary, &licenses, &phosphor, problems);
    }
}

fn check_icon_contract(rows: &[Row], problems: &mut Vec<String>) {
    for row in rows {
        let icon = row.get("Icon Name").cloned().unwrap_or_default();
        let role = row.get("Semantic Role").cloned().unwrap_or_default();
        let contexts: HashSet<String> = split_field(
            row.get("Allowed Contexts")
                .map(|s| s.as_str())
                .unwrap_or(""),
            '|',
        )
        .into_iter()
        .collect();
        let usage = row.get("Usage").cloned().unwrap_or_default();
        if !ICON_ROLES.contains(&role.as_str()) {
            problems.push(format!("[icons:{}] invalid Semantic Role '{}'", icon, role));
        }
        let expected: HashSet<String> = ICON_CONTEXTS.iter().map(|s| s.to_string()).collect();
        if contexts != expected {
            problems.push(format!("[icons:{}] incomplete contextual semantics", icon));
        }
        if re(r"[\u{3400}-\u{9fff}]").is_match(&usage) {
            problems.push(format!("[icons:{}] Usage must use canonical English", icon));
        }
        let imports = row.get("Import Code").cloned().unwrap_or_default();
        if imports.contains("IconName") || !icon_import_re().is_match(&imports) {
            problems.push(format!(
                "[icons:{}] invalid or placeholder icon import",
                icon
            ));
        }
        for pattern in ICON_USAGE_REQUIREMENTS {
            if !re_i(pattern).is_match(&usage) {
                problems.push(format!(
                    "[icons:{}] incomplete contextual accessibility guidance",
                    icon
                ));
                break;
            }
        }
    }
}

fn check_ux_contract(rows: &[Row], problems: &mut Vec<String>) {
    let ux_by_issue: HashMap<String, &Row> = rows
        .iter()
        .map(|r| (r.get("Issue").cloned().unwrap_or_default(), r))
        .collect();
    let present: HashSet<String> = ux_by_issue.keys().cloned().collect();
    let required: HashSet<String> = REQUIRED_UX_GUIDANCE
        .iter()
        .map(|(i, _)| i.to_string())
        .collect();
    let mut missing: Vec<String> = required.difference(&present).cloned().collect();
    missing.sort();
    for issue in missing {
        problems.push(format!("[ux] missing WCAG 2.2 guidance '{}'", issue));
    }
    for (issue, platform) in REQUIRED_UX_GUIDANCE {
        if let Some(row) = ux_by_issue.get(*issue) {
            let severity = row.get("Severity").cloned().unwrap_or_default();
            if row.get("Platform").map(|s| s.as_str()) != Some(*platform)
                || !["Medium", "High", "Critical"].contains(&severity.as_str())
            {
                problems.push(format!("[ux:{}] shifted or invalid semantic fields", issue));
            }
        }
    }
}

fn check_motion_contract(rows: &[Row], problems: &mut Vec<String>) {
    for row in rows {
        let text = row
            .values()
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        if !text.contains("reduced-motion") && !text.contains("user-controlled") {
            problems.push(format!(
                "[motion:{}] missing explicit motion opt-out",
                row.get("No").cloned().unwrap_or_default()
            ));
        }
    }
}

fn check_app_interface_contract(rows: &[Row], problems: &mut Vec<String>) {
    let native_target = rows
        .iter()
        .find(|r| r.get("Issue").map(|s| s.as_str()) == Some("Touch Target Size"));
    let joined = native_target
        .map(|r| r.values().cloned().collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    let found: HashSet<String> = re(r"44pt|48dp")
        .find_iter(&joined)
        .map(|m| m.as_str().to_string())
        .collect();
    let required: HashSet<String> = ["44pt", "48dp"].iter().map(|s| s.to_string()).collect();
    if !required.is_subset(&found) {
        problems
            .push("[web:Touch Target Size] must distinguish iOS 44pt and Android 48dp".to_string());
    }
}

fn check_react_contract(rows: &[Row], problems: &mut Vec<String>) {
    let react_text = rows
        .iter()
        .map(|r| r.values().cloned().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n");
    if react_text.contains("useLatest") {
        problems
            .push("[react] unqualified community useLatest guidance is not allowed".to_string());
    }
    let effect_event = rows
        .iter()
        .find(|r| r.get("Issue").map(|s| s.as_str()) == Some("Effect Events"));
    let effect_text = effect_event
        .map(|r| {
            r.values()
                .cloned()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        })
        .unwrap_or_default();
    if !effect_text.contains("inside effects") || !effect_text.contains("dependencies") {
        problems.push(
            "[react:Effect Events] current scope/dependency guidance is required".to_string(),
        );
    }
}

fn check_landing_claims(rows: &[Row], problems: &mut Vec<String>) {
    for row in rows {
        let optimization = row
            .get("Conversion Optimization")
            .cloned()
            .unwrap_or_default();
        if optimization.contains('%')
            || landing_quantified_claim().is_match(&optimization)
            || optimization.to_lowercase().contains("best conversion")
        {
            problems.push(format!(
                "[landing:{}] unsupported quantitative claim",
                row.get("Pattern Name").cloned().unwrap_or_default()
            ));
        }
    }
}

fn check_core_data_contract(domain_rows: &HashMap<String, Vec<Row>>, problems: &mut Vec<String>) {
    let get = |k: &str| -> &[Row] { domain_rows.get(k).map(|v| v.as_slice()).unwrap_or(&[]) };
    check_color_contract(get("color"), problems);
    check_chart_contract(get("chart"), problems);
    check_typography_contract(get("typography"), problems);
    check_icon_contract(get("icons"), problems);
    check_ux_contract(get("ux"), problems);
    check_motion_contract(get("gsap"), problems);
    check_app_interface_contract(get("web"), problems);
    check_react_contract(get("react"), problems);
    check_landing_claims(get("landing"), problems);
}

// ---------- provenance ----------

fn valid_provenance_source(
    source: &Value,
    source_index: usize,
    identity: &(String, String),
    problems: &mut Vec<String>,
) -> bool {
    let obj = match source.as_object() {
        Some(o) => o,
        None => {
            problems.push(format!(
                "[provenance] source {} for {:?} must be an object",
                source_index,
                identity_display(identity)
            ));
            return false;
        }
    };
    let source_type = obj.get("type").and_then(Value::as_str).unwrap_or("");
    let refv = obj.get("ref").and_then(Value::as_str);
    if !PROVENANCE_SOURCE_TYPES.contains(&source_type) || refv.is_none() {
        problems.push(format!(
            "[provenance] invalid source for {}",
            identity_display(identity)
        ));
        return false;
    }
    let refv = refv.unwrap();
    if source_type == "official" {
        let parsed = urlsplit(refv);
        if parsed.scheme != "https" || !OFFICIAL_SOURCE_HOSTS.contains(&parsed.hostname.as_str()) {
            problems.push(format!(
                "[provenance] unapproved official source for {}",
                identity_display(identity)
            ));
            return false;
        }
    } else if refv.is_empty() || !urlsplit(refv).scheme.is_empty() {
        problems.push(format!(
            "[provenance] derived source must use a local dataset reference for {}",
            identity_display(identity)
        ));
        return false;
    }
    true
}

fn identity_display(identity: &(String, String)) -> String {
    format!("('{}', '{}')", identity.0, identity.1)
}

fn valid_dataset_source_key(
    catalog: &Catalog,
    source_file: &str,
    source_key: &Map<String, Value>,
    identity: &(String, String),
    problems: &mut Vec<String>,
) -> bool {
    if !CORE_PROVENANCE_FILES.contains(&source_file) {
        problems.push(format!(
            "[provenance] unknown dataset-contract sourceFile for {}",
            identity_display(identity)
        ));
        return false;
    }
    let table = match catalog.table(source_file) {
        Ok(t) => t,
        Err(_) => {
            problems.push(format!(
                "[provenance] unreadable sourceFile for {}",
                identity_display(identity)
            ));
            return false;
        }
    };
    let scope = source_key.get("Scope").and_then(Value::as_str);
    if !scope.map(|s| s.starts_with("No ")).unwrap_or(false) {
        problems.push(format!(
            "[provenance] dataset scope must bind rows for {}",
            identity_display(identity)
        ));
        return false;
    }
    let scope = scope.unwrap();
    let row_part = scope.split(';').next().unwrap_or("");
    let mut referenced: HashSet<i64> = re(r"\b\d+\b")
        .find_iter(row_part)
        .map(|m| m.as_str().parse().unwrap_or(0))
        .collect();
    for caps in re(r"(\d+)\s*-\s*(\d+)").captures_iter(row_part) {
        let (a, b): (i64, i64) = (caps[1].parse().unwrap_or(0), caps[2].parse().unwrap_or(0));
        for v in a..=b {
            referenced.insert(v);
        }
    }
    let row_ids: HashSet<i64> = table
        .rows
        .iter()
        .filter(|r| {
            r.get("No")
                .map(|v| v.chars().all(|c| c.is_ascii_digit()))
                .unwrap_or(false)
        })
        .filter_map(|r| r.get("No").and_then(|v| v.parse().ok()))
        .collect();
    if referenced.is_empty() || !referenced.is_subset(&row_ids) {
        problems.push(format!(
            "[provenance] dataset scope references unknown rows for {}",
            identity_display(identity)
        ));
        return false;
    }
    let semantic_headers: Vec<&String> = table.headers.iter().filter(|h| *h != "No").collect();
    let has_field_ref = semantic_headers.iter().any(|h| {
        let pattern = format!("(?i)\\b{}\\b", regex::escape(h));
        re(&pattern).is_match(scope)
    });
    if !has_field_ref {
        problems.push(format!(
            "[provenance] dataset scope must bind source fields for {}",
            identity_display(identity)
        ));
        return false;
    }
    true
}

fn valid_catalog_source_key(
    catalog: &Catalog,
    source_file: &str,
    source_key: &Map<String, Value>,
    identity: &(String, String),
    problems: &mut Vec<String>,
) -> bool {
    if !CATALOG_PROVENANCE_FILES.contains(&source_file) {
        problems.push(format!(
            "[provenance] unknown catalog sourceFile for {}",
            identity_display(identity)
        ));
        return false;
    }
    if !catalog.path(source_file).is_file() {
        problems.push(format!(
            "[provenance] missing catalog sourceFile for {}",
            identity_display(identity)
        ));
        return false;
    }
    let snapshot = source_key.get("Snapshot").and_then(Value::as_str);
    let count = source_key.get("Count");
    let count_ok = match count {
        Some(Value::Number(n)) => n.as_i64().map(|v| v > 0).unwrap_or(false),
        _ => false,
    };
    if snapshot != Some("catalog-summary.json") || !count_ok {
        problems.push(format!(
            "[provenance] catalog sourceKey must bind snapshot and count for {}",
            identity_display(identity)
        ));
        return false;
    }
    let expected_count: Result<Option<i64>, ()> = if source_file.ends_with(".csv") {
        catalog
            .table(source_file)
            .map(|t| Some(t.rows.len() as i64))
            .map_err(|_| ())
    } else {
        std::fs::read_to_string(catalog.path(source_file))
            .map_err(|_| ())
            .and_then(|t| serde_json::from_str::<Value>(&t).map_err(|_| ()))
            .map(|v| v.get("iconCount").and_then(Value::as_i64))
    };
    match expected_count {
        Err(()) => {
            problems.push(format!(
                "[provenance] unreadable catalog sourceFile for {}",
                identity_display(identity)
            ));
            false
        }
        Ok(expected) => {
            // `count != expected_count` — count was validated int>0 above.
            if count.and_then(Value::as_i64) != expected {
                problems.push(format!(
                    "[provenance] catalog count is stale for {}",
                    identity_display(identity)
                ));
                return false;
            }
            true
        }
    }
}

fn check_provenance(
    catalog: &Catalog,
    reasoning: &[Row],
    styles: &[Row],
    problems: &mut Vec<String>,
) {
    let path = catalog.path("data-provenance.json");
    let payload = std::fs::read_to_string(&path)
        .map_err(|e| e.to_string())
        .and_then(|t| serde_json::from_str::<Value>(&t).map_err(|e| e.to_string()));
    let payload = match payload {
        Ok(v) => v,
        Err(e) => {
            problems.push(format!("[provenance] invalid data-provenance.json: {}", e));
            return;
        }
    };
    let records = payload.get("records").and_then(Value::as_array);
    if !payload.is_object() || payload.get("schemaVersion") != Some(&json!(1)) || records.is_none()
    {
        problems.push("[provenance] schemaVersion 1 and records array are required".to_string());
        return;
    }
    let records = records.unwrap();
    let mut covered: HashSet<String> = HashSet::new();
    let mut covered_styles: HashSet<String> = HashSet::new();
    let mut identities: HashSet<(String, String)> = HashSet::new();
    let mut valid_records: Vec<&Value> = Vec::new();
    for (index, record) in records.iter().enumerate() {
        if !record.is_object() {
            problems.push(format!("[provenance] record {} must be an object", index));
            continue;
        }
        let identity = (
            record
                .get("entityKind")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            record
                .get("entityId")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        );
        if !PROVENANCE_KINDS.contains(&identity.0.as_str())
            || !record
                .get("entityId")
                .map(|v| v.is_string() && !v.as_str().unwrap_or("").is_empty())
                .unwrap_or(false)
        {
            problems.push(format!("[provenance] invalid identity at record {}", index));
            continue;
        }
        if identities.contains(&identity) {
            problems.push(format!(
                "[provenance] duplicate identity {}",
                identity_display(&identity)
            ));
        }
        identities.insert(identity.clone());
        if !PROVENANCE_STATUSES
            .contains(&record.get("status").and_then(Value::as_str).unwrap_or(""))
        {
            problems.push(format!(
                "[provenance] invalid status for {}",
                identity_display(&identity)
            ));
        }
        if !PROVENANCE_SLAS.contains(&record.get("sla").and_then(Value::as_str).unwrap_or("")) {
            problems.push(format!(
                "[provenance] invalid sla for {}",
                identity_display(&identity)
            ));
        }
        let verified = record
            .get("verifiedAt")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !valid_date(verified) {
            problems.push(format!(
                "[provenance] invalid verifiedAt for {}",
                identity_display(&identity)
            ));
        }
        if !record
            .get("sourceFile")
            .map(|v| v.is_string() && !v.as_str().unwrap_or("").is_empty())
            .unwrap_or(false)
        {
            problems.push(format!(
                "[provenance] sourceFile required for {}",
                identity_display(&identity)
            ));
        }
        if !record
            .get("sourceKey")
            .map(|v| v.is_object() && !v.as_object().unwrap().is_empty())
            .unwrap_or(false)
        {
            problems.push(format!(
                "[provenance] sourceKey object required for {}",
                identity_display(&identity)
            ));
        }
        let applies_to = record.get("appliesTo").and_then(Value::as_array);
        let applies_ok = applies_to
            .map(|a| {
                !a.is_empty()
                    && a.iter().all(|v| {
                        v.as_str()
                            .map(|s| PROVENANCE_APPLIES_TO.contains(&s))
                            .unwrap_or(false)
                    })
            })
            .unwrap_or(false);
        if !applies_ok {
            problems.push(format!(
                "[provenance] invalid appliesTo for {}",
                identity_display(&identity)
            ));
        }
        if !valid_confidence(record.get("confidence").unwrap_or(&Value::Null)) {
            problems.push(format!(
                "[provenance] invalid confidence for {}",
                identity_display(&identity)
            ));
        }
        let sources = record.get("sources").and_then(Value::as_array);
        let mut source_list: Vec<Value> = vec![];
        match sources {
            Some(list) if !list.is_empty() => source_list = list.clone(),
            _ => {
                problems.push(format!(
                    "[provenance] sources required for {}",
                    identity_display(&identity)
                ));
            }
        }
        let valid_sources: Vec<Value> = source_list
            .iter()
            .enumerate()
            .filter(|(i, s)| valid_provenance_source(s, *i, &identity, problems))
            .map(|(_, s)| s.clone())
            .collect();
        let source_types: HashSet<&str> = valid_sources
            .iter()
            .filter_map(|s| s.get("type").and_then(Value::as_str))
            .collect();
        if record.get("sla") == Some(&json!("manual-verified"))
            && source_types.iter().all(|t| *t == "derived")
        {
            problems.push(format!(
                "[provenance] {} cannot be manual-verified from derived sources only",
                identity_display(&identity)
            ));
        }
        let source_key = record.get("sourceKey").and_then(Value::as_object);
        let empty: Map<String, Value> = Map::new();
        let source_key = source_key.unwrap_or(&empty);
        if identity.0 == "reasoning" {
            if let Some(cat) = source_key
                .get("UI_Category")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                covered.insert(cat.to_string());
            }
        }
        if identity.0 == "style" {
            if let Some(sid) = source_key
                .get("Style ID")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                covered_styles.insert(sid.to_string());
            }
        }
        let source_key_valid = match identity.0.as_str() {
            "dataset-contract" => valid_dataset_source_key(
                catalog,
                record
                    .get("sourceFile")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
                source_key,
                &identity,
                problems,
            ),
            "catalog-snapshot" => valid_catalog_source_key(
                catalog,
                record
                    .get("sourceFile")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
                source_key,
                &identity,
                problems,
            ),
            _ => true,
        };
        if !valid_sources.is_empty() && source_key_valid {
            valid_records.push(record);
        }
    }
    let new_rows: HashSet<String> = reasoning
        .iter()
        .filter(|r| r.get("No").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0) >= 162)
        .map(|r| r.get("UI_Category").cloned().unwrap_or_default())
        .collect();
    let missing: Vec<String> = new_rows.difference(&covered).cloned().collect();
    if !missing.is_empty() {
        let mut sorted = missing;
        sorted.sort();
        problems.push(format!(
            "[provenance] missing new reasoning rows: {}",
            sorted.join(", ")
        ));
    }
    let new_styles: HashSet<String> = styles
        .iter()
        .filter(|r| r.get("No").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0) > 85)
        .map(|r| r.get("Style ID").cloned().unwrap_or_default())
        .collect();
    let missing_styles: Vec<String> = new_styles.difference(&covered_styles).cloned().collect();
    if !missing_styles.is_empty() {
        let mut sorted = missing_styles;
        sorted.sort();
        problems.push(format!(
            "[provenance] missing new style rows: {}",
            sorted.join(", ")
        ));
    }
    let core_files: HashSet<String> = valid_records
        .iter()
        .filter(|r| {
            r.get("entityKind") == Some(&json!("dataset-contract"))
                && r.get("sources")
                    .and_then(Value::as_array)
                    .map(|ss| ss.iter().any(|s| s.get("type") == Some(&json!("official"))))
                    .unwrap_or(false)
        })
        .filter_map(|r| {
            r.get("sourceFile")
                .and_then(Value::as_str)
                .map(|s| s.to_string())
        })
        .collect();
    let required: HashSet<String> = CORE_PROVENANCE_FILES
        .iter()
        .map(|s| s.to_string())
        .collect();
    if !required.is_subset(&core_files) {
        let mut missing: Vec<String> = required.difference(&core_files).cloned().collect();
        missing.sort();
        problems.push(format!(
            "[provenance] missing official core dataset records: {}",
            missing.join(", ")
        ));
    }
    let catalog_ids: HashSet<String> = valid_records
        .iter()
        .filter(|r| {
            r.get("entityKind") == Some(&json!("catalog-snapshot"))
                && r.get("sources")
                    .and_then(Value::as_array)
                    .map(|ss| ss.iter().any(|s| s.get("type") == Some(&json!("official"))))
                    .unwrap_or(false)
        })
        .filter_map(|r| {
            r.get("entityId")
                .and_then(Value::as_str)
                .map(|s| s.to_string())
        })
        .collect();
    let required: HashSet<String> = CATALOG_PROVENANCE_IDS
        .iter()
        .map(|s| s.to_string())
        .collect();
    if !required.is_subset(&catalog_ids) {
        let mut missing: Vec<String> = required.difference(&catalog_ids).cloned().collect();
        missing.sort();
        problems.push(format!(
            "[provenance] missing official catalog snapshots: {}",
            missing.join(", ")
        ));
    }
}

fn check_stack_freshness_contract(stack: &str, rows: &[Row], problems: &mut Vec<String>) {
    let official = stack_official_hosts(stack);
    if official.is_empty() {
        return;
    }
    let mut active_count = 0;
    for row in rows {
        let identity = format!(
            "[stack:{}:{}]",
            stack,
            row.get("No").cloned().unwrap_or_else(|| "?".to_string())
        );
        let status = row.get("Status").cloned().unwrap_or_default();
        let applies_to = row
            .get("Applies To")
            .cloned()
            .unwrap_or_default()
            .trim()
            .to_lowercase();
        let expected = stack_applicability(stack);
        if applies_to.is_empty() || !applies_to.starts_with(stack) {
            problems.push(format!(
                "{} Applies To must start with '{}'",
                identity, stack
            ));
        } else if status == "active" && !applies_to.starts_with(expected) {
            problems.push(format!(
                "{} Applies To must target '{}'",
                identity, expected
            ));
        }
        if status == "active" {
            active_count += 1;
            if applies_to.contains("legacy") {
                problems.push(format!(
                    "{} active row cannot target legacy versions",
                    identity
                ));
            }
        } else if status == "deprecated" && !applies_to.contains("legacy") {
            problems.push(format!(
                "{} deprecated row must be visibly legacy",
                identity
            ));
        } else if status == "unverified" {
            problems.push(format!(
                "{} curated stack row cannot remain unverified",
                identity
            ));
        }

        let severity = row.get("Severity").cloned().unwrap_or_default();
        if !["Critical", "High"].contains(&severity.as_str()) {
            continue;
        }
        let docs_url = row.get("Docs URL").cloned().unwrap_or_default();
        let parsed = urlsplit(&docs_url);
        if parsed.scheme != "https" || !official.contains(&parsed.hostname.as_str()) {
            problems.push(format!(
                "{} Critical/High row requires an official Docs URL",
                identity
            ));
        }
        if !valid_date(row.get("Verified At").map(|s| s.as_str()).unwrap_or("")) {
            problems.push(format!(
                "{} Critical/High row requires ISO Verified At",
                identity
            ));
        }
    }
    if active_count == 0 && stack != "uwp" {
        problems.push(format!(
            "[stack:{}] requires at least one active current row",
            stack
        ));
    }
}

// ============ ENTRY ============

pub fn validate(catalog: &Catalog) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();
    let mut domain_rows: HashMap<String, Vec<Row>> = HashMap::new();

    for &(domain, file, search_cols, output_cols) in CSV_CONFIG.iter() {
        let (_, rows) = check_file(
            catalog,
            &format!("domain:{}", domain),
            file,
            search_cols,
            output_cols,
            &mut problems,
        );
        domain_rows.insert(domain.to_string(), rows);
    }

    for &(stack, file) in STACK_CONFIG.iter() {
        let (headers, rows) = check_file(
            catalog,
            &format!("stack:{}", stack),
            file,
            STACK_SEARCH_COLS,
            STACK_OUTPUT_COLS,
            &mut problems,
        );
        let required: HashSet<&str> = ["Applies To", "Status", "Verified At"]
            .into_iter()
            .collect();
        let header_set: HashSet<&str> = headers.iter().map(|s| s.as_str()).collect();
        if !headers.is_empty() && !required.is_subset(&header_set) {
            let mut missing: Vec<&&str> = required.difference(&header_set).collect();
            missing.sort();
            // Python prints a list repr with single quotes.
            let rendered = format!(
                "[{}]",
                missing
                    .iter()
                    .map(|s| format!("'{}'", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            problems.push(format!(
                "[stack:{}] missing contract fields {}",
                stack, rendered
            ));
        }
        for row in &rows {
            let status = row.get("Status").cloned().unwrap_or_default();
            let verified = row.get("Verified At").cloned().unwrap_or_default();
            if !STACK_STATUSES.contains(&status.as_str()) {
                problems.push(format!("[stack:{}] invalid Status '{}'", stack, status));
            }
            if status != "unverified" && !valid_date(&verified) {
                problems.push(format!(
                    "[stack:{}] active rows require ISO Verified At",
                    stack
                ));
            }
        }
        check_stack_freshness_contract(stack, &rows, &mut problems);
    }

    let reasoning = if catalog.path(REASONING_FILE).exists() {
        let (_, rows) = check_file(
            catalog,
            "reasoning",
            REASONING_FILE,
            &["UI_Category"],
            &["UI_Category", "Decision_Rules", "Reasoning", "Confidence"],
            &mut problems,
        );
        rows
    } else {
        problems.push(format!(
            "[reasoning] missing file: {}",
            catalog.path(REASONING_FILE).display()
        ));
        vec![]
    };

    let styles = domain_rows.get("style").cloned().unwrap_or_default();
    let products = domain_rows.get("product").cloned().unwrap_or_default();
    let colors = domain_rows.get("color").cloned().unwrap_or_default();
    let landing = domain_rows.get("landing").cloned().unwrap_or_default();
    let mut patterns: HashSet<String> = landing
        .iter()
        .map(|r| r.get("Pattern Name").cloned().unwrap_or_default())
        .collect();
    for row in &landing {
        patterns.extend(split_field(
            row.get("Aliases").map(|s| s.as_str()).unwrap_or(""),
            '|',
        ));
    }
    let pattern_ids: HashSet<String> = landing
        .iter()
        .map(|r| r.get("Pattern ID").cloned().unwrap_or_default())
        .collect();
    if pattern_ids.len() != landing.len() || pattern_ids.contains("") {
        problems.push("[landing] Pattern ID values must be non-empty and unique".to_string());
    }
    let mut landing_identities: HashMap<String, String> = HashMap::new();
    for row in &landing {
        let mut identities = vec![
            row.get("Pattern ID").cloned().unwrap_or_default(),
            row.get("Pattern Name").cloned().unwrap_or_default(),
        ];
        identities.extend(split_field(
            row.get("Aliases").map(|s| s.as_str()).unwrap_or(""),
            '|',
        ));
        for identity in identities {
            let folded = identity.trim().to_lowercase();
            let row_id = row.get("Pattern ID").cloned().unwrap_or_default();
            if folded.is_empty() {
                problems.push("[landing] Pattern Name values must be non-empty".to_string());
            } else if let Some(owner) = landing_identities.get(&folded) {
                if *owner != row_id {
                    problems.push(format!(
                        "[landing] ambiguous identity '{}' -> {}, {}",
                        identity, owner, row_id
                    ));
                }
            } else {
                landing_identities.insert(folded, row_id);
            }
        }
    }

    let mut replacement_targets: HashMap<String, HashSet<String>> = HashMap::new();
    replacement_targets.insert("landing".to_string(), pattern_ids.clone());
    replacement_targets.insert(
        "style".to_string(),
        styles
            .iter()
            .map(|r| r.get("Style ID").cloned().unwrap_or_default())
            .collect(),
    );
    let style_ids = check_style_contract(
        &styles,
        &products,
        &reasoning,
        &mut problems,
        &replacement_targets,
    );
    check_reasoning_contract(
        &products,
        &colors,
        &reasoning,
        &style_ids,
        &patterns,
        &mut problems,
    );
    check_core_data_contract(&domain_rows, &mut problems);
    check_catalog_contract(catalog, &domain_rows, &mut problems);
    let section_num = re(r"^\d+\.\s");
    for row in &landing {
        let sections: Vec<String> = row
            .get("Section Order")
            .map(|s| s.split(" > ").map(|p| p.to_string()).collect())
            .unwrap_or_default();
        if sections.len() < 2 || sections.iter().any(|p| section_num.is_match(p)) {
            problems.push(format!(
                "[landing:{}] invalid Section Order delimiter",
                row.get("Pattern Name").cloned().unwrap_or_default()
            ));
        }
    }
    check_provenance(catalog, &reasoning, &styles, &mut problems);

    problems
}

/// `validate_data.py __main__` — print problem list, exit 1, or OK, exit 0.
pub fn run(_argv: &[String]) -> i32 {
    let catalog = Catalog::new();
    let problems = validate(&catalog);
    if !problems.is_empty() {
        println!(
            "FAILED: {} data integrity issue(s) found:\n",
            problems.len()
        );
        for p in &problems {
            println!("  - {}", p);
        }
        return 1;
    }
    println!(
        "OK: validated {} domain files, {} stack files, and {}",
        CSV_CONFIG.len(),
        STACK_CONFIG.len(),
        REASONING_FILE
    );
    0
}
