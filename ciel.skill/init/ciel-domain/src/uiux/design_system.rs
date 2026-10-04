//! Port of `skills/ui-ux-pro-max/scripts/design_system.py` — aggregates
//! domain searches, applies the reasoning contract, formats ascii/markdown
//! design-system reports, and persists MASTER.md + page overrides.

use crate::common::cli::{parse, ArgSpec, Args};
use crate::common::csvio::Row;
use crate::common::py;
use crate::uiux::core::search;
use crate::uiux::data::Catalog;
use crate::uiux::reasoning::{apply_decision_rules, parse_decision_rules};
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const REASONING_FILE: &str = "ui-reasoning.csv";

/// (domain, max_results) in Python SEARCH_CONFIG order.
const SEARCH_CONFIG: &[(&str, i64)] = &[
    ("product", 1),
    ("style", 3),
    ("color", 5),
    ("landing", 2),
    ("typography", 2),
];

/// (label, colors-dict key, css var) — semantic palette table.
const SEMANTIC_COLOR_ENTRIES: &[(&str, &str, &str)] = &[
    ("Primary", "primary", "--color-primary"),
    ("On Primary", "on_primary", "--color-on-primary"),
    ("Secondary", "secondary", "--color-secondary"),
    ("On Secondary", "on_secondary", "--color-on-secondary"),
    ("Accent/CTA", "accent", "--color-accent"),
    ("On Accent/CTA", "on_accent", "--color-on-accent"),
    ("Background", "background", "--color-background"),
    ("Foreground", "foreground", "--color-foreground"),
    ("Card", "card", "--color-card"),
    (
        "Card Foreground",
        "card_foreground",
        "--color-card-foreground",
    ),
    ("Muted", "muted", "--color-muted"),
    (
        "Muted Foreground",
        "muted_foreground",
        "--color-muted-foreground",
    ),
    ("Border", "border", "--color-border"),
    ("Destructive", "destructive", "--color-destructive"),
    ("On Destructive", "on_destructive", "--color-on-destructive"),
    ("Ring", "ring", "--color-ring"),
];

const DARK_PRIMARY_MARKERS: &[&str] = &[
    "dark mode primary",
    "dark primary",
    "dark-only",
    "dark only",
    "dark preferred",
    "dark focused",
    "dark-first",
    "dark rich",
    "light mode only as exception",
];

const DARK_QUERY_MARKERS: &[&str] = &[
    "dark mode",
    "dark theme",
    "dark ui",
    "dark-mode",
    "darkmode",
    "night mode",
    "midnight",
    "oled",
];

const DARK_ANTI_PATTERN_MARKERS: &[&str] = &["dark mode", "dark modes", "dark theme"];

const DARK_BACKGROUND_MAX_LUMINANCE: f64 = 0.18;

// ============ DESIGN DIALS ============

#[derive(Clone)]
pub struct DialInfo {
    pub value: i64,
    pub label: &'static str,
    pub style_keywords: Vec<&'static str>,
    pub tier: Option<&'static str>,
    pub spacing: Option<Vec<(&'static str, &'static str)>>,
}

fn spacious() -> Vec<(&'static str, &'static str)> {
    vec![
        ("xs", "4px"),
        ("sm", "8px"),
        ("md", "24px"),
        ("lg", "32px"),
        ("xl", "48px"),
        ("2xl", "64px"),
        ("3xl", "96px"),
    ]
}
fn standard_spacing() -> Vec<(&'static str, &'static str)> {
    vec![
        ("xs", "4px"),
        ("sm", "8px"),
        ("md", "16px"),
        ("lg", "24px"),
        ("xl", "32px"),
        ("2xl", "48px"),
        ("3xl", "64px"),
    ]
}
fn dense() -> Vec<(&'static str, &'static str)> {
    vec![
        ("xs", "2px"),
        ("sm", "4px"),
        ("md", "8px"),
        ("lg", "12px"),
        ("xl", "16px"),
        ("2xl", "24px"),
        ("3xl", "32px"),
    ]
}

/// `_resolve_dial` — bucket a 1-10 dial into its tier config.
fn resolve_dial(dial_name: &str, value: Option<i64>) -> Option<DialInfo> {
    let value = value?.clamp(1, 10);
    match dial_name {
        "variance" => {
            let (label, kws) = match value {
                1..=3 => (
                    "Centered / Minimal",
                    vec![
                        "Minimalism",
                        "Exaggerated Minimalism",
                        "centered",
                        "symmetric",
                        "grid-based",
                    ],
                ),
                4..=7 => (
                    "Balanced / Modern",
                    vec!["modern", "structured", "balanced"],
                ),
                _ => (
                    "Bold / Asymmetric",
                    vec!["Brutalism", "Bento Grids", "asymmetric", "experimental"],
                ),
            };
            Some(DialInfo {
                value,
                label,
                style_keywords: kws,
                tier: None,
                spacing: None,
            })
        }
        "motion" => {
            let (label, tier) = match value {
                1..=3 => ("Subtle", "Subtle"),
                4..=7 => ("Standard", "Standard"),
                _ => ("Complex", "Complex"),
            };
            Some(DialInfo {
                value,
                label,
                style_keywords: vec![],
                tier: Some(tier),
                spacing: None,
            })
        }
        "density" => {
            let (label, spacing) = match value {
                1..=3 => ("Spacious", spacious()),
                4..=7 => ("Standard", standard_spacing()),
                _ => ("Dense / Dashboard", dense()),
            };
            Some(DialInfo {
                value,
                label,
                style_keywords: vec![],
                tier: None,
                spacing: Some(spacing),
            })
        }
        _ => None,
    }
}

// ============ COLOR MODE RESOLUTION ============

/// WCAG relative luminance of a #RRGGBB string, or None if unparseable.
fn relative_luminance(hex_color: &str) -> Option<f64> {
    if hex_color.is_empty() {
        return None;
    }
    let mut value = hex_color.trim().trim_start_matches('#').to_string();
    if value.len() == 3 {
        value = value.chars().flat_map(|c| [c, c]).collect();
    }
    if value.len() != 6 {
        return None;
    }
    let mut channels = [0f64; 3];
    for (i, ch) in channels.iter_mut().enumerate() {
        *ch = f64::from(u8::from_str_radix(&value[i * 2..i * 2 + 2], 16).ok()?) / 255.0;
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
    Some(0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2])
}

fn get<'a>(map: &'a Map<String, Value>, key: &str) -> &'a str {
    map.get(key).and_then(Value::as_str).unwrap_or("")
}

/// Python `dict.get(key, default)` — the stored value (even "") when the key
/// is present, otherwise `default`.
fn get_or(map: &Map<String, Value>, key: &str, default: &str) -> String {
    map.get(key)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_string()
}

fn row_get<'a>(row: &'a Row, key: &str) -> &'a str {
    row.get(key).map(|s| s.as_str()).unwrap_or("")
}

fn palette_is_dark(palette: &Map<String, Value>) -> bool {
    match relative_luminance(get(palette, "Background")) {
        Some(l) => l < DARK_BACKGROUND_MAX_LUMINANCE,
        None => false,
    }
}

fn contrast_ratio(first: &str, second: &str) -> Option<f64> {
    let a = relative_luminance(first)?;
    let b = relative_luminance(second)?;
    let (lighter, darker) = (a.max(b), a.min(b));
    Some((lighter + 0.05) / (darker + 0.05))
}

fn style_is_dark_primary(style: &Map<String, Value>) -> bool {
    if style.is_empty() {
        return false;
    }
    let preferred = get(style, "Preferred Mode").trim().to_lowercase();
    if preferred == "dark" || preferred == "light" {
        return preferred == "dark";
    }
    if get(style, "Light Mode \u{2713}") == "not-recommended"
        && get(style, "Dark Mode \u{2713}") == "supported"
    {
        return true;
    }
    let declared = format!(
        "{} {}",
        get(style, "Light Mode \u{2713}"),
        get(style, "Dark Mode \u{2713}")
    )
    .to_lowercase();
    DARK_PRIMARY_MARKERS.iter().any(|m| declared.contains(m))
}

fn query_wants_dark(query: &str) -> bool {
    let lowered = query.to_lowercase();
    DARK_QUERY_MARKERS.iter().any(|m| lowered.contains(m))
}

fn resolve_color_mode(query: &str, style: &Map<String, Value>) -> &'static str {
    if query_wants_dark(query) || style_is_dark_primary(style) {
        "dark"
    } else {
        "light"
    }
}

fn derive_dark_palette(palette: &Map<String, Value>) -> Map<String, Value> {
    let mut derived = palette.clone();
    let background = "#0F172A";
    let ring_candidates = [
        get(palette, "Ring"),
        get(palette, "Accent"),
        get(palette, "Primary"),
        "#60A5FA",
    ];
    let ring = ring_candidates
        .iter()
        .find(|c| contrast_ratio(c, background).unwrap_or(0.0) >= 3.0)
        .copied()
        .unwrap_or("#60A5FA");
    for (k, v) in [
        ("Background", background),
        ("Foreground", "#F8FAFC"),
        ("Card", "#111827"),
        ("Card Foreground", "#F8FAFC"),
        ("Muted", "#1E293B"),
        ("Muted Foreground", "#CBD5E1"),
        ("Border", "#334155"),
        ("Ring", ring),
        ("_mode_derivation", "derived-dark"),
    ] {
        derived.insert(k.to_string(), json!(v));
    }
    derived
}

fn select_palette_for_mode(
    palettes: &[Map<String, Value>],
    mode: &str,
    category: &str,
) -> Map<String, Value> {
    if palettes.is_empty() {
        return Map::new();
    }
    let category_palette = palettes.iter().find(|p| get(p, "Product Type") == category);
    if let Some(palette) = category_palette {
        if mode == "dark" && !palette_is_dark(palette) {
            return derive_dark_palette(palette);
        }
        return palette.clone();
    }
    if mode == "dark" {
        for palette in palettes {
            if palette_is_dark(palette) {
                return palette.clone();
            }
        }
    }
    palettes[0].clone()
}

fn filter_anti_patterns_for_mode(anti_patterns: &str, mode: &str) -> String {
    if mode != "dark" || anti_patterns.is_empty() {
        return anti_patterns.to_string();
    }
    let kept: Vec<String> = anti_patterns
        .split('+')
        .map(str::trim)
        .filter(|clause| {
            !clause.is_empty()
                && !DARK_ANTI_PATTERN_MARKERS
                    .iter()
                    .any(|m| clause.to_lowercase().contains(m))
        })
        .map(|s| s.to_string())
        .collect();
    kept.join(" + ")
}

fn mode_support_labels(style: &Map<String, Value>, anti_patterns: &str) -> (String, String) {
    let light = get(style, "light_mode").to_string();
    let mut dark = get(style, "dark_mode").to_string();
    let avoids_dark = DARK_ANTI_PATTERN_MARKERS
        .iter()
        .any(|m| anti_patterns.to_lowercase().contains(m));
    if !dark.is_empty() && dark != "not-recommended" && avoids_dark {
        dark = format!("{}, but avoid for this product", dark);
    }
    (light, dark)
}

fn button_outline_text_color(colors: &Map<String, Value>) -> String {
    let primary = get(colors, "primary");
    let primary = if primary.is_empty() {
        "#2563EB"
    } else {
        primary
    };
    let background = get(colors, "background");
    let background = if background.is_empty() {
        "#FFFFFF"
    } else {
        background
    };
    if contrast_ratio(primary, background).unwrap_or(0.0) >= 4.5 {
        return primary.to_string();
    }
    let fg = get(colors, "foreground");
    if fg.is_empty() {
        primary.to_string()
    } else {
        fg.to_string()
    }
}

// ============ GENERATOR ============

pub struct DesignSystemGenerator<'a> {
    catalog: &'a Catalog,
    reasoning_data: Vec<Row>,
    style_lookup: HashMap<String, Row>,
    landing_lookup: HashMap<String, Row>,
}

impl<'a> DesignSystemGenerator<'a> {
    pub fn new(catalog: &'a Catalog) -> Self {
        let reasoning_data = catalog.rows_or_empty(REASONING_FILE);
        let style_data = catalog.rows_or_empty("styles.csv");
        let mut style_lookup = HashMap::new();
        for style in &style_data {
            let mut keys = vec![
                row_get(style, "Style ID").to_string(),
                row_get(style, "Style Category").to_string(),
            ];
            keys.extend(row_get(style, "Aliases").split('|').map(|s| s.to_string()));
            for key in keys {
                let key = key.trim();
                if !key.is_empty() {
                    style_lookup.insert(key.to_lowercase(), style.clone());
                }
            }
        }
        let mut landing_lookup = HashMap::new();
        for row in catalog.rows_or_empty("landing.csv") {
            let mut identities = vec![
                row_get(&row, "Pattern ID").to_string(),
                row_get(&row, "Pattern Name").to_string(),
            ];
            identities.extend(row_get(&row, "Aliases").split('|').map(|s| s.to_string()));
            for identity in identities {
                let identity = identity.trim();
                if !identity.is_empty() {
                    landing_lookup.insert(identity.to_lowercase(), row.clone());
                }
            }
        }
        DesignSystemGenerator {
            catalog,
            reasoning_data,
            style_lookup,
            landing_lookup,
        }
    }

    /// `_resolve_style` — follow deprecated parents; {} on cycle/orphan.
    fn resolve_style(&self, reference: &str) -> Option<Row> {
        let mut style = self
            .style_lookup
            .get(&reference.trim().to_lowercase())
            .cloned();
        let mut seen = std::collections::HashSet::new();
        while let Some(s) = &style {
            if s.get("Status").map(|v| v.as_str()).unwrap_or("active") != "deprecated" {
                break;
            }
            let style_id = row_get(s, "Style ID").to_string();
            let parent_id = row_get(s, "Parent Style ID").to_string();
            if parent_id.is_empty() || seen.contains(&style_id) {
                return None;
            }
            seen.insert(style_id);
            style = self.style_lookup.get(&parent_id.to_lowercase()).cloned();
        }
        style
    }

    fn find_reasoning_rule(&self, category: &str) -> Option<Row> {
        let folded = category.trim().to_lowercase();
        self.reasoning_data
            .iter()
            .find(|rule| row_get(rule, "UI_Category").trim().to_lowercase() == folded)
            .cloned()
    }

    /// `_apply_reasoning` — merge the category rule with activated
    /// decision-rule mutations.
    fn apply_reasoning(&self, category: &str, query: &str) -> Map<String, Value> {
        let rule = self.find_reasoning_rule(category);
        let rule = match rule {
            Some(r) => r,
            None => {
                let mut m = Map::new();
                m.insert("pattern".into(), json!("Hero + Features + CTA"));
                m.insert(
                    "style_priority".into(),
                    json!(["Minimalism", "Flat Design"]),
                );
                m.insert("color_mood".into(), json!("Professional"));
                m.insert("typography_mood".into(), json!("Clean"));
                m.insert("key_effects".into(), json!("Subtle hover transitions"));
                m.insert("anti_patterns".into(), json!(""));
                m.insert("decision_rules".into(), json!({}));
                m.insert("activated_rules".into(), json!([]));
                m.insert("constraints".into(), json!([]));
                m.insert("preferred_mode".into(), Value::Null);
                m.insert("is_default".into(), json!(true));
                m.insert("severity".into(), json!("MEDIUM"));
                return m;
            }
        };
        let decision_rules =
            parse_decision_rules(row_get(&rule, "Decision_Rules")).unwrap_or_default();
        let applied = apply_decision_rules(&decision_rules, query);
        let style_priority: Vec<String> = row_get(&rule, "Style_Priority")
            .split('+')
            .map(|s| s.trim().to_string())
            .collect();
        let applied_style_names: Vec<String> = applied
            .style_ids
            .iter()
            .map(|style_id| {
                self.resolve_style(style_id)
                    .map(|s| row_get(&s, "Style Category").to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| style_id.clone())
            })
            .collect();
        let mut combined = applied_style_names;
        combined.extend(style_priority);

        let mut m = Map::new();
        m.insert(
            "pattern".into(),
            json!(applied
                .pattern
                .unwrap_or_else(|| row_get(&rule, "Recommended_Pattern").to_string())),
        );
        m.insert("style_priority".into(), json!(combined));
        m.insert("color_mood".into(), json!(row_get(&rule, "Color_Mood")));
        m.insert(
            "typography_mood".into(),
            json!(row_get(&rule, "Typography_Mood")),
        );
        m.insert("key_effects".into(), json!(row_get(&rule, "Key_Effects")));
        m.insert(
            "anti_patterns".into(),
            json!(row_get(&rule, "Anti_Patterns")),
        );
        m.insert("decision_rules".into(), Value::Object(decision_rules));
        m.insert("activated_rules".into(), json!(applied.activated));
        m.insert("constraints".into(), json!(applied.constraints));
        m.insert(
            "preferred_mode".into(),
            applied.mode.map(Value::String).unwrap_or(Value::Null),
        );
        m.insert("is_default".into(), json!(false));
        let severity = {
            let s = row_get(&rule, "Severity");
            if s.is_empty() {
                "MEDIUM".to_string()
            } else {
                s.to_string()
            }
        };
        m.insert("severity".into(), Value::String(severity));
        m
    }

    /// `_multi_domain_search` — per-domain queries resolved per the Python
    /// routing table.
    fn multi_domain_search(
        &self,
        query: &str,
        category: &str,
        reasoning: &Map<String, Value>,
        style_priority: &[String],
    ) -> HashMap<String, Value> {
        let constraints = reasoning
            .get("constraints")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(|s| s.replace('-', " "))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        let resolved_query = [query, category, constraints.as_str()]
            .iter()
            .filter(|p| !p.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" ");

        let mut results = HashMap::new();
        for &(domain, max_results) in SEARCH_CONFIG.iter() {
            let result = if domain == "style" && !style_priority.is_empty() {
                let priority_query = style_priority[..style_priority.len().min(2)].join(" ");
                search(
                    self.catalog,
                    &format!("{} {}", resolved_query, priority_query),
                    Some("style"),
                    max_results,
                    false,
                )
            } else if domain == "color" {
                let mood = get(reasoning, "color_mood");
                search(
                    self.catalog,
                    &format!("{} {}", mood, resolved_query),
                    Some("color"),
                    max_results,
                    false,
                )
            } else if domain == "landing" {
                let pattern = get(reasoning, "pattern");
                let landing_query = if self.landing_lookup.contains_key(&pattern.to_lowercase()) {
                    pattern.to_string()
                } else {
                    format!("{} {}", pattern, resolved_query)
                };
                let q = if landing_query.is_empty() {
                    query.to_string()
                } else {
                    landing_query
                };
                search(self.catalog, &q, Some("landing"), max_results, false)
            } else if domain == "typography" {
                let mood = get(reasoning, "typography_mood");
                search(
                    self.catalog,
                    &format!("{} {}", mood, resolved_query),
                    Some("typography"),
                    max_results,
                    false,
                )
            } else {
                search(self.catalog, query, Some(domain), max_results, false)
            };
            results.insert(domain.to_string(), result);
        }
        results
    }

    /// `_select_best_match` — canonical reasoning wins over lexical ranking.
    fn select_best_match(
        &self,
        results: &[Map<String, Value>],
        priority_keywords: &[String],
    ) -> Map<String, Value> {
        if results.is_empty() {
            return Map::new();
        }
        if priority_keywords.is_empty() {
            return results[0].clone();
        }
        for priority in priority_keywords {
            if let Some(resolved) = self.resolve_style(priority) {
                if resolved
                    .get("Status")
                    .map(|s| s.as_str())
                    .unwrap_or("active")
                    != "deprecated"
                {
                    let mut m = Map::new();
                    for (k, v) in &resolved {
                        m.insert(k.clone(), json!(v));
                    }
                    return m;
                }
            }
        }
        static TOKEN_RE: OnceLock<Regex> = OnceLock::new();
        let token_re = TOKEN_RE.get_or_init(|| Regex::new(r"[a-z0-9]+").unwrap());
        let mut scored: Vec<(i64, Map<String, Value>)> = Vec::new();
        for result in results {
            let result_str = result
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v.as_str().unwrap_or("")))
                .collect::<Vec<_>>()
                .join(", ")
                .to_lowercase();
            let mut score = 0i64;
            for kw in priority_keywords {
                let kw_tokens: std::collections::HashSet<String> = token_re
                    .find_iter(&kw.to_lowercase())
                    .map(|m| m.as_str().to_string())
                    .collect();
                let name_tokens: std::collections::HashSet<String> = token_re
                    .find_iter(&get(result, "Style Category").to_lowercase())
                    .map(|m| m.as_str().to_string())
                    .collect();
                let keyword_tokens: std::collections::HashSet<String> = token_re
                    .find_iter(&get(result, "Keywords").to_lowercase())
                    .map(|m| m.as_str().to_string())
                    .collect();
                if !kw_tokens.is_empty() && kw_tokens.is_subset(&name_tokens) {
                    score += 10;
                } else if kw_tokens.iter().any(|t| keyword_tokens.contains(t)) {
                    score += 3;
                } else if kw_tokens.iter().any(|t| result_str.contains(t.as_str())) {
                    score += 1;
                }
            }
            scored.push((score, result.clone()));
        }
        scored.sort_by_key(|x| std::cmp::Reverse(x.0));
        if !scored.is_empty() && scored[0].0 > 0 {
            scored[0].1.clone()
        } else {
            results[0].clone()
        }
    }

    /// `generate(query, project_name, variance, motion, density)` — the full
    /// design-system dict (key order matches the Python return literal).
    pub fn generate(
        &self,
        query: &str,
        project_name: Option<&str>,
        variance: Option<i64>,
        motion: Option<i64>,
        density: Option<i64>,
    ) -> Value {
        let variance_info = resolve_dial("variance", variance);
        let motion_info = resolve_dial("motion", motion);
        let density_info = resolve_dial("density", density);

        // Step 1: product search -> category
        let product_result = search(self.catalog, query, Some("product"), 1, false);
        let product_results = product_result
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut category = "General".to_string();
        if let Some(first) = product_results.first() {
            category = first
                .get("Product Type")
                .and_then(Value::as_str)
                .unwrap_or("General")
                .to_string();
        }

        // Step 2: reasoning rules
        let reasoning = self.apply_reasoning(&category, query);
        let style_priority: Vec<String> = reasoning
            .get("style_priority")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let effective_style_priority: Vec<String> = match &variance_info {
            Some(info) => info
                .style_keywords
                .iter()
                .map(|s| s.to_string())
                .chain(style_priority.iter().cloned())
                .collect(),
            None => style_priority.clone(),
        };

        // Step 3: multi-domain search
        let mut search_results =
            self.multi_domain_search(query, &category, &reasoning, &effective_style_priority);
        search_results.insert("product".to_string(), product_result);

        // Step 4: select best matches
        let extract = |domain: &str| -> Vec<Map<String, Value>> {
            search_results
                .get(domain)
                .and_then(|v| v.get("results"))
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|v| v.as_object().cloned()).collect())
                .unwrap_or_default()
        };
        let style_results = extract("style");
        let color_results = extract("color");
        let typography_results = extract("typography");
        let landing_results = extract("landing");

        let best_style = self.select_best_match(&style_results, &effective_style_priority);
        let color_mode = reasoning
            .get("preferred_mode")
            .and_then(Value::as_str)
            .map(|s| s.to_string())
            .unwrap_or_else(|| resolve_color_mode(query, &best_style).to_string());
        let best_color = select_palette_for_mode(&color_results, &color_mode, &category);
        let best_typography = typography_results.first().cloned().unwrap_or_default();
        let best_landing = landing_results
            .iter()
            .find(|row| get(row, "Pattern Name") == get(&reasoning, "pattern"))
            .cloned()
            .unwrap_or_else(|| landing_results.first().cloned().unwrap_or_default());

        // Motion dial -> motion.csv skeleton
        let mut motion_snippet = Map::new();
        if let Some(info) = &motion_info {
            let tier = info.tier.unwrap_or("");
            let motion_result = search(
                self.catalog,
                &format!("{} {}", query, tier),
                Some("gsap"),
                5,
                false,
            );
            let matches: Vec<Map<String, Value>> = motion_result
                .get("results")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|v| v.as_object().cloned()).collect())
                .unwrap_or_default();
            if let Some(t) = matches
                .iter()
                .find(|m| get(m, "Intensity Tier") == tier)
                .or_else(|| matches.first())
            {
                motion_snippet = t.clone();
            }
        }

        let style_effects = get(&best_style, "Effects & Animation").to_string();
        let reasoning_effects = get(&reasoning, "key_effects").to_string();
        let combined_effects = if !style_effects.is_empty() {
            style_effects.clone()
        } else {
            reasoning_effects.clone()
        };

        let mut pattern = Map::new();
        pattern.insert(
            "name".into(),
            json!(get_or(
                &best_landing,
                "Pattern Name",
                &get_or(&reasoning, "pattern", "Hero + Features + CTA")
            )),
        );
        pattern.insert(
            "sections".into(),
            json!(get_or(
                &best_landing,
                "Section Order",
                "Hero > Features > CTA"
            )),
        );
        pattern.insert(
            "cta_placement".into(),
            json!(get_or(&best_landing, "Primary CTA Placement", "Above fold")),
        );
        pattern.insert(
            "color_strategy".into(),
            json!(get(&best_landing, "Color Strategy")),
        );
        pattern.insert(
            "conversion".into(),
            json!(get(&best_landing, "Conversion Optimization")),
        );

        let mut style = Map::new();
        for (key, val) in [
            (
                "id",
                get_or(&best_style, "Style ID", "minimalism-and-swiss-style"),
            ),
            ("name", get_or(&best_style, "Style Category", "Minimalism")),
            ("type", get_or(&best_style, "Type", "General")),
            ("effects", style_effects.clone()),
            ("keywords", get(&best_style, "Keywords").to_string()),
            ("best_for", get(&best_style, "Best For").to_string()),
            ("performance", get(&best_style, "Performance").to_string()),
            (
                "accessibility",
                get(&best_style, "Accessibility").to_string(),
            ),
            (
                "light_mode",
                get(&best_style, "Light Mode \u{2713}").to_string(),
            ),
            (
                "dark_mode",
                get(&best_style, "Dark Mode \u{2713}").to_string(),
            ),
        ] {
            style.insert(key.into(), json!(val));
        }

        let mut colors = Map::new();
        for (key, src, default) in [
            ("primary", "Primary", "#2563EB"),
            ("on_primary", "On Primary", ""),
            ("secondary", "Secondary", "#3B82F6"),
            ("on_secondary", "On Secondary", ""),
            ("accent", "Accent", "#F97316"),
            ("on_accent", "On Accent", ""),
            ("background", "Background", "#F8FAFC"),
            ("foreground", "Foreground", "#1E293B"),
            ("card", "Card", ""),
            ("card_foreground", "Card Foreground", ""),
            ("muted", "Muted", ""),
            ("muted_foreground", "Muted Foreground", ""),
            ("border", "Border", ""),
            ("destructive", "Destructive", ""),
            ("on_destructive", "On Destructive", ""),
            ("ring", "Ring", ""),
            ("notes", "Notes", ""),
            ("cta", "Accent", "#F97316"),
            ("text", "Foreground", "#1E293B"),
            ("on_cta", "On Accent", ""),
        ] {
            colors.insert(key.into(), json!(get_or(&best_color, src, default)));
        }

        let mut typography = Map::new();
        for (key, src, default) in [
            ("heading", "Heading Font", "Inter"),
            ("body", "Body Font", "Inter"),
            (
                "mood",
                "Mood/Style Keywords",
                get(&reasoning, "typography_mood"),
            ),
            ("best_for", "Best For", ""),
            ("google_fonts_url", "Google Fonts URL", ""),
            ("css_import", "CSS Import", ""),
        ] {
            typography.insert(key.into(), json!(get_or(&best_typography, src, default)));
        }

        let mut identities = Map::new();
        identities.insert(
            "product".into(),
            if product_results.is_empty() {
                Value::Null
            } else {
                json!(category)
            },
        );
        identities.insert(
            "reasoning".into(),
            if reasoning
                .get("is_default")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                Value::Null
            } else {
                json!(category)
            },
        );
        identities.insert("style".into(), {
            let v = get(&best_style, "Style ID");
            if v.is_empty() {
                let alt = get(&best_style, "Style Category");
                if alt.is_empty() {
                    Value::Null
                } else {
                    json!(alt)
                }
            } else {
                json!(v)
            }
        });
        identities.insert("color".into(), {
            let v = get(&best_color, "Product Type");
            if v.is_empty() {
                Value::Null
            } else {
                json!(v)
            }
        });
        identities.insert("typography".into(), {
            let v = get(&best_typography, "Font Pairing Name");
            if v.is_empty() {
                Value::Null
            } else {
                json!(v)
            }
        });
        identities.insert("landing".into(), {
            let v = get(&best_landing, "Pattern Name");
            if v.is_empty() {
                Value::Null
            } else {
                json!(v)
            }
        });

        let mut derivations = Map::new();
        derivations.insert(
            "color_mode".into(),
            best_color
                .get("_mode_derivation")
                .cloned()
                .unwrap_or(Value::Null),
        );

        let mut dials = Map::new();
        dials.insert(
            "variance".into(),
            variance_info
                .as_ref()
                .map(|i| json!(i.value))
                .unwrap_or(Value::Null),
        );
        dials.insert(
            "variance_label".into(),
            variance_info
                .as_ref()
                .map(|i| json!(i.label))
                .unwrap_or(Value::Null),
        );
        dials.insert(
            "motion".into(),
            motion_info
                .as_ref()
                .map(|i| json!(i.value))
                .unwrap_or(Value::Null),
        );
        dials.insert(
            "motion_label".into(),
            motion_info
                .as_ref()
                .map(|i| json!(i.label))
                .unwrap_or(Value::Null),
        );
        dials.insert(
            "density".into(),
            density_info
                .as_ref()
                .map(|i| json!(i.value))
                .unwrap_or(Value::Null),
        );
        dials.insert(
            "density_label".into(),
            density_info
                .as_ref()
                .map(|i| json!(i.label))
                .unwrap_or(Value::Null),
        );

        let mut out = Map::new();
        out.insert(
            "project_name".into(),
            json!(project_name
                .map(|s| s.to_string())
                .unwrap_or_else(|| query.to_uppercase())),
        );
        out.insert("category".into(), json!(category));
        out.insert("pattern".into(), Value::Object(pattern));
        out.insert("style".into(), Value::Object(style));
        out.insert("colors".into(), Value::Object(colors));
        out.insert("typography".into(), Value::Object(typography));
        out.insert("key_effects".into(), json!(combined_effects));
        out.insert(
            "anti_patterns".into(),
            json!(filter_anti_patterns_for_mode(
                get(&reasoning, "anti_patterns"),
                &color_mode
            )),
        );
        out.insert(
            "decision_rules".into(),
            reasoning
                .get("decision_rules")
                .cloned()
                .unwrap_or(json!({})),
        );
        out.insert(
            "activated_rules".into(),
            reasoning
                .get("activated_rules")
                .cloned()
                .unwrap_or(json!([])),
        );
        out.insert(
            "constraints".into(),
            reasoning.get("constraints").cloned().unwrap_or(json!([])),
        );
        out.insert(
            "reasoning_default".into(),
            reasoning.get("is_default").cloned().unwrap_or(json!(false)),
        );
        out.insert("source_identities".into(), Value::Object(identities));
        out.insert("source_derivations".into(), Value::Object(derivations));
        let severity = {
            let s = get(&reasoning, "severity");
            if s.is_empty() {
                "MEDIUM".to_string()
            } else {
                s.to_string()
            }
        };
        out.insert("severity".into(), Value::String(severity));
        out.insert("dials".into(), Value::Object(dials));
        // motion_snippet is a projected search-result row — already in
        // output_cols order.
        out.insert("motion_snippet".into(), Value::Object(motion_snippet));
        out.insert(
            "spacing_scale".into(),
            match &density_info {
                Some(info) => {
                    let mut m = Map::new();
                    if let Some(spacing) = &info.spacing {
                        for (k, v) in spacing {
                            m.insert(k.to_string(), json!(v));
                        }
                    }
                    Value::Object(m)
                }
                None => Value::Null,
            },
        );
        Value::Object(out)
    }
}

// ============ OUTPUT FORMATTERS ============

const BOX_WIDTH: usize = 90;

fn hex_to_ansi(hex_color: &str) -> String {
    if hex_color.is_empty() || !hex_color.starts_with('#') {
        return String::new();
    }
    let colorterm = std::env::var("COLORTERM").unwrap_or_default();
    if colorterm != "truecolor" && colorterm != "24bit" {
        return String::new();
    }
    let hex = hex_color.trim_start_matches('#');
    if hex.len() != 6 {
        return String::new();
    }
    let (r, g, b) = match (
        u8::from_str_radix(&hex[0..2], 16),
        u8::from_str_radix(&hex[2..4], 16),
        u8::from_str_radix(&hex[4..6], 16),
    ) {
        (Ok(r), Ok(g), Ok(b)) => (r, g, b),
        _ => return String::new(),
    };
    format!("\x1b[38;2;{};{};{}m██\x1b[0m ", r, g, b)
}

static ANSI_RE: OnceLock<Regex> = OnceLock::new();

fn visible_len(s: &str) -> usize {
    let re = ANSI_RE.get_or_init(|| Regex::new(r"\x1b\[[0-9;]*m").unwrap());
    re.replace_all(s, "").chars().count()
}

fn ansi_ljust(s: &str, width: usize) -> String {
    let pad = width.saturating_sub(visible_len(s));
    format!("{}{}", s, " ".repeat(pad))
}

fn ljust_chars(s: &str, width: usize) -> String {
    let pad = width.saturating_sub(s.chars().count());
    format!("{}{}", s, " ".repeat(pad))
}

fn section_header(name: &str, width: usize) -> String {
    let label = format!("─── {} ", name);
    let fill = "─".repeat(width.saturating_sub(label.chars().count() + 1));
    format!("├{}{}┤", label, fill)
}

fn wrap_text(text: &str, prefix: &str, width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }
    let room = width.saturating_sub(2 + prefix.chars().count());
    let mut words: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        let chars: Vec<char> = word.chars().collect();
        if chars.len() <= room {
            words.push(word.to_string());
        } else {
            let mut i = 0;
            while i < chars.len() {
                let end = (i + room).min(chars.len());
                words.push(chars[i..end].iter().collect());
                i = end;
            }
        }
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = prefix.to_string();
    for word in &words {
        if current.chars().count() + word.chars().count() < width - 2 {
            if current != prefix {
                current.push(' ');
            }
            current.push_str(word);
        } else {
            if current != prefix {
                lines.push(current.clone());
            }
            current = format!("{}{}", prefix, word);
        }
    }
    if current != prefix {
        lines.push(current);
    }
    lines
}

fn add_wrapped(lines: &mut Vec<String>, text: &str, prefix: &str) {
    for line in wrap_text(text, prefix, BOX_WIDTH) {
        lines.push(format!("{}│", ljust_chars(&line, BOX_WIDTH)));
    }
}

pub fn format_ascii_box(design_system: &Map<String, Value>) -> String {
    let project = get(design_system, "project_name");
    let project = if project.is_empty() {
        "PROJECT"
    } else {
        project
    };
    let pattern = design_system
        .get("pattern")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let style = design_system
        .get("style")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let colors = design_system
        .get("colors")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let typography = design_system
        .get("typography")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let effects = get(design_system, "key_effects").to_string();
    let anti_patterns = get(design_system, "anti_patterns").to_string();
    let dials = design_system
        .get("dials")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let motion_snippet = design_system
        .get("motion_snippet")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut sections: Vec<String> = get(&pattern, "sections")
        .split(" > ")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut lines: Vec<String> = Vec::new();
    let w = BOX_WIDTH - 1;
    lines.push(format!("╔{}╗", "═".repeat(w)));
    lines.push(format!(
        "{}║",
        ansi_ljust(
            &format!("║  TARGET: {} - RECOMMENDED DESIGN SYSTEM", project),
            BOX_WIDTH
        )
    ));
    lines.push(format!("╚{}╝", "═".repeat(w)));
    lines.push(format!("┌{}┐", "─".repeat(w)));

    let any_dial = ["variance", "motion", "density"]
        .iter()
        .any(|k| dials.get(*k).map(|v| !v.is_null()).unwrap_or(false));
    if any_dial {
        lines.push(section_header("DESIGN DIALS", BOX_WIDTH));
        for (key, label) in [
            ("variance", "Variance"),
            ("motion", "Motion  "),
            ("density", "Density "),
        ] {
            if let Some(v) = dials.get(key).filter(|v| !v.is_null()) {
                let lbl = get(&dials, &format!("{}_label", key));
                lines.push(format!(
                    "{}│",
                    ljust_chars(&format!("│  {}: {}/10 — {}", label, v, lbl), BOX_WIDTH)
                ));
            }
        }
    }

    lines.push(section_header("PATTERN", BOX_WIDTH));
    add_wrapped(
        &mut lines,
        &format!("Name: {}", get(&pattern, "name")),
        "│  ",
    );
    if !get(&pattern, "conversion").is_empty() {
        add_wrapped(
            &mut lines,
            &format!("Conversion: {}", get(&pattern, "conversion")),
            "│     ",
        );
    }
    if !get(&pattern, "cta_placement").is_empty() {
        add_wrapped(
            &mut lines,
            &format!("CTA: {}", get(&pattern, "cta_placement")),
            "│     ",
        );
    }
    lines.push(format!("{}│", ljust_chars("│     Sections:", BOX_WIDTH)));
    for (i, section) in sections.drain(..).enumerate() {
        add_wrapped(&mut lines, &format!("{}. {}", i + 1, section), "│       ");
    }

    lines.push(section_header("STYLE", BOX_WIDTH));
    add_wrapped(&mut lines, &format!("Name: {}", get(&style, "name")), "│  ");
    let (light, dark) = mode_support_labels(&style, &anti_patterns);
    if !light.is_empty() || !dark.is_empty() {
        add_wrapped(
            &mut lines,
            &format!("Mode Support: Light {} | Dark {}", light, dark),
            "│     ",
        );
    }
    if !get(&style, "keywords").is_empty() {
        add_wrapped(
            &mut lines,
            &format!("Keywords: {}", get(&style, "keywords")),
            "│     ",
        );
    }
    if !get(&style, "best_for").is_empty() {
        add_wrapped(
            &mut lines,
            &format!("Best For: {}", get(&style, "best_for")),
            "│     ",
        );
    }
    if !get(&style, "performance").is_empty() || !get(&style, "accessibility").is_empty() {
        add_wrapped(
            &mut lines,
            &format!(
                "Performance: {} | Accessibility: {}",
                get(&style, "performance"),
                get(&style, "accessibility")
            ),
            "│     ",
        );
    }

    lines.push(section_header("COLORS", BOX_WIDTH));
    for &(label, key, css_var) in SEMANTIC_COLOR_ENTRIES.iter() {
        let hex_val = get(&colors, key);
        if hex_val.is_empty() {
            continue;
        }
        let swatch = hex_to_ansi(hex_val);
        let content = format!(
            "│     {}{:18} {:10} ({})",
            swatch,
            format!("{}:", label),
            hex_val,
            css_var
        );
        lines.push(format!("{}│", ansi_ljust(&content, BOX_WIDTH)));
    }
    if !get(&colors, "notes").is_empty() {
        for line in wrap_text(
            &format!("Notes: {}", get(&colors, "notes")),
            "│     ",
            BOX_WIDTH,
        ) {
            lines.push(format!("{}│", ljust_chars(&line, BOX_WIDTH)));
        }
    }

    lines.push(section_header("TYPOGRAPHY", BOX_WIDTH));
    add_wrapped(
        &mut lines,
        &format!(
            "{} / {}",
            get(&typography, "heading"),
            get(&typography, "body")
        ),
        "│  ",
    );
    if !get(&typography, "mood").is_empty() {
        add_wrapped(
            &mut lines,
            &format!("Mood: {}", get(&typography, "mood")),
            "│     ",
        );
    }
    if !get(&typography, "best_for").is_empty() {
        add_wrapped(
            &mut lines,
            &format!("Best For: {}", get(&typography, "best_for")),
            "│     ",
        );
    }
    if !get(&typography, "google_fonts_url").is_empty() {
        add_wrapped(
            &mut lines,
            &format!("Google Fonts: {}", get(&typography, "google_fonts_url")),
            "│     ",
        );
    }
    if !get(&typography, "css_import").is_empty() {
        let truncated: String = get(&typography, "css_import").chars().take(67).collect();
        lines.push(format!(
            "{}│",
            ljust_chars(&format!("│     CSS Import: {}...", truncated), BOX_WIDTH)
        ));
    }

    if !effects.is_empty() {
        lines.push(section_header("KEY EFFECTS", BOX_WIDTH));
        for line in wrap_text(&effects, "│     ", BOX_WIDTH) {
            lines.push(format!("{}│", ljust_chars(&line, BOX_WIDTH)));
        }
    }

    if !motion_snippet.is_empty() {
        lines.push(section_header("MOTION", BOX_WIDTH));
        add_wrapped(
            &mut lines,
            &format!(
                "{} ({})",
                get(&motion_snippet, "Category"),
                get(&motion_snippet, "Intensity Tier")
            ),
            "│  ",
        );
        add_wrapped(
            &mut lines,
            &format!(
                "Trigger: {} | Duration: {} | Easing: {}",
                get(&motion_snippet, "Trigger"),
                get(&motion_snippet, "Duration"),
                get(&motion_snippet, "Easing")
            ),
            "│     ",
        );
        for line in wrap_text(
            &format!("GSAP: {}", get(&motion_snippet, "GSAP Snippet")),
            "│     ",
            BOX_WIDTH,
        ) {
            lines.push(format!("{}│", ljust_chars(&line, BOX_WIDTH)));
        }
        if !get(&motion_snippet, "Framework Notes").is_empty() {
            for line in wrap_text(
                &format!("Framework: {}", get(&motion_snippet, "Framework Notes")),
                "│     ",
                BOX_WIDTH,
            ) {
                lines.push(format!("{}│", ljust_chars(&line, BOX_WIDTH)));
            }
        }
    }

    if !anti_patterns.is_empty() {
        lines.push(section_header("AVOID", BOX_WIDTH));
        for line in wrap_text(&anti_patterns, "│     ", BOX_WIDTH) {
            lines.push(format!("{}│", ljust_chars(&line, BOX_WIDTH)));
        }
    }

    lines.push(section_header("PRE-DELIVERY CHECKLIST", BOX_WIDTH));
    for item in [
        "[ ] No emojis as icons (use SVG: Heroicons/Lucide)",
        "[ ] cursor-pointer on all clickable elements",
        "[ ] Hover states with smooth transitions (150-300ms)",
        "[ ] Light mode: text contrast 4.5:1 minimum",
        "[ ] Focus states visible for keyboard nav",
        "[ ] prefers-reduced-motion respected",
        "[ ] Responsive: 375px, 768px, 1024px, 1440px",
    ] {
        lines.push(format!(
            "{}│",
            ljust_chars(&format!("│     {}", item), BOX_WIDTH)
        ));
    }
    lines.push(format!("└{}┘", "─".repeat(w)));
    lines.join("\n")
}

pub fn format_markdown(design_system: &Map<String, Value>) -> String {
    let project = get(design_system, "project_name");
    let project = if project.is_empty() {
        "PROJECT"
    } else {
        project
    };
    let obj = |key: &str| -> Map<String, Value> {
        design_system
            .get(key)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default()
    };
    let pattern = obj("pattern");
    let style = obj("style");
    let colors = obj("colors");
    let typography = obj("typography");
    let dials = obj("dials");
    let motion_snippet = obj("motion_snippet");
    let effects = get(design_system, "key_effects");
    let anti_patterns = get(design_system, "anti_patterns");

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("## Design System: {}", project));
    lines.push(String::new());

    let any_dial = ["variance", "motion", "density"]
        .iter()
        .any(|k| dials.get(*k).map(|v| !v.is_null()).unwrap_or(false));
    if any_dial {
        lines.push("### Design Dials".into());
        for (key, label) in [
            ("variance", "Variance"),
            ("motion", "Motion"),
            ("density", "Density"),
        ] {
            if let Some(v) = dials.get(key).filter(|v| !v.is_null()) {
                lines.push(format!(
                    "- **{}:** {}/10 — {}",
                    label,
                    v,
                    get(&dials, &format!("{}_label", key))
                ));
            }
        }
        lines.push(String::new());
    }

    lines.push("### Pattern".into());
    lines.push(format!("- **Name:** {}", get(&pattern, "name")));
    if !get(&pattern, "conversion").is_empty() {
        lines.push(format!(
            "- **Conversion Focus:** {}",
            get(&pattern, "conversion")
        ));
    }
    if !get(&pattern, "cta_placement").is_empty() {
        lines.push(format!(
            "- **CTA Placement:** {}",
            get(&pattern, "cta_placement")
        ));
    }
    if !get(&pattern, "color_strategy").is_empty() {
        lines.push(format!(
            "- **Color Strategy:** {}",
            get(&pattern, "color_strategy")
        ));
    }
    lines.push(format!("- **Sections:** {}", get(&pattern, "sections")));
    lines.push(String::new());

    lines.push("### Style".into());
    lines.push(format!("- **Name:** {}", get(&style, "name")));
    let (light, dark) = mode_support_labels(&style, anti_patterns);
    if !light.is_empty() || !dark.is_empty() {
        lines.push(format!(
            "- **Mode Support:** Light {} | Dark {}",
            light, dark
        ));
    }
    if !get(&style, "keywords").is_empty() {
        lines.push(format!("- **Keywords:** {}", get(&style, "keywords")));
    }
    if !get(&style, "best_for").is_empty() {
        lines.push(format!("- **Best For:** {}", get(&style, "best_for")));
    }
    if !get(&style, "performance").is_empty() || !get(&style, "accessibility").is_empty() {
        lines.push(format!(
            "- **Performance:** {} | **Accessibility:** {}",
            get(&style, "performance"),
            get(&style, "accessibility")
        ));
    }
    lines.push(String::new());

    lines.push("### Colors".into());
    lines.push("| Role | Hex | CSS Variable |".into());
    lines.push("|------|-----|--------------|".into());
    for &(label, key, css_var) in SEMANTIC_COLOR_ENTRIES.iter() {
        let hex_val = get(&colors, key);
        if !hex_val.is_empty() {
            lines.push(format!("| {} | `{}` | `{}` |", label, hex_val, css_var));
        }
    }
    if !get(&colors, "notes").is_empty() {
        lines.push(format!("\n*Notes: {}*", get(&colors, "notes")));
    }
    lines.push(String::new());

    lines.push("### Typography".into());
    lines.push(format!("- **Heading:** {}", get(&typography, "heading")));
    lines.push(format!("- **Body:** {}", get(&typography, "body")));
    if !get(&typography, "mood").is_empty() {
        lines.push(format!("- **Mood:** {}", get(&typography, "mood")));
    }
    if !get(&typography, "best_for").is_empty() {
        lines.push(format!("- **Best For:** {}", get(&typography, "best_for")));
    }
    if !get(&typography, "google_fonts_url").is_empty() {
        lines.push(format!(
            "- **Google Fonts:** {}",
            get(&typography, "google_fonts_url")
        ));
    }
    if !get(&typography, "css_import").is_empty() {
        lines.push("- **CSS Import:**".into());
        lines.push("```css".into());
        lines.push(get(&typography, "css_import").to_string());
        lines.push("```".into());
    }
    lines.push(String::new());

    if !effects.is_empty() {
        lines.push("### Key Effects".into());
        lines.push(effects.to_string());
        lines.push(String::new());
    }

    if !motion_snippet.is_empty() {
        lines.push("### Motion".into());
        lines.push(format!(
            "**{}** ({}) — Trigger: {} | Duration: {} | Easing: `{}`",
            get(&motion_snippet, "Category"),
            get(&motion_snippet, "Intensity Tier"),
            get(&motion_snippet, "Trigger"),
            get(&motion_snippet, "Duration"),
            get(&motion_snippet, "Easing")
        ));
        lines.push("```js".into());
        lines.push(get(&motion_snippet, "GSAP Snippet").to_string());
        lines.push("```".into());
        if !get(&motion_snippet, "Framework Notes").is_empty() {
            lines.push(format!(
                "*Framework notes: {}*",
                get(&motion_snippet, "Framework Notes")
            ));
        }
        let motion_do = get(&motion_snippet, "Do");
        let motion_dont = get(&motion_snippet, "Don't");
        if !motion_do.is_empty() {
            lines.push(format!("- ✅ {}", motion_do));
        }
        if !motion_dont.is_empty() {
            lines.push(format!("- ❌ {}", motion_dont));
        }
        lines.push(String::new());
    }

    if !anti_patterns.is_empty() {
        lines.push("### Avoid (Anti-patterns)".into());
        lines.push(format!("- {}", anti_patterns.replace(" + ", "\n- ")));
        lines.push(String::new());
    }

    lines.push("### Pre-Delivery Checklist".into());
    for item in [
        "- [ ] No emojis as icons (use SVG: Heroicons/Lucide)",
        "- [ ] cursor-pointer on all clickable elements",
        "- [ ] Hover states with smooth transitions (150-300ms)",
        "- [ ] Light mode: text contrast 4.5:1 minimum",
        "- [ ] Focus states visible for keyboard nav",
        "- [ ] prefers-reduced-motion respected",
        "- [ ] Responsive: 375px, 768px, 1024px, 1440px",
    ] {
        lines.push(item.to_string());
    }
    lines.push(String::new());
    lines.join("\n")
}

// ============ PERSISTENCE ============

/// `safe_slug` — only [a-z0-9_-] survives; everything else collapses to '-'.
pub fn safe_slug(name: &str, fallback: &str) -> String {
    static SLUG_RE: OnceLock<Regex> = OnceLock::new();
    let re = SLUG_RE.get_or_init(|| Regex::new(r"[^a-z0-9_-]+").unwrap());
    let slug = re
        .replace_all(&name.to_lowercase(), "-")
        .trim_matches('-')
        .to_string();
    if slug.is_empty() {
        fallback.to_string()
    } else {
        slug
    }
}

/// `_write_persisted_file` — fully write a temp file in the same directory,
/// then publish via rename (force) or an atomic no-overwrite hard link.
fn write_persisted_file(path: &Path, content: &str, force: bool) -> Result<(), std::io::Error> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "out".to_string());
    let temp_path = parent.join(format!(".{}.{}.tmp", name, std::process::id()));
    let result = (|| {
        {
            let mut handle = fs::File::create(&temp_path)?;
            handle.write_all(content.as_bytes())?;
            handle.sync_all()?;
        }
        if force {
            fs::rename(&temp_path, path)
        } else {
            // hard link publishes only when the destination is absent
            fs::hard_link(&temp_path, path)
        }
    })();
    let _ = fs::remove_file(&temp_path);
    result
}

/// `persist_design_system` — Master + Overrides folder pattern.
/// `Err` mirrors an unexpected Python exception (I/O failure other than
/// FileExistsError); callers exit non-zero.
pub fn persist_design_system(
    design_system: &Map<String, Value>,
    page: Option<&str>,
    output_dir: Option<&str>,
    page_query: &str,
    force: bool,
    catalog: &Catalog,
) -> Result<Value, String> {
    let base_dir = output_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let project_name = {
        let p = get(design_system, "project_name");
        if p.is_empty() {
            "default"
        } else {
            p
        }
    };
    let project_slug = safe_slug(project_name, "default");
    let ds_dir = base_dir.join("design-system").join(&project_slug);
    let pages_dir = ds_dir.join("pages");
    let master_file = ds_dir.join("MASTER.md");

    let mut created_files: Vec<String> = Vec::new();
    fs::create_dir_all(&pages_dir).map_err(|e| e.to_string())?;

    let master_content = format_master_md(design_system);
    match write_persisted_file(&master_file, &master_content, force) {
        Ok(()) => created_files.push(master_file.to_string_lossy().into_owned()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if page.is_none() {
                return Ok(json!({
                    "status": "skipped_exists",
                    "design_system_dir": ds_dir.to_string_lossy(),
                    "master_file": master_file.to_string_lossy(),
                    "created_files": [],
                    "message": format!(
                        "{} already exists and was not modified. Read it first to check for prior design decisions, then re-run with force=True / --force to overwrite.",
                        master_file.to_string_lossy()
                    ),
                }));
            }
            // A page-only run still produces the page override below.
        }
        Err(e) => return Err(e.to_string()),
    }

    if let Some(page) = page {
        let page_file = pages_dir.join(format!("{}.md", safe_slug(page, "page")));
        let page_content = format_page_override_md(design_system, page, page_query, catalog);
        match write_persisted_file(&page_file, &page_content, force) {
            Ok(()) => created_files.push(page_file.to_string_lossy().into_owned()),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if created_files.is_empty() {
                    return Ok(json!({
                        "status": "skipped_exists",
                        "design_system_dir": ds_dir.to_string_lossy(),
                        "master_file": master_file.to_string_lossy(),
                        "created_files": [],
                        "message": format!("{} already exists and was not modified.", page_file.to_string_lossy()),
                    }));
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(json!({
        "status": "success",
        "design_system_dir": ds_dir.to_string_lossy(),
        "master_file": master_file.to_string_lossy(),
        "created_files": created_files,
    }))
}

fn spacing_scale_value(design_system: &Map<String, Value>) -> Vec<(String, String)> {
    if let Some(scale) = design_system
        .get("spacing_scale")
        .and_then(Value::as_object)
    {
        return scale
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect();
    }
    standard_spacing()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn format_g(x: f64) -> String {
    if x.fract() == 0.0 {
        format!("{}", x as i64)
    } else {
        let s = format!("{}", x);
        s
    }
}

pub fn format_master_md(design_system: &Map<String, Value>) -> String {
    let project = get(design_system, "project_name");
    let project = if project.is_empty() {
        "PROJECT"
    } else {
        project
    };
    let obj = |key: &str| -> Map<String, Value> {
        design_system
            .get(key)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default()
    };
    let pattern = obj("pattern");
    let style = obj("style");
    let colors = obj("colors");
    let typography = obj("typography");
    let dials = obj("dials");
    let motion_snippet = obj("motion_snippet");
    let effects = get(design_system, "key_effects");
    let anti_patterns = get(design_system, "anti_patterns");
    let timestamp = py::now_local_hms();

    let mut lines: Vec<String> = Vec::new();
    lines.push("# Design System Master File".into());
    lines.push(String::new());
    lines.push("> **LOGIC:** When building a specific page, first check `design-system/pages/[page-name].md`.".into());
    lines.push("> If that file exists, its rules **override** this Master file.".into());
    lines.push("> If not, strictly follow the rules below.".into());
    lines.push(String::new());
    lines.push("---".into());
    lines.push(String::new());
    lines.push(format!("**Project:** {}", project));
    lines.push(format!("**Generated:** {}", timestamp));
    lines.push(format!("**Category:** {}", {
        let c = get(design_system, "category");
        if c.is_empty() {
            "General"
        } else {
            c
        }
    }));
    let any_dial = ["variance", "motion", "density"]
        .iter()
        .any(|k| dials.get(*k).map(|v| !v.is_null()).unwrap_or(false));
    if any_dial {
        let mut parts = Vec::new();
        for (key, label) in [
            ("variance", "Variance"),
            ("motion", "Motion"),
            ("density", "Density"),
        ] {
            if let Some(v) = dials.get(key).filter(|v| !v.is_null()) {
                parts.push(format!(
                    "{} {}/10 ({})",
                    label,
                    v,
                    get(&dials, &format!("{}_label", key))
                ));
            }
        }
        lines.push(format!("**Design Dials:** {}", parts.join(" | ")));
    }
    lines.push(String::new());
    lines.push("---".into());
    lines.push(String::new());

    lines.push("## Global Rules".into());
    lines.push(String::new());
    lines.push("### Color Palette".into());
    lines.push(String::new());
    lines.push("| Role | Hex | CSS Variable |".into());
    lines.push("|------|-----|--------------|".into());
    for &(label, key, css_var) in SEMANTIC_COLOR_ENTRIES.iter() {
        let hex_val = get(&colors, key);
        if !hex_val.is_empty() {
            lines.push(format!("| {} | `{}` | `{}` |", label, hex_val, css_var));
        }
    }
    lines.push(String::new());
    if !get(&colors, "notes").is_empty() {
        lines.push(format!("**Color Notes:** {}", get(&colors, "notes")));
        lines.push(String::new());
    }

    lines.push("### Typography".into());
    lines.push(String::new());
    lines.push(format!("- **Heading Font:** {}", {
        let v = get(&typography, "heading");
        if v.is_empty() {
            "Inter"
        } else {
            v
        }
    }));
    lines.push(format!("- **Body Font:** {}", {
        let v = get(&typography, "body");
        if v.is_empty() {
            "Inter"
        } else {
            v
        }
    }));
    if !get(&typography, "mood").is_empty() {
        lines.push(format!("- **Mood:** {}", get(&typography, "mood")));
    }
    if !get(&typography, "google_fonts_url").is_empty() {
        lines.push(format!(
            "- **Google Fonts:** [{} + {}]({})",
            get(&typography, "heading"),
            get(&typography, "body"),
            get(&typography, "google_fonts_url")
        ));
    }
    lines.push(String::new());
    if !get(&typography, "css_import").is_empty() {
        lines.push("**CSS Import:**".into());
        lines.push("```css".into());
        lines.push(get(&typography, "css_import").to_string());
        lines.push("```".into());
        lines.push(String::new());
    }

    let scale = spacing_scale_value(design_system);
    let spacing_usage: &[(&str, &str)] = &[
        ("xs", "Tight gaps"),
        ("sm", "Icon gaps, inline spacing"),
        ("md", "Standard padding"),
        ("lg", "Section padding"),
        ("xl", "Large gaps"),
        ("2xl", "Section margins"),
        ("3xl", "Hero padding"),
    ];
    lines.push("### Spacing Variables".into());
    lines.push(String::new());
    let has_spacing_override = design_system
        .get("spacing_scale")
        .map(|v| !v.is_null())
        .unwrap_or(false);
    if has_spacing_override {
        let density = dials.get("density").cloned().unwrap_or(Value::Null);
        lines.push(format!(
            "*Density: {}/10 — {}*",
            density,
            get(&dials, "density_label")
        ));
        lines.push(String::new());
    }
    lines.push("| Token | Value | Usage |".into());
    lines.push("|-------|-------|-------|".into());
    for (token, usage) in spacing_usage {
        let px_value = scale
            .iter()
            .find(|(k, _)| k == token)
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let rem_value = px_value
            .trim_end_matches("px")
            .parse::<f64>()
            .map(|n| format_g(n / 16.0))
            .unwrap_or_default();
        lines.push(format!(
            "| `--space-{}` | `{}` / `{}rem` | {} |",
            token, px_value, rem_value, usage
        ));
    }
    lines.push(String::new());

    lines.push("### Shadow Depths".into());
    lines.push(String::new());
    lines.push("| Level | Value | Usage |".into());
    lines.push("|-------|-------|-------|".into());
    lines.push("| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.05)` | Subtle lift |".into());
    lines.push("| `--shadow-md` | `0 4px 6px rgba(0,0,0,0.1)` | Cards, buttons |".into());
    lines.push("| `--shadow-lg` | `0 10px 15px rgba(0,0,0,0.1)` | Modals, dropdowns |".into());
    lines.push(
        "| `--shadow-xl` | `0 20px 25px rgba(0,0,0,0.15)` | Hero images, featured cards |".into(),
    );
    lines.push(String::new());

    lines.push("---".into());
    lines.push(String::new());
    lines.push("## Component Specs".into());
    lines.push(String::new());
    lines.push("### Buttons".into());
    lines.push(String::new());
    lines.push("```css".into());
    lines.push("/* Primary Button */".into());
    lines.push(".btn-primary {".into());
    lines.push(format!("  background: {};", {
        let v = get(&colors, "cta");
        if v.is_empty() {
            "#F97316"
        } else {
            v
        }
    }));
    lines.push(format!("  color: {};", {
        let v = get(&colors, "on_cta");
        if v.is_empty() {
            "white"
        } else {
            v
        }
    }));
    lines.push("  padding: 12px 24px;".into());
    lines.push("  border-radius: 8px;".into());
    lines.push("  font-weight: 600;".into());
    lines.push("  transition: all 200ms ease;".into());
    lines.push("  cursor: pointer;".into());
    lines.push("}".into());
    lines.push(String::new());
    lines.push(".btn-primary:hover {".into());
    lines.push("  opacity: 0.9;".into());
    lines.push("  transform: translateY(-1px);".into());
    lines.push("}".into());
    lines.push(String::new());
    lines.push("/* Secondary Button */".into());
    lines.push(".btn-secondary {".into());
    lines.push("  background: transparent;".into());
    lines.push(format!("  color: {};", button_outline_text_color(&colors)));
    lines.push(format!("  border: 2px solid {};", {
        let v = get(&colors, "primary");
        if v.is_empty() {
            "#2563EB"
        } else {
            v
        }
    }));
    lines.push("  padding: 12px 24px;".into());
    lines.push("  border-radius: 8px;".into());
    lines.push("  font-weight: 600;".into());
    lines.push("  transition: all 200ms ease;".into());
    lines.push("  cursor: pointer;".into());
    lines.push("}".into());
    lines.push("```".into());
    lines.push(String::new());

    lines.push("### Cards".into());
    lines.push(String::new());
    lines.push("```css".into());
    lines.push(".card {".into());
    lines.push(format!("  background: {};", {
        let v = get(&colors, "background");
        if v.is_empty() {
            "#FFFFFF"
        } else {
            v
        }
    }));
    lines.push("  border-radius: 12px;".into());
    lines.push("  padding: 24px;".into());
    lines.push("  box-shadow: var(--shadow-md);".into());
    lines.push("  transition: all 200ms ease;".into());
    lines.push("  cursor: pointer;".into());
    lines.push("}".into());
    lines.push(String::new());
    lines.push(".card:hover {".into());
    lines.push("  box-shadow: var(--shadow-lg);".into());
    lines.push("  transform: translateY(-2px);".into());
    lines.push("}".into());
    lines.push("```".into());
    lines.push(String::new());

    lines.push("### Inputs".into());
    lines.push(String::new());
    lines.push("```css".into());
    lines.push(".input {".into());
    lines.push("  padding: 12px 16px;".into());
    lines.push("  border: 1px solid #E2E8F0;".into());
    lines.push("  border-radius: 8px;".into());
    lines.push("  font-size: 16px;".into());
    lines.push("  transition: border-color 200ms ease;".into());
    lines.push("}".into());
    lines.push(String::new());
    lines.push(".input:focus {".into());
    lines.push(format!("  border-color: {};", {
        let v = get(&colors, "primary");
        if v.is_empty() {
            "#2563EB"
        } else {
            v
        }
    }));
    lines.push("  outline: none;".into());
    lines.push(format!("  box-shadow: 0 0 0 3px {}20;", {
        let v = get(&colors, "primary");
        if v.is_empty() {
            "#2563EB"
        } else {
            v
        }
    }));
    lines.push("}".into());
    lines.push("```".into());
    lines.push(String::new());

    lines.push("### Modals".into());
    lines.push(String::new());
    lines.push("```css".into());
    lines.push(".modal-overlay {".into());
    lines.push("  background: rgba(0, 0, 0, 0.5);".into());
    lines.push("  backdrop-filter: blur(4px);".into());
    lines.push("}".into());
    lines.push(String::new());
    lines.push(".modal {".into());
    lines.push("  background: white;".into());
    lines.push("  border-radius: 16px;".into());
    lines.push("  padding: 32px;".into());
    lines.push("  box-shadow: var(--shadow-xl);".into());
    lines.push("  max-width: 500px;".into());
    lines.push("  width: 90%;".into());
    lines.push("}".into());
    lines.push("```".into());
    lines.push(String::new());

    lines.push("---".into());
    lines.push(String::new());
    lines.push("## Style Guidelines".into());
    lines.push(String::new());
    lines.push(format!("**Style:** {}", {
        let v = get(&style, "name");
        if v.is_empty() {
            "Minimalism"
        } else {
            v
        }
    }));
    lines.push(String::new());
    if !get(&style, "keywords").is_empty() {
        lines.push(format!("**Keywords:** {}", get(&style, "keywords")));
        lines.push(String::new());
    }
    if !get(&style, "best_for").is_empty() {
        lines.push(format!("**Best For:** {}", get(&style, "best_for")));
        lines.push(String::new());
    }
    if !effects.is_empty() {
        lines.push(format!("**Key Effects:** {}", effects));
        lines.push(String::new());
    }

    lines.push("### Page Pattern".into());
    lines.push(String::new());
    lines.push(format!("**Pattern Name:** {}", get(&pattern, "name")));
    lines.push(String::new());
    if !get(&pattern, "conversion").is_empty() {
        lines.push(format!(
            "- **Conversion Strategy:** {}",
            get(&pattern, "conversion")
        ));
    }
    if !get(&pattern, "cta_placement").is_empty() {
        lines.push(format!(
            "- **CTA Placement:** {}",
            get(&pattern, "cta_placement")
        ));
    }
    lines.push(format!(
        "- **Section Order:** {}",
        get(&pattern, "sections")
    ));
    lines.push(String::new());

    if !motion_snippet.is_empty() {
        lines.push("---".into());
        lines.push(String::new());
        lines.push("## Motion".into());
        lines.push(String::new());
        lines.push(format!(
            "**{}** ({}) — Trigger: {} | Duration: {} | Easing: `{}`",
            get(&motion_snippet, "Category"),
            get(&motion_snippet, "Intensity Tier"),
            get(&motion_snippet, "Trigger"),
            get(&motion_snippet, "Duration"),
            get(&motion_snippet, "Easing")
        ));
        lines.push(String::new());
        lines.push("```js".into());
        lines.push(get(&motion_snippet, "GSAP Snippet").to_string());
        lines.push("```".into());
        lines.push(String::new());
        if !get(&motion_snippet, "Framework Notes").is_empty() {
            lines.push(format!(
                "**Framework notes:** {}",
                get(&motion_snippet, "Framework Notes")
            ));
            lines.push(String::new());
        }
        let motion_do = get(&motion_snippet, "Do");
        let motion_dont = get(&motion_snippet, "Don't");
        if !motion_do.is_empty() {
            lines.push(format!("- ✅ {}", motion_do));
        }
        if !motion_dont.is_empty() {
            lines.push(format!("- ❌ {}", motion_dont));
        }
        if !get(&motion_snippet, "Performance Notes").is_empty() {
            lines.push(format!(
                "- ⚡ {}",
                get(&motion_snippet, "Performance Notes")
            ));
        }
        lines.push(String::new());
    }

    lines.push("---".into());
    lines.push(String::new());
    lines.push("## Anti-Patterns (Do NOT Use)".into());
    lines.push(String::new());
    if !anti_patterns.is_empty() {
        for anti in anti_patterns
            .split('+')
            .map(|a| a.trim())
            .filter(|a| !a.is_empty())
        {
            lines.push(format!("- ❌ {}", anti));
        }
    }
    lines.push(String::new());
    lines.push("### Additional Forbidden Patterns".into());
    lines.push(String::new());
    lines.push("- ❌ **Emojis as icons** — Use SVG icons (Heroicons, Lucide, Simple Icons)".into());
    lines.push(
        "- ❌ **Missing cursor:pointer** — All clickable elements must have cursor:pointer".into(),
    );
    lines.push("- ❌ **Layout-shifting hovers** — Avoid scale transforms that shift layout".into());
    lines.push("- ❌ **Low contrast text** — Maintain 4.5:1 minimum contrast ratio".into());
    lines.push("- ❌ **Instant state changes** — Always use transitions (150-300ms)".into());
    lines.push("- ❌ **Invisible focus states** — Focus states must be visible for a11y".into());
    lines.push(String::new());

    lines.push("---".into());
    lines.push(String::new());
    lines.push("## Pre-Delivery Checklist".into());
    lines.push(String::new());
    lines.push("Before delivering any UI code, verify:".into());
    lines.push(String::new());
    for item in [
        "- [ ] No emojis used as icons (use SVG instead)",
        "- [ ] All icons from consistent icon set (Heroicons/Lucide)",
        "- [ ] `cursor-pointer` on all clickable elements",
        "- [ ] Hover states with smooth transitions (150-300ms)",
        "- [ ] Light mode: text contrast 4.5:1 minimum",
        "- [ ] Focus states visible for keyboard navigation",
        "- [ ] `prefers-reduced-motion` respected",
        "- [ ] Responsive: 375px, 768px, 1024px, 1440px",
        "- [ ] No content hidden behind fixed navbars",
        "- [ ] No horizontal scroll on mobile",
    ] {
        lines.push(item.to_string());
    }
    lines.push(String::new());
    lines.join("\n")
}

/// Python `str.title()` — uppercase each word's first char, lowercase rest.
fn title_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut new_word = true;
    for c in s.chars() {
        if c.is_alphanumeric() {
            if new_word {
                out.extend(c.to_uppercase());
            } else {
                out.extend(c.to_lowercase());
            }
            new_word = false;
        } else {
            out.push(c);
            new_word = true;
        }
    }
    out
}

fn format_page_override_md(
    design_system: &Map<String, Value>,
    page_name: &str,
    page_query: &str,
    catalog: &Catalog,
) -> String {
    let project = {
        let p = get(design_system, "project_name");
        if p.is_empty() {
            "PROJECT"
        } else {
            p
        }
    };
    let timestamp = py::now_local_hms();
    let page_title = title_case(&page_name.replace(['-', '_'], " "));
    let page_overrides =
        generate_intelligent_overrides(page_name, page_query, design_system, catalog);

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("# {} Page Overrides", page_title));
    lines.push(String::new());
    lines.push(format!("> **PROJECT:** {}", project));
    lines.push(format!("> **Generated:** {}", timestamp));
    lines.push(format!("> **Page Type:** {}", {
        let v = get(&page_overrides, "page_type");
        if v.is_empty() {
            "General"
        } else {
            v
        }
    }));
    lines.push(String::new());
    lines.push("> ⚠️ **IMPORTANT:** Rules in this file **override** the Master file (`design-system/MASTER.md`).".into());
    lines.push("> Only deviations from the Master are documented here. For all other rules, refer to the Master.".into());
    lines.push(String::new());
    lines.push("---".into());
    lines.push(String::new());
    lines.push("## Page-Specific Rules".into());
    lines.push(String::new());

    let dict_section = |lines: &mut Vec<String>, title: &str, key: &str, empty: &str| {
        lines.push(format!("### {}", title));
        lines.push(String::new());
        if let Some(map) = page_overrides.get(key).and_then(Value::as_object) {
            if !map.is_empty() {
                for (k, v) in map {
                    lines.push(format!("- **{}:** {}", k, v.as_str().unwrap_or("")));
                }
            } else {
                lines.push(format!("- {}", empty));
            }
        } else {
            lines.push(format!("- {}", empty));
        }
        lines.push(String::new());
    };
    dict_section(
        &mut lines,
        "Layout Overrides",
        "layout",
        "No overrides — use Master layout",
    );
    dict_section(
        &mut lines,
        "Spacing Overrides",
        "spacing",
        "No overrides — use Master spacing",
    );
    dict_section(
        &mut lines,
        "Typography Overrides",
        "typography",
        "No overrides — use Master typography",
    );
    dict_section(
        &mut lines,
        "Color Overrides",
        "colors",
        "No overrides — use Master colors",
    );

    lines.push("### Component Overrides".into());
    lines.push(String::new());
    let components = page_overrides
        .get("components")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if components.is_empty() {
        lines.push("- No overrides — use Master component specs".into());
    } else {
        for c in &components {
            lines.push(format!("- {}", c.as_str().unwrap_or("")));
        }
    }
    lines.push(String::new());

    lines.push("---".into());
    lines.push(String::new());
    lines.push("## Page-Specific Components".into());
    lines.push(String::new());
    let unique = page_overrides
        .get("unique_components")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if unique.is_empty() {
        lines.push("- No unique components for this page".into());
    } else {
        for c in &unique {
            lines.push(format!("- {}", c.as_str().unwrap_or("")));
        }
    }
    lines.push(String::new());

    lines.push("---".into());
    lines.push(String::new());
    lines.push("## Recommendations".into());
    lines.push(String::new());
    let recs = page_overrides
        .get("recommendations")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !recs.is_empty() {
        for r in &recs {
            lines.push(format!("- {}", r.as_str().unwrap_or("")));
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

fn generate_intelligent_overrides(
    page_name: &str,
    page_query: &str,
    design_system: &Map<String, Value>,
    catalog: &Catalog,
) -> Map<String, Value> {
    let _ = design_system;
    let page_lower = page_name.to_lowercase();
    let query_lower = page_query.to_lowercase();
    let combined_context = format!("{} {}", page_lower, query_lower);

    let style_search = search(catalog, &combined_context, Some("style"), 1, false);
    let ux_search = search(catalog, &combined_context, Some("ux"), 3, false);
    let landing_search = search(catalog, &combined_context, Some("landing"), 1, false);

    let results_of = |v: &Value| -> Vec<Map<String, Value>> {
        v.get("results")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|x| x.as_object().cloned()).collect())
            .unwrap_or_default()
    };
    let style_results = results_of(&style_search);
    let ux_results = results_of(&ux_search);
    let landing_results = results_of(&landing_search);

    let page_type = detect_page_type(&combined_context, &style_results);

    let mut layout = Map::new();
    let mut spacing = Map::new();
    let typography = Map::new();
    let mut colors = Map::new();
    let mut components: Vec<String> = Vec::new();
    let unique_components: Vec<String> = Vec::new();
    let mut recommendations: Vec<String> = Vec::new();

    if let Some(style) = style_results.first() {
        let keywords = get(style, "Keywords").to_lowercase();
        if ["data", "dense", "dashboard", "grid"]
            .iter()
            .any(|kw| keywords.contains(kw))
        {
            layout.insert("Max Width".into(), json!("1400px or full-width"));
            layout.insert("Grid".into(), json!("12-column grid for data flexibility"));
            spacing.insert(
                "Content Density".into(),
                json!("High — optimize for information display"),
            );
        } else if ["minimal", "simple", "clean", "single"]
            .iter()
            .any(|kw| keywords.contains(kw))
        {
            layout.insert("Max Width".into(), json!("800px (narrow, focused)"));
            layout.insert("Layout".into(), json!("Single column, centered"));
            spacing.insert("Content Density".into(), json!("Low — focus on clarity"));
        } else {
            layout.insert("Max Width".into(), json!("1200px (standard)"));
            layout.insert(
                "Layout".into(),
                json!("Full-width sections, centered content"),
            );
        }
        let effects = get(style, "Effects & Animation");
        if !effects.is_empty() {
            recommendations.push(format!("Effects: {}", effects));
        }
    }

    for ux in &ux_results {
        let category = get(ux, "Category");
        let do_text = get(ux, "Do");
        let dont_text = get(ux, "Don't");
        if !do_text.is_empty() {
            recommendations.push(format!("{}: {}", category, do_text));
        }
        if !dont_text.is_empty() {
            components.push(format!("Avoid: {}", dont_text));
        }
    }

    if let Some(landing) = landing_results.first() {
        let sections = get(landing, "Section Order");
        let cta = get(landing, "Primary CTA Placement");
        let strategy = get(landing, "Color Strategy");
        if !sections.is_empty() {
            layout.insert("Sections".into(), json!(sections));
        }
        if !cta.is_empty() {
            recommendations.push(format!("CTA Placement: {}", cta));
        }
        if !strategy.is_empty() {
            colors.insert("Strategy".into(), json!(strategy));
        }
    }

    if layout.is_empty() {
        layout.insert("Max Width".into(), json!("1200px"));
        layout.insert("Layout".into(), json!("Responsive grid"));
    }
    if recommendations.is_empty() {
        recommendations = vec![
            "Refer to MASTER.md for all design rules".to_string(),
            "Add specific overrides as needed for this page".to_string(),
        ];
    }

    let mut out = Map::new();
    out.insert("page_type".into(), json!(page_type));
    out.insert("layout".into(), Value::Object(layout));
    out.insert("spacing".into(), Value::Object(spacing));
    out.insert("typography".into(), Value::Object(typography));
    out.insert("colors".into(), Value::Object(colors));
    out.insert("components".into(), json!(components));
    out.insert("unique_components".into(), json!(unique_components));
    out.insert("recommendations".into(), json!(recommendations));
    out
}

fn detect_page_type(context: &str, style_results: &[Map<String, Value>]) -> &'static str {
    let context_lower = context.to_lowercase();
    let page_patterns: &[(&[&str], &str)] = &[
        (
            &[
                "dashboard",
                "admin",
                "analytics",
                "data",
                "metrics",
                "stats",
                "monitor",
                "overview",
            ],
            "Dashboard / Data View",
        ),
        (
            &[
                "checkout", "payment", "cart", "purchase", "order", "billing",
            ],
            "Checkout / Payment",
        ),
        (
            &["settings", "profile", "account", "preferences", "config"],
            "Settings / Profile",
        ),
        (
            &["landing", "marketing", "homepage", "hero", "home", "promo"],
            "Landing / Marketing",
        ),
        (
            &["login", "signin", "signup", "register", "auth", "password"],
            "Authentication",
        ),
        (
            &["pricing", "plans", "subscription", "tiers", "packages"],
            "Pricing / Plans",
        ),
        (
            &["blog", "article", "post", "news", "content", "story"],
            "Blog / Article",
        ),
        (
            &["product", "item", "detail", "pdp", "shop", "store"],
            "Product Detail",
        ),
        (
            &["search", "results", "browse", "filter", "catalog", "list"],
            "Search Results",
        ),
        (
            &["empty", "404", "error", "not found", "zero"],
            "Empty State",
        ),
    ];
    for (keywords, page_type) in page_patterns {
        if keywords.iter().any(|kw| context_lower.contains(kw)) {
            return page_type;
        }
    }
    if let Some(style) = style_results.first() {
        let best_for = get(style, "Best For").to_lowercase();
        if best_for.contains("dashboard") || best_for.contains("data") {
            return "Dashboard / Data View";
        }
        if best_for.contains("landing") || best_for.contains("marketing") {
            return "Landing / Marketing";
        }
    }
    "General"
}

// ============ ENTRY POINT ============

pub struct Generated {
    pub text: String,
    pub design_system: Value,
    pub persistence: Option<Value>,
}

/// `generate_design_system` — returns text + dict + persistence info.
#[allow(clippy::too_many_arguments)]
pub fn generate_design_system(
    catalog: &Catalog,
    query: &str,
    project_name: Option<&str>,
    output_format: &str,
    persist: bool,
    page: Option<&str>,
    output_dir: Option<&str>,
    variance: Option<i64>,
    motion: Option<i64>,
    density: Option<i64>,
    force: bool,
) -> Generated {
    let generator = DesignSystemGenerator::new(catalog);
    let design_system = generator.generate(query, project_name, variance, motion, density);
    let ds_map = design_system.as_object().cloned().unwrap_or_default();

    let persistence = if persist {
        match persist_design_system(&ds_map, page, output_dir, query, force, catalog) {
            Ok(v) => Some(v),
            Err(e) => {
                // Python lets the I/O exception propagate → traceback, exit 1.
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    let text = if output_format == "markdown" {
        format_markdown(&ds_map)
    } else {
        format_ascii_box(&ds_map)
    };

    Generated {
        text,
        design_system,
        persistence,
    }
}

/// Standalone `design_system.py` CLI: `query [-p name] [-f ascii|markdown]`.
pub fn run(argv: &[String]) -> i32 {
    let args: Args = parse(
        "design_system.py",
        argv,
        &[
            ArgSpec::positional("query").req(),
            ArgSpec::value("project_name", Some('p'), "project-name"),
            ArgSpec::value("format", Some('f'), "format")
                .choices(&["ascii", "markdown"])
                .def("ascii"),
        ],
    );
    let query = args.get("query").unwrap_or("");
    let catalog = Catalog::new();
    let result = generate_design_system(
        &catalog,
        query,
        args.get("project_name"),
        args.get_or("format", "ascii").as_str(),
        false,
        None,
        None,
        None,
        None,
        None,
        false,
    );
    println!("{}", result.text);
    0
}
