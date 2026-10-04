//! Port of `skills/ui-ux-pro-max/scripts/core.py` — BM25 search over the
//! ui-ux-pro-max CSV catalog with calibrated abstention thresholds,
//! query rewrites, exact-identity routing, and stack version filtering.

use crate::common::csvio::Row;
use crate::common::difflib;
use crate::uiux::data::Catalog;
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

pub const MAX_RESULTS: i64 = 3;
pub const INDEX_VERSION: i64 = 2;
pub const SEARCH_CALIBRATION_VERSION: &str = "2026-08-12-v1";

type RowFilter<'a> = Box<dyn Fn(&Row) -> bool + 'a>;

fn accept_all(row: &Row) -> bool {
    let _ = row;
    true
}

/// (domain, file, search_cols, output_cols) in Python dict order.
pub const CSV_CONFIG: &[(&str, &str, &[&str], &[&str])] = &[
    (
        "style",
        "styles.csv",
        &[
            "Style ID",
            "Style Category",
            "Aliases",
            "Keywords",
            "Best For",
            "Type",
            "AI Prompt Keywords",
        ],
        &[
            "Style ID",
            "Style Category",
            "Aliases",
            "Status",
            "Parent Style ID",
            "Preferred Mode",
            "Type",
            "Keywords",
            "Primary Colors",
            "Effects & Animation",
            "Best For",
            "Light Mode \u{2713}",
            "Dark Mode \u{2713}",
            "Performance",
            "Accessibility",
            "Framework Compatibility",
            "Complexity",
            "AI Prompt Keywords",
            "CSS/Technical Keywords",
            "Implementation Checklist",
            "Design System Variables",
        ],
    ),
    (
        "color",
        "colors.csv",
        &["Product Type", "Notes"],
        &[
            "Product Type",
            "Primary",
            "On Primary",
            "Secondary",
            "On Secondary",
            "Accent",
            "On Accent",
            "Background",
            "Foreground",
            "Card",
            "Card Foreground",
            "Muted",
            "Muted Foreground",
            "Border",
            "Destructive",
            "On Destructive",
            "Ring",
            "Notes",
        ],
    ),
    (
        "chart",
        "charts.csv",
        &[
            "Data Type",
            "Keywords",
            "Best Chart Type",
            "When to Use",
            "When NOT to Use",
            "Accessibility Notes",
        ],
        &[
            "Data Type",
            "Keywords",
            "Best Chart Type",
            "Secondary Options",
            "When to Use",
            "When NOT to Use",
            "Data Volume Threshold",
            "Color Guidance",
            "Accessibility Grade",
            "Accessibility Risk",
            "Accessibility Notes",
            "A11y Fallback",
            "Library Recommendation",
            "Interactive Level",
        ],
    ),
    (
        "landing",
        "landing.csv",
        &[
            "Pattern ID",
            "Pattern Name",
            "Aliases",
            "Keywords",
            "Conversion Optimization",
            "Section Order",
        ],
        &[
            "Pattern ID",
            "Pattern Name",
            "Aliases",
            "Keywords",
            "Section Order",
            "Primary CTA Placement",
            "Color Strategy",
            "Conversion Optimization",
        ],
    ),
    (
        "product",
        "products.csv",
        &[
            "Product Type",
            "Keywords",
            "Primary Style Recommendation",
            "Key Considerations",
        ],
        &[
            "Product Type",
            "Keywords",
            "Primary Style Recommendation",
            "Secondary Styles",
            "Landing Page Pattern",
            "Dashboard Style (if applicable)",
            "Color Palette Focus",
        ],
    ),
    (
        "ux",
        "ux-guidelines.csv",
        &["Category", "Issue", "Description", "Platform"],
        &[
            "Category",
            "Issue",
            "Platform",
            "Description",
            "Do",
            "Don't",
            "Code Example Good",
            "Code Example Bad",
            "Severity",
        ],
    ),
    (
        "typography",
        "typography.csv",
        &[
            "Font Pairing Name",
            "Category",
            "Mood/Style Keywords",
            "Best For",
            "Heading Font",
            "Body Font",
        ],
        &[
            "Font Pairing Name",
            "Category",
            "Heading Font",
            "Body Font",
            "Mood/Style Keywords",
            "Best For",
            "Google Fonts URL",
            "CSS Import",
            "Tailwind Config",
            "Notes",
        ],
    ),
    (
        "icons",
        "icons.csv",
        &["Category", "Icon Name", "Keywords", "Best For", "Library"],
        &[
            "Category",
            "Icon Name",
            "Keywords",
            "Library",
            "Import Code",
            "Usage",
            "Best For",
            "Style",
            "Semantic Role",
            "Allowed Contexts",
        ],
    ),
    (
        "gsap",
        "motion.csv",
        &["Category", "Intensity Tier", "Keywords", "Trigger"],
        &[
            "Category",
            "Intensity Tier",
            "Trigger",
            "Duration",
            "Easing",
            "GSAP Snippet",
            "Framework Notes",
            "Do",
            "Don't",
            "Performance Notes",
        ],
    ),
    (
        "react",
        "react-performance.csv",
        &["Category", "Issue", "Keywords", "Description"],
        &[
            "Category",
            "Issue",
            "Platform",
            "Description",
            "Do",
            "Don't",
            "Code Example Good",
            "Code Example Bad",
            "Severity",
        ],
    ),
    (
        "web",
        "app-interface.csv",
        &["Category", "Issue", "Keywords", "Description"],
        &[
            "Category",
            "Issue",
            "Platform",
            "Description",
            "Do",
            "Don't",
            "Code Example Good",
            "Code Example Bad",
            "Severity",
        ],
    ),
    (
        "google-fonts",
        "google-fonts.csv",
        &[
            "Family",
            "Category",
            "Stroke",
            "Classifications",
            "Keywords",
            "Subsets",
            "Designers",
        ],
        &[
            "Family",
            "Category",
            "Stroke",
            "Classifications",
            "Styles",
            "Variable Axes",
            "Subsets",
            "Designers",
            "Popularity Rank",
            "Google Fonts URL",
        ],
    ),
];

/// Columns whose content must never be hard-truncated for display.
pub const UNTRUNCATED_COLS: &[&str] = &[
    "Code Example Good",
    "Code Example Bad",
    "Code Good",
    "Code Bad",
    "Implementation Checklist",
    "Design System Variables",
    "CSS Import",
    "Tailwind Config",
    "GSAP Snippet",
];

/// (stack, file) in Python dict order.
pub const STACK_CONFIG: &[(&str, &str)] = &[
    ("react", "stacks/react.csv"),
    ("nextjs", "stacks/nextjs.csv"),
    ("vue", "stacks/vue.csv"),
    ("svelte", "stacks/svelte.csv"),
    ("astro", "stacks/astro.csv"),
    ("swiftui", "stacks/swiftui.csv"),
    ("react-native", "stacks/react-native.csv"),
    ("flutter", "stacks/flutter.csv"),
    ("nuxtjs", "stacks/nuxtjs.csv"),
    ("nuxt-ui", "stacks/nuxt-ui.csv"),
    ("html-tailwind", "stacks/html-tailwind.csv"),
    ("shadcn", "stacks/shadcn.csv"),
    ("jetpack-compose", "stacks/jetpack-compose.csv"),
    ("threejs", "stacks/threejs.csv"),
    ("angular", "stacks/angular.csv"),
    ("laravel", "stacks/laravel.csv"),
    ("javafx", "stacks/javafx.csv"),
    ("wpf", "stacks/wpf.csv"),
    ("winui", "stacks/winui.csv"),
    ("avalonia", "stacks/avalonia.csv"),
    ("uno", "stacks/uno.csv"),
    ("uwp", "stacks/uwp.csv"),
];

pub const STACK_SEARCH_COLS: &[&str] = &[
    "Category",
    "Guideline",
    "Description",
    "Do",
    "Don't",
    "Code Good",
    "Code Bad",
];
pub const STACK_OUTPUT_COLS: &[&str] = &[
    "Category",
    "Guideline",
    "Description",
    "Do",
    "Don't",
    "Code Good",
    "Code Bad",
    "Severity",
    "Docs URL",
    "Applies To",
    "Status",
    "Verified At",
];

const WEB_STACK_CURRENT_MAJORS: &[(&str, i64)] = &[
    ("react", 19),
    ("nextjs", 16),
    ("vue", 3),
    ("svelte", 5),
    ("astro", 7),
    ("angular", 22),
    ("html-tailwind", 4),
    ("nuxtjs", 4),
    ("nuxt-ui", 4),
];

fn stack_current_versions() -> &'static HashMap<&'static str, Vec<i64>> {
    static MAP: OnceLock<HashMap<&'static str, Vec<i64>>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m: HashMap<&'static str, Vec<i64>> = HashMap::new();
        for (stack, major) in WEB_STACK_CURRENT_MAJORS {
            m.insert(*stack, vec![*major]);
        }
        m.insert("react-native", vec![0, 86]);
        m.insert("flutter", vec![3, 44]);
        m.insert("swiftui", vec![16]);
        m.insert("jetpack-compose", vec![1, 11]);
        m.insert("avalonia", vec![12]);
        m.insert("winui", vec![3]);
        m.insert("javafx", vec![26]);
        m.insert("threejs", vec![0, 185]);
        m.insert("laravel", vec![13]);
        m
    })
}

fn legacy_only_stacks() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| {
        let mut s = HashSet::new();
        s.insert("uwp");
        s
    })
}

pub const STACK_CURRENT_APPLICABILITY: &[(&str, &str)] = &[
    ("react", "react 19.2.x"),
    ("nextjs", "nextjs 16.2"),
    ("vue", "vue 3.5.x"),
    ("svelte", "svelte 5"),
    ("astro", "astro 7.1.6"),
    ("angular", "angular 22.x"),
    ("html-tailwind", "html-tailwind 4.3"),
    ("shadcn", "shadcn cli 4"),
    ("nuxtjs", "nuxtjs 4.5"),
    ("nuxt-ui", "nuxt-ui 4.11.1"),
    ("react-native", "react-native 0.86.x"),
    ("flutter", "flutter 3.44.x"),
    ("swiftui", "swiftui current"),
    ("jetpack-compose", "jetpack-compose 1.11.4"),
    ("avalonia", "avalonia 12"),
    ("uwp", "uwp legacy"),
    ("winui", "winui current"),
    ("wpf", "wpf current"),
    ("uno", "uno current"),
    ("javafx", "javafx 26"),
    ("threejs", "threejs 0.185.1"),
    ("laravel", "laravel 13.x"),
];

/// `STACK_CURRENT_APPLICABILITY[stack]` — used by validate.rs.
pub fn stack_applicability(stack: &str) -> &'static str {
    STACK_CURRENT_APPLICABILITY
        .iter()
        .find(|(k, _)| *k == stack)
        .map(|(_, v)| *v)
        .unwrap_or("")
}

const STACK_QUERY_NAMES: &[(&str, &str)] = &[
    ("react", r"react"),
    ("nextjs", r"next(?:\.js|js)?"),
    ("vue", r"vue"),
    ("svelte", r"svelte"),
    ("astro", r"astro"),
    ("angular", r"angular"),
    ("html-tailwind", r"tailwind(?:\s*css)?"),
    ("nuxtjs", r"nuxt(?:\.js|js)?"),
    ("nuxt-ui", r"nuxt\s*ui"),
    ("react-native", r"react[\s-]*native"),
    ("flutter", r"flutter"),
    ("swiftui", r"(?:ios|swiftui\s+ios)"),
    ("jetpack-compose", r"(?:jetpack\s*)?compose"),
    ("avalonia", r"avalonia"),
    ("winui", r"winui"),
    ("javafx", r"javafx"),
    ("threejs", r"three(?:\.js|js)?"),
    ("laravel", r"laravel"),
];

pub fn available_stacks() -> Vec<&'static str> {
    STACK_CONFIG.iter().map(|(k, _)| *k).collect()
}

#[derive(Clone, Copy)]
pub struct Threshold {
    pub min_score: f64,
    pub min_margin: f64,
    pub min_coverage: f64,
}

const NO_THRESHOLD: Threshold = Threshold {
    min_score: 0.0,
    min_margin: 0.0,
    min_coverage: 0.0,
};

const STACK_THRESHOLD: Threshold = Threshold {
    min_score: 3.6,
    min_margin: 0.0,
    min_coverage: 1.0 / 3.0,
};

fn search_threshold(domain: &str) -> Threshold {
    let min_score = match domain {
        "style" => 4.3,
        "landing" => 4.0,
        "product" => 6.0,
        "icons" => 5.8,
        "react" => 3.3,
        _ => 0.0,
    };
    Threshold {
        min_score,
        min_margin: 0.0,
        min_coverage: if domain == "landing" { 0.5 } else { 0.0 },
    }
}

const STYLE_IDENTITY_FIELDS: &[&str] = &["Style ID", "Style Category", "Aliases"];
const LANDING_IDENTITY_FIELDS: &[&str] = &["Pattern ID", "Pattern Name", "Aliases"];

fn domain_query_rewrites(domain: &str) -> &'static [(&'static str, Option<&'static str>)] {
    match domain {
        "color" => &[
            ("color", None),
            ("palette", None),
            ("hex", None),
            ("rgb", None),
            ("token", None),
            ("semantic", None),
            ("destructive", None),
            ("muted", None),
            ("foreground", None),
        ],
        "landing" => &[("testimonial", Some("testimonials"))],
        "style" => &[
            ("css", None),
            ("implementation", None),
            ("variable", None),
            ("checklist", None),
            ("tailwind", None),
        ],
        "ux" => &[
            ("ux", Some("accessibility")),
            ("usability", Some("accessibility")),
            ("wcag", Some("accessibility")),
        ],
        "google-fonts" => &[("typography", Some("font"))],
        "icons" => &[
            ("lucide", None),
            ("symbol", None),
            ("glyph", None),
            ("pictogram", None),
        ],
        "gsap" => &[
            ("gsap", Some("animation")),
            ("quickto", None),
            ("scrolltrigger", Some("scroll")),
            ("flip plugin", None),
            ("splittext", None),
        ],
        "react" => &[
            ("nextjs", Some("react")),
            ("usecallback", Some("memoization")),
            ("useeffect", Some("effects")),
        ],
        "web" => &[
            ("aria", Some("accessibility")),
            ("outline", Some("focus")),
            ("semantic", None),
            ("autocomplete", Some("input")),
            ("preconnect", None),
        ],
        _ => &[],
    }
}

// ============ TOKENIZATION ============

const STOPWORDS: &[&str] = &[
    "to", "in", "on", "at", "is", "of", "by", "or", "an", "if", "no", "so", "do", "be", "we", "it",
    "as", "the", "and", "for", "are", "was",
];

fn stopwords() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| STOPWORDS.iter().copied().collect())
}

/// (variant, canonical) — applied longest-first at token boundaries.
const SYNONYMS: &[(&str, &str)] = &[
    ("q&a", "question answer"),
    ("e-commerce", "ecommerce"),
    ("dark-mode", "dark"),
    ("darkmode", "dark"),
    ("light-mode", "light"),
    ("lightmode", "light"),
    ("a11y", "accessibility"),
    ("nav", "navigation"),
    ("sign-up", "signup"),
    ("log-in", "login"),
    ("colour", "color"),
    ("colours", "colors"),
    ("customisation", "customization"),
    ("organisation", "organization"),
    ("behaviour", "behavior"),
    ("ux/ui", "ux ui"),
];

fn synonym_patterns() -> &'static Vec<(Regex, &'static str)> {
    static PATTERNS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let mut items: Vec<(&str, &str)> = SYNONYMS.to_vec();
        items.sort_by_key(|(variant, _)| std::cmp::Reverse(variant.len()));
        items
            .into_iter()
            .map(|(variant, canonical)| {
                (
                    Regex::new(&format!("(?i){}", regex::escape(variant))).unwrap(),
                    canonical,
                )
            })
            .collect()
    })
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `_normalize` — longest-first synonym substitution at token boundaries.
/// Python uses `(?<!\w)variant(?!\w)` with IGNORECASE; the regex crate lacks
/// look-around, so matches are filtered by a manual ASCII word-boundary
/// check (equivalent for the ASCII variants in the table).
pub fn normalize(text: &str) -> String {
    let mut normalized = text.to_string();
    for (pattern, canonical) in synonym_patterns() {
        let bytes = normalized.as_bytes();
        let mut rebuilt = String::with_capacity(normalized.len());
        let mut cursor = 0usize;
        let mut touched = false;
        for m in pattern.find_iter(&normalized) {
            let prev_ok = m.start() == 0 || !is_word_byte(bytes[m.start() - 1]);
            let next_ok = m.end() >= bytes.len() || !is_word_byte(bytes[m.end()]);
            if !(prev_ok && next_ok) {
                continue;
            }
            rebuilt.push_str(&normalized[cursor..m.start()]);
            rebuilt.push_str(canonical);
            cursor = m.end();
            touched = true;
        }
        if touched {
            rebuilt.push_str(&normalized[cursor..]);
            normalized = rebuilt;
        }
    }
    normalized
}

// ============ BM25 ============

pub struct Bm25 {
    k1: f64,
    b: f64,
    pub n: usize,
    doc_lengths: Vec<usize>,
    avgdl: f64,
    pub idf: HashMap<String, f64>,
    pub doc_freqs: HashMap<String, i64>,
    term_freqs: Vec<HashMap<String, i64>>,
}

impl Default for Bm25 {
    fn default() -> Self {
        Self::new()
    }
}

impl Bm25 {
    pub fn new() -> Bm25 {
        Bm25 {
            k1: 1.5,
            b: 0.75,
            n: 0,
            doc_lengths: Vec::new(),
            avgdl: 0.0,
            idf: HashMap::new(),
            doc_freqs: HashMap::new(),
            term_freqs: Vec::new(),
        }
    }

    /// Lowercase, normalize synonyms, split, strip punctuation, drop
    /// stopwords and single-char tokens.
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let text = normalize(&text.to_lowercase());
        static NON_WORD: OnceLock<Regex> = OnceLock::new();
        let re = NON_WORD.get_or_init(|| Regex::new(r"[^\w\s]").unwrap());
        let cleaned = re.replace_all(&text, " ");
        cleaned
            .split_whitespace()
            .filter(|w| w.chars().count() >= 2 && !stopwords().contains(*w))
            .map(|w| w.to_string())
            .collect()
    }

    pub fn fit(&mut self, documents: &[String]) {
        let corpus: Vec<Vec<String>> = documents.iter().map(|d| self.tokenize(d)).collect();
        self.n = corpus.len();
        if self.n == 0 {
            return;
        }
        self.doc_lengths = corpus.iter().map(|d| d.len()).collect();
        self.avgdl = (self.doc_lengths.iter().sum::<usize>() as f64 / self.n as f64).max(0.0);
        if self.avgdl == 0.0 {
            self.avgdl = 1.0;
        }
        self.term_freqs = Vec::with_capacity(self.n);
        for doc in &corpus {
            let mut tf: HashMap<String, i64> = HashMap::new();
            for word in doc {
                *tf.entry(word.clone()).or_insert(0) += 1;
            }
            for word in tf.keys() {
                *self.doc_freqs.entry(word.clone()).or_insert(0) += 1;
            }
            self.term_freqs.push(tf);
        }
        for (word, freq) in &self.doc_freqs {
            let idf = ((self.n as f64 - *freq as f64 + 0.5) / (*freq as f64 + 0.5) + 1.0).ln();
            self.idf.insert(word.clone(), idf);
        }
    }

    /// (index, score) sorted by score descending.
    pub fn score(&self, query: &str) -> Vec<(usize, f64)> {
        let query_tokens = self.tokenize(query);
        let mut scores = Vec::with_capacity(self.n);
        for idx in 0..self.n {
            let mut score = 0.0f64;
            let doc_len = self.doc_lengths[idx] as f64;
            for token in &query_tokens {
                if let Some(idf) = self.idf.get(token) {
                    let tf = *self.term_freqs[idx].get(token).unwrap_or(&0) as f64;
                    let numerator = tf * (self.k1 + 1.0);
                    let denominator = tf + self.k1 * (1.0 - self.b + self.b * doc_len / self.avgdl);
                    score += idf * numerator / denominator;
                }
            }
            scores.push((idx, score));
        }
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
    }

    pub fn vocabulary(&self) -> Vec<&String> {
        self.idf.keys().collect()
    }
}

// ============ SEARCH ============

fn query_coverage(index: &Bm25, query: &str) -> f64 {
    let tokens: HashSet<String> = index.tokenize(query).into_iter().collect();
    if tokens.is_empty() {
        return 0.0;
    }
    let hits = tokens.iter().filter(|t| index.idf.contains_key(*t)).count();
    hits as f64 / tokens.len() as f64
}

fn project_row(row: &Row, columns: &[&str]) -> Map<String, Value> {
    let mut map = Map::new();
    for col in columns {
        if let Some(v) = row.get(*col) {
            map.insert(col.to_string(), Value::String(v.clone()));
        }
    }
    map
}

fn exact_match_diagnostic(query: &str, reason: &str) -> Map<String, Value> {
    let mut d = Map::new();
    d.insert("normalized_query".into(), json!(normalize(query)));
    d.insert("search_query".into(), json!(query));
    d.insert("query_rewrites".into(), json!(Vec::<String>::new()));
    d.insert("top_score".into(), json!(0.0));
    d.insert("runner_up_score".into(), json!(0.0));
    d.insert("margin".into(), json!(0.0));
    d.insert("token_coverage".into(), json!(1.0));
    d.insert("abstained".into(), json!(false));
    d.insert(
        "calibration_version".into(),
        json!(SEARCH_CALIBRATION_VERSION),
    );
    d.insert("reason".into(), json!(reason));
    d
}

pub struct DetailedSearch {
    pub results: Vec<Map<String, Value>>,
    pub index: Option<Bm25>,
    pub diagnostic: Map<String, Value>,
}

#[allow(clippy::too_many_arguments)]
fn search_csv_detailed(
    catalog: &Catalog,
    filepath: &str,
    search_cols: &[&str],
    output_cols: &[&str],
    query: &str,
    max_results: usize,
    threshold: Threshold,
    routing_domain: Option<&str>,
    row_filter: Option<&RowFilter<'_>>,
) -> DetailedSearch {
    let path = catalog.path(filepath);
    if !path.exists() {
        let mut d = Map::new();
        d.insert("reason".into(), json!("missing-file"));
        return DetailedSearch {
            results: vec![],
            index: None,
            diagnostic: d,
        };
    }
    let data = match catalog.rows(filepath) {
        Ok(rows) => rows,
        Err(_) => {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| filepath.to_string());
            let mut d = Map::new();
            d.insert("reason".into(), json!("read-error"));
            d.insert(
                "error".into(),
                json!(format!("Unable to read search data: {}", name)),
            );
            return DetailedSearch {
                results: vec![],
                index: None,
                diagnostic: d,
            };
        }
    };
    if data.is_empty() {
        let mut d = Map::new();
        d.insert("reason".into(), json!("empty-data"));
        return DetailedSearch {
            results: vec![],
            index: None,
            diagnostic: d,
        };
    }
    let accept: &dyn Fn(&Row) -> bool = match row_filter {
        Some(f) => &**f,
        None => &accept_all,
    };
    let data: Vec<Row> = data.into_iter().filter(|r| accept(r)).collect();
    if data.is_empty() {
        let mut d = Map::new();
        d.insert("reason".into(), json!("empty-data"));
        return DetailedSearch {
            results: vec![],
            index: None,
            diagnostic: d,
        };
    }

    let documents: Vec<String> = data
        .iter()
        .map(|row| {
            search_cols
                .iter()
                .map(|c| row.get(*c).cloned().unwrap_or_default())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    let mut bm25 = Bm25::new();
    bm25.fit(&documents);
    let (search_query, rewrites) = rewrite_query_for_domain(catalog, query, routing_domain, &bm25);
    let ranked = bm25.score(&search_query);
    let top_score = ranked.first().map(|(_, s)| *s).unwrap_or(0.0);
    let runner_up_score = ranked.get(1).map(|(_, s)| *s).unwrap_or(0.0);
    let coverage = query_coverage(&bm25, &search_query);
    let abstain = top_score <= threshold.min_score
        || coverage < threshold.min_coverage
        || (threshold.min_margin > 0.0 && top_score - runner_up_score < threshold.min_margin);

    let mut results = Vec::new();
    if !abstain {
        for (idx, score) in ranked.iter().take(max_results) {
            if *score <= 0.0 {
                continue;
            }
            results.push(project_row(&data[*idx], output_cols));
        }
    }

    let mut d = Map::new();
    d.insert("normalized_query".into(), json!(normalize(query)));
    d.insert("search_query".into(), json!(search_query));
    d.insert("query_rewrites".into(), json!(rewrites));
    d.insert("top_score".into(), json!(top_score));
    d.insert("runner_up_score".into(), json!(runner_up_score));
    d.insert("margin".into(), json!(top_score - runner_up_score));
    d.insert("token_coverage".into(), json!(coverage));
    d.insert("abstained".into(), json!(abstain));
    d.insert(
        "calibration_version".into(),
        json!(SEARCH_CALIBRATION_VERSION),
    );
    d.insert(
        "reason".into(),
        json!(if abstain { "low-confidence" } else { "matched" }),
    );
    DetailedSearch {
        results,
        index: Some(bm25),
        diagnostic: d,
    }
}

fn passes_threshold(index: &Bm25, query: &str, threshold: Threshold) -> bool {
    let ranked = index.score(query);
    let top = ranked.first().map(|(_, s)| *s).unwrap_or(0.0);
    let runner = ranked.get(1).map(|(_, s)| *s).unwrap_or(0.0);
    top > threshold.min_score
        && query_coverage(index, query) >= threshold.min_coverage
        && (threshold.min_margin <= 0.0 || top - runner >= threshold.min_margin)
}

fn suggest_terms(
    bm25: Option<&Bm25>,
    query: &str,
    limit: usize,
    threshold: Option<Threshold>,
) -> Vec<String> {
    let bm25 = match bm25 {
        Some(b) => b,
        None => return vec![],
    };
    let query_tokens: HashSet<String> = bm25.tokenize(query).into_iter().collect();
    if query_tokens.is_empty() {
        return vec![];
    }
    let qt_chars: Vec<Vec<char>> = query_tokens.iter().map(|t| t.chars().collect()).collect();
    let mut candidates: Vec<(f64, f64, String)> = Vec::new();
    for term in bm25.vocabulary() {
        if query_tokens.contains(term) {
            continue;
        }
        let term_chars: Vec<char> = term.chars().collect();
        let similarity = qt_chars
            .iter()
            .map(|token| difflib::ratio_chars(token, &term_chars))
            .fold(0.0f64, f64::max);
        if similarity >= 0.72
            && threshold
                .map(|t| passes_threshold(bm25, term, t))
                .unwrap_or(true)
        {
            let freq = *bm25.doc_freqs.get(term).unwrap_or(&0) as f64;
            candidates.push((-similarity, -freq, term.clone()));
        }
    }
    candidates.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.2.cmp(&b.2))
    });
    candidates
        .into_iter()
        .take(limit)
        .map(|(_, _, t)| t)
        .collect()
}

fn row_identities(row: &Row, fields: &[&str]) -> Vec<String> {
    let mut identities = Vec::new();
    for field in fields {
        if *field == "Aliases" {
            for value in row.get(*field).cloned().unwrap_or_default().split('|') {
                let v = value.trim();
                if !v.is_empty() {
                    identities.push(v.to_string());
                }
            }
        } else {
            let v = row.get(*field).cloned().unwrap_or_default();
            let v = v.trim();
            if !v.is_empty() {
                identities.push(v.to_string());
            }
        }
    }
    identities
}

fn suggest_identities(rows: &[Row], query: &str, fields: &[&str], limit: usize) -> Vec<String> {
    let tokenizer = Bm25::new();
    let query_tokens: HashSet<String> = tokenizer.tokenize(query).into_iter().collect();
    if query_tokens.is_empty() {
        return vec![];
    }
    let qt_chars: Vec<Vec<char>> = query_tokens.iter().map(|t| t.chars().collect()).collect();
    let mut candidates: Vec<(f64, usize, String)> = Vec::new();
    for row in rows {
        for identity in row_identities(row, fields) {
            let identity_tokens: HashSet<String> =
                tokenizer.tokenize(&identity).into_iter().collect();
            if identity_tokens.is_empty() {
                continue;
            }
            let it_chars: Vec<Vec<char>> = identity_tokens
                .iter()
                .map(|t| t.chars().collect())
                .collect();
            let mut similarity = 0.0f64;
            for source in &qt_chars {
                for target in &it_chars {
                    similarity = similarity.max(difflib::ratio_chars(source, target));
                }
            }
            if similarity >= 0.72 && identity.to_lowercase() != query.trim().to_lowercase() {
                candidates.push((-similarity, identity_tokens.len(), identity.clone()));
            }
        }
    }
    // sorted(set(candidates)) — dedupe identical tuples then sort.
    candidates.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.cmp(&b.1))
            .then(a.2.cmp(&b.2))
    });
    candidates.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1 && a.2 == b.2);
    candidates
        .into_iter()
        .take(limit)
        .map(|(_, _, id)| id)
        .collect()
}

fn load_product_keywords(catalog: &Catalog) -> Vec<String> {
    let seed: Vec<&str> = vec![
        "saas",
        "ecommerce",
        "fintech",
        "healthcare",
        "gaming",
        "portfolio",
        "crypto",
        "fitness",
        "marketplace",
        "banking",
        "cybersecurity",
        "education",
        "travel",
        "restaurant",
        "real estate",
        "social media",
        "beauty",
        "spa",
        "salon",
        "wellness",
        "booking",
    ];
    let mut keywords: HashSet<String> = seed.iter().map(|s| s.to_string()).collect();
    let file = CSV_CONFIG
        .iter()
        .find(|(d, _, _, _)| *d == "product")
        .map(|(_, f, _, _)| *f)
        .unwrap();
    static PARENS: OnceLock<Regex> = OnceLock::new();
    let parens = PARENS.get_or_init(|| Regex::new(r"\([^)]*\)").unwrap());
    if let Ok(rows) = catalog.rows(file) {
        for row in &rows {
            let label = parens
                .replace_all(
                    row.get("Product Type").map(|s| s.as_str()).unwrap_or(""),
                    "",
                )
                .trim()
                .to_lowercase();
            if label.len() >= 4 {
                keywords.insert(label);
            }
        }
    }
    let mut out: Vec<String> = keywords.into_iter().collect();
    out.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    out
}

fn domain_keywords(catalog: &Catalog) -> HashMap<&'static str, Vec<String>> {
    let mut m: HashMap<&'static str, Vec<String>> = HashMap::new();
    let to_vec = |items: &[&str]| items.iter().map(|s| s.to_string()).collect();
    m.insert(
        "color",
        to_vec(&[
            "color",
            "palette",
            "hex",
            "rgb",
            "token",
            "semantic",
            "accent",
            "destructive",
            "muted",
            "foreground",
        ]),
    );
    m.insert(
        "chart",
        to_vec(&[
            "time series",
            "chart",
            "graph",
            "visualization",
            "trend",
            "bar chart",
            "pie",
            "scatter",
            "heatmap",
            "funnel",
            "forecast",
        ]),
    );
    m.insert(
        "landing",
        to_vec(&[
            "landing",
            "page",
            "cta",
            "conversion",
            "hero",
            "testimonial",
            "pricing",
            "section",
        ]),
    );
    m.insert("product", load_product_keywords(catalog));
    m.insert(
        "style",
        to_vec(&[
            "style",
            "design",
            "ui",
            "minimalism",
            "glassmorphism",
            "neumorphism",
            "brutalism",
            "dark mode",
            "flat",
            "aurora",
            "css",
            "implementation",
            "variable",
            "checklist",
            "tailwind",
        ]),
    );
    m.insert(
        "ux",
        to_vec(&[
            "ux",
            "usability",
            "accessibility",
            "wcag",
            "touch",
            "scroll",
            "animation",
            "keyboard",
            "navigation",
            "mobile",
        ]),
    );
    m.insert(
        "typography",
        to_vec(&[
            "font pairing",
            "typography pairing",
            "heading font",
            "body font",
        ]),
    );
    m.insert(
        "google-fonts",
        to_vec(&[
            "google font",
            "font family",
            "font weight",
            "font style",
            "variable font",
            "noto",
            "font for",
            "find font",
            "font subset",
            "font language",
            "monospace font",
            "serif font",
            "sans serif font",
            "display font",
            "handwriting font",
            "font",
            "typography",
            "serif",
            "sans",
        ]),
    );
    m.insert(
        "icons",
        to_vec(&[
            "icon",
            "icons",
            "lucide",
            "phosphor",
            "heroicons",
            "symbol",
            "glyph",
            "pictogram",
            "svg icon",
        ]),
    );
    m.insert(
        "gsap",
        to_vec(&[
            "gsap",
            "quickto",
            "scrolltrigger",
            "stagger",
            "magnetic cursor",
            "parallax",
            "page transition",
            "scroll reveal",
            "scroll-triggered",
            "scrollytelling",
            "flip plugin",
            "splittext",
            "shimmer",
            "skeleton loader",
        ]),
    );
    m.insert(
        "react",
        to_vec(&[
            "react",
            "next.js",
            "nextjs",
            "suspense",
            "memo",
            "usecallback",
            "useeffect",
            "rerender",
            "bundle",
            "waterfall",
            "barrel",
            "dynamic import",
            "rsc",
            "server component",
        ]),
    );
    m.insert(
        "web",
        to_vec(&[
            "aria",
            "focus",
            "outline",
            "semantic",
            "virtualize",
            "autocomplete",
            "form",
            "input type",
            "preconnect",
            "drag reorder",
            "single pointer",
            "touch target",
            "native accessibility",
        ]),
    );
    m
}

fn word_char_at(text: &str, byte_idx: usize) -> bool {
    text.as_bytes()
        .get(byte_idx)
        .map(|b| is_word_byte(*b))
        .unwrap_or(false)
}

/// `_contains_phrase` — regex-escaped literal at word boundaries for phrases
/// containing a word char; plain substring otherwise.
fn contains_phrase(text: &str, phrase: &str) -> bool {
    if phrase.chars().any(|c| c.is_alphanumeric() || c == '_') {
        let lower = text.to_lowercase();
        let needle = phrase.to_lowercase();
        let re = match Regex::new(&format!("(?i){}", regex::escape(&needle))) {
            Ok(r) => r,
            Err(_) => return false,
        };
        for m in re.find_iter(&lower) {
            let prev_ok = m.start() == 0 || !word_char_at(&lower, m.start() - 1);
            let next_ok = m.end() >= lower.len() || !word_char_at(&lower, m.end());
            if prev_ok && next_ok {
                return true;
            }
        }
        false
    } else {
        text.contains(phrase)
    }
}

fn rewrite_query_for_domain(
    catalog: &Catalog,
    query: &str,
    domain: Option<&str>,
    index: &Bm25,
) -> (String, Vec<String>) {
    let domain = match domain {
        Some(d) if CSV_CONFIG.iter().any(|(name, _, _, _)| *name == d) => d,
        _ => return (query.to_string(), vec![]),
    };
    let keywords = domain_keywords(catalog);
    if !keywords.contains_key(domain) {
        return (query.to_string(), vec![]);
    }
    let normalized = normalize(&query.to_lowercase());
    let vocabulary: HashSet<&String> = index.idf.keys().collect();
    let mut rewrites: Vec<String> = Vec::new();
    let mut replacement_terms: Vec<String> = Vec::new();
    let rewrite_map: HashMap<&str, Option<&str>> =
        domain_query_rewrites(domain).iter().copied().collect();
    let tokenizer = Bm25::new();
    for keyword in &keywords[domain] {
        if !contains_phrase(&normalized, keyword) {
            continue;
        }
        let kw_tokens: HashSet<String> = tokenizer.tokenize(keyword).into_iter().collect();
        if kw_tokens.iter().any(|t| vocabulary.contains(t)) {
            continue;
        }
        if let Some(Some(replacement)) = rewrite_map.get(keyword.as_str()) {
            rewrites.push(format!("{}->{}", keyword, replacement));
            replacement_terms.push(replacement.to_string());
        }
    }
    if replacement_terms.is_empty() {
        return (query.to_string(), vec![]);
    }
    let mut terms: Vec<String> = replacement_terms.clone();
    terms.sort();
    terms.dedup();
    rewrites.sort();
    rewrites.dedup();
    (format!("{} {}", query, terms.join(" ")), rewrites)
}

const DOMAIN_TIEBREAK_ORDER: &[&str] = &[
    "ux",
    "product",
    "style",
    "color",
    "typography",
    "google-fonts",
    "chart",
    "landing",
    "icons",
    "gsap",
    "react",
    "web",
];

fn tiebreak_rank(domain: &str) -> usize {
    DOMAIN_TIEBREAK_ORDER
        .iter()
        .position(|d| *d == domain)
        .unwrap_or(999)
}

/// `detect_domain(query)` → (domain, runner_up or None).
pub fn detect_domain(catalog: &Catalog, query: &str) -> (String, Option<String>) {
    let query_lower = normalize(&query.to_lowercase());
    let keywords = domain_keywords(catalog);

    let mut scores: HashMap<&str, f64> = HashMap::new();
    for (domain, kws) in &keywords {
        let mut total = 0.0;
        for kw in kws {
            if contains_phrase(&query_lower, kw) {
                let specificity = kw.split_whitespace().count().max(1) as f64;
                total += if *domain != "product" {
                    2.0 * specificity
                } else {
                    specificity
                };
            }
        }
        scores.insert(domain, total);
    }
    static HEX_RE: OnceLock<Regex> = OnceLock::new();
    let hex_re =
        HEX_RE.get_or_init(|| Regex::new(r"(?i)(?:^|[^\w])#[0-9a-f]{3,8}(?:[^\w]|$)").unwrap());
    if hex_re.is_match(&query_lower) {
        *scores.entry("color").or_insert(0.0) += 2.0;
    }

    let mut ranked: Vec<(&str, f64)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(tiebreak_rank(a.0).cmp(&tiebreak_rank(b.0)))
    });
    let (best_domain, best_score) = ranked.first().copied().unwrap_or(("style", 0.0));
    let result = if best_score > 0.0 {
        best_domain
    } else {
        "style"
    };
    let runner_up = ranked
        .get(1)
        .filter(|(_, s)| *s > 0.0)
        .map(|(d, _)| d.to_string());
    (result.to_string(), runner_up)
}

fn style_identity(rows: &[Row], query: &str, allow_contained: bool) -> Option<Row> {
    let folded = query.trim().to_lowercase();
    static WORD_RE: OnceLock<Regex> = OnceLock::new();
    let word_re = WORD_RE.get_or_init(|| Regex::new(r"\w+").unwrap());
    let query_tokens: HashSet<String> = word_re
        .find_iter(&normalize(&folded))
        .map(|m| m.as_str().to_string())
        .collect();
    let generic_tokens: HashSet<&str> = ["app", "design", "interface", "style", "system", "ui"]
        .iter()
        .copied()
        .collect();
    let mut candidates: Vec<(usize, usize, usize, Row)> = Vec::new();
    for row in rows {
        let identities = row_identities(row, STYLE_IDENTITY_FIELDS);
        if identities.iter().any(|id| id.to_lowercase() == folded) {
            return Some(row.clone());
        }
        if !allow_contained {
            continue;
        }
        for identity in &identities {
            let identity_tokens: HashSet<String> = word_re
                .find_iter(&normalize(&identity.to_lowercase()))
                .map(|m| m.as_str().to_string())
                .collect();
            if !identity_tokens.is_empty()
                && identity_tokens.is_subset(&query_tokens)
                && identity_tokens.iter().any(|t| t.len() >= 4)
            {
                let distinctive: HashSet<&String> = identity_tokens
                    .iter()
                    .filter(|t| !generic_tokens.contains(t.as_str()))
                    .collect();
                candidates.push((
                    distinctive.len(),
                    identity_tokens.len(),
                    identity.len(),
                    (*row).clone(),
                ));
            }
        }
    }
    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by_key(|x| std::cmp::Reverse((x.0, x.1)));
    let best = (candidates[0].0, candidates[0].1, candidates[0].2);
    // Python builds an insertion-ordered dict {Style ID: row} from the tied
    // candidates (last write wins) and returns the single entry only when the
    // tie produced exactly one distinct Style ID.
    let mut best_rows: Vec<(String, Row)> = Vec::new();
    for c in &candidates {
        if (c.0, c.1, c.2) == best {
            let key = c.3.get("Style ID").cloned().unwrap_or_default();
            if let Some(existing) = best_rows.iter_mut().find(|(k, _)| *k == key) {
                existing.1 = c.3.clone();
            } else {
                best_rows.push((key, c.3.clone()));
            }
        }
    }
    if best_rows.len() == 1 {
        Some(best_rows[0].1.clone())
    } else {
        None
    }
}

fn exact_row_identity(rows: &[Row], query: &str, fields: &[&str]) -> Option<Row> {
    let folded = query.trim().to_lowercase();
    let mut matches: Vec<Row> = Vec::new();
    for row in rows {
        if row_identities(row, fields)
            .iter()
            .any(|id| id.to_lowercase() == folded)
        {
            matches.push(row.clone());
        }
    }
    if matches.len() == 1 {
        Some(matches.remove(0))
    } else {
        None
    }
}

fn valid_max_results(value: i64) -> bool {
    (1..=20).contains(&value)
}

/// `_style_search_destination` — resolve deprecated in-domain parents or
/// produce a cross-domain redirect.
/// Returns (Option<row>, Option<redirect>).
fn style_search_destination(rows: &[Row], matched: Option<Row>) -> (Option<Row>, Option<Value>) {
    let matched = match matched {
        Some(m) => m,
        None => return (None, None),
    };
    if matched
        .get("Status")
        .map(|s| s.as_str())
        .unwrap_or("active")
        != "deprecated"
    {
        return (Some(matched), None);
    }
    let parent_id = matched
        .get("Parent Style ID")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if !parent_id.is_empty() {
        let parent = rows
            .iter()
            .find(|r| r.get("Style ID").map(|s| s.as_str()) == Some(parent_id.as_str()))
            .cloned();
        return (parent, None);
    }
    let domain = matched
        .get("Replacement Domain")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let replacement_id = matched
        .get("Replacement ID")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if domain == "style" && !replacement_id.is_empty() {
        let replacement = rows
            .iter()
            .find(|r| r.get("Style ID").map(|s| s.as_str()) == Some(replacement_id.as_str()))
            .cloned();
        return (replacement, None);
    }
    if !domain.is_empty() && !replacement_id.is_empty() {
        return (None, Some(json!({"domain": domain, "id": replacement_id})));
    }
    (None, None)
}

/// `search(query, domain, max_results, diagnostics)` — main search entry.
pub fn search(
    catalog: &Catalog,
    query: &str,
    domain: Option<&str>,
    max_results: i64,
    diagnostics: bool,
) -> Value {
    if !valid_max_results(max_results) {
        return json!({
            "error": "max_results must be an integer from 1 to 20",
            "domain": domain,
        });
    }
    let auto_detected = domain.is_none();
    let mut runner_up: Option<String> = None;
    let mut style_rows: Option<Vec<Row>> = None;
    let mut exact_style: Option<Row> = None;
    let mut redirect: Option<Value> = None;
    let mut domain_owned: Option<String> = domain.map(|d| d.to_string());

    if domain.is_none() {
        let style_file = csv_config_file("style");
        style_rows = Some(catalog.rows_or_empty(style_file));
        let matched_style = style_identity(style_rows.as_ref().unwrap(), query, false);
        if let Some(m) = matched_style {
            domain_owned = Some("style".to_string());
            let (exact, redir) = style_search_destination(style_rows.as_ref().unwrap(), Some(m));
            exact_style = exact;
            redirect = redir;
        } else {
            let (d, r) = detect_domain(catalog, query);
            domain_owned = Some(d);
            runner_up = r;
        }
    }
    let domain = domain_owned.unwrap_or_else(|| "style".to_string());

    let search_domain = if CSV_CONFIG.iter().any(|(name, _, _, _)| *name == domain) {
        domain.clone()
    } else {
        "style".to_string()
    };
    let (_, file, search_cols, output_cols) = CSV_CONFIG
        .iter()
        .find(|(name, _, _, _)| *name == search_domain)
        .copied()
        .unwrap();
    let filepath = file;
    if !catalog.path(filepath).exists() {
        return json!({
            "error": format!("File not found: {}", catalog.path(filepath).display()),
            "domain": domain,
        });
    }

    let mut landing_rows: Vec<Row> = Vec::new();
    if search_domain == "style" && exact_style.is_none() && redirect.is_none() {
        if style_rows.is_none() {
            style_rows = Some(catalog.rows_or_empty(filepath));
        }
        let found = style_identity(style_rows.as_ref().unwrap(), query, true);
        let (exact, redir) = style_search_destination(style_rows.as_ref().unwrap(), found);
        exact_style = exact;
        redirect = redir;
    } else if search_domain == "landing" {
        landing_rows = catalog.rows_or_empty(filepath);
        exact_style = exact_row_identity(&landing_rows, query, LANDING_IDENTITY_FIELDS);
    }

    let results: Vec<Map<String, Value>>;
    let mut bm25: Option<Bm25> = None;
    let diagnostic: Map<String, Value>;
    if let Some(exact) = &exact_style {
        results = vec![project_row(exact, output_cols)];
        diagnostic = exact_match_diagnostic(query, "exact-identity");
    } else if redirect.is_some() {
        results = vec![];
        let mut d = Map::new();
        d.insert("normalized_query".into(), json!(normalize(query)));
        d.insert("search_query".into(), json!(query));
        d.insert("query_rewrites".into(), json!(Vec::<String>::new()));
        d.insert("abstained".into(), json!(true));
        d.insert(
            "calibration_version".into(),
            json!(SEARCH_CALIBRATION_VERSION),
        );
        d.insert("reason".into(), json!("cross-domain-redirect"));
        diagnostic = d;
    } else {
        let style_filter: Option<RowFilter> = if search_domain == "style" {
            Some(Box::new(|row: &Row| {
                row.get("Status").map(|s| s.as_str()).unwrap_or("active") == "active"
            }))
        } else {
            None
        };
        let detailed = search_csv_detailed(
            catalog,
            filepath,
            search_cols,
            output_cols,
            query,
            max_results as usize,
            search_threshold(&search_domain),
            Some(&search_domain),
            style_filter.as_ref(),
        );
        results = detailed.results;
        bm25 = detailed.index;
        diagnostic = detailed.diagnostic;
    }

    let mut results = results;
    let mut diagnostic = diagnostic;
    if search_domain == "icons" && contains_phrase(&normalize(&query.to_lowercase()), "lucide") {
        results = vec![];
        diagnostic.insert("abstained".into(), json!(true));
        diagnostic.insert("reason".into(), json!("unsupported-library"));
    }

    let mut out = Map::new();
    out.insert("domain".into(), json!(domain));
    out.insert("query".into(), json!(query));
    out.insert("file".into(), json!(file));
    out.insert("count".into(), json!(results.len()));
    out.insert(
        "results".into(),
        Value::Array(results.iter().map(|r| Value::Object(r.clone())).collect()),
    );
    if auto_detected {
        out.insert("auto_detected".into(), json!(true));
        if let Some(r) = &runner_up {
            out.insert("runner_up_domain".into(), json!(r));
        }
    }
    if let Some(redir) = &redirect {
        out.insert("redirect".into(), redir.clone());
    }
    if let Some(err) = diagnostic.get("error") {
        out.insert("error".into(), err.clone());
    }
    if results.is_empty() {
        if search_domain == "landing" {
            out.insert(
                "suggestions".into(),
                json!(suggest_identities(
                    &landing_rows,
                    query,
                    LANDING_IDENTITY_FIELDS,
                    6
                )),
            );
        } else {
            out.insert(
                "suggestions".into(),
                json!(suggest_terms(
                    bm25.as_ref(),
                    query,
                    6,
                    Some(search_threshold(&search_domain))
                )),
            );
        }
    }
    if diagnostics {
        out.insert("diagnostics".into(), Value::Object(diagnostic));
    }
    Value::Object(out)
}

pub fn csv_config_file(domain: &str) -> &'static str {
    CSV_CONFIG
        .iter()
        .find(|(name, _, _, _)| *name == domain)
        .map(|(_, f, _, _)| *f)
        .unwrap_or("")
}

// ============ STACK SEARCH ============

fn stack_query_requests_legacy(query: &str, stack: &str) -> bool {
    let normalized = normalize(&query.to_lowercase());
    if legacy_only_stacks().contains(stack) {
        return true;
    }
    let versions = stack_current_versions();
    let current_version = versions.get(stack);
    let stack_name = STACK_QUERY_NAMES
        .iter()
        .find(|(k, _)| *k == stack)
        .map(|(_, v)| *v);
    if let (Some(current), Some(name)) = (current_version, stack_name) {
        let pattern = format!(
            r"\b(?:{})\s*(?:sdk|ui)?\s*(?:[@(]\s*)?(?:v(?:ersion)?\s*)?(\d+)(?:\.(\d+))?\s*\)?",
            name
        );
        let re = Regex::new(&pattern).unwrap();
        let mut requested_versions: Vec<Vec<i64>> = Vec::new();
        for caps in re.captures_iter(&normalized) {
            let mut v = Vec::new();
            for i in 1..=2 {
                if let Some(m) = caps.get(i) {
                    v.push(m.as_str().parse::<i64>().unwrap_or(0));
                }
            }
            requested_versions.push(v);
        }
        if stack == "threejs" {
            static R_RE: OnceLock<Regex> = OnceLock::new();
            let r_re = R_RE.get_or_init(|| Regex::new(r"\br(\d+)\b").unwrap());
            for m in r_re.captures_iter(&normalized) {
                requested_versions.push(vec![0, m[1].parse().unwrap_or(0)]);
            }
        }
        static MIGRATION_RE: OnceLock<Regex> = OnceLock::new();
        let migration_re = MIGRATION_RE.get_or_init(|| {
            Regex::new(r"\b(?:migrat\w*|upgrad\w*|replac\w*|instead|modern|current)\b").unwrap()
        });
        let migration_intent = migration_re.is_match(&normalized);
        if !requested_versions.is_empty() {
            let prefix: Vec<i64> = current.clone();
            let cmp_at = |requested: &[i64]| {
                let upto = requested.len().min(prefix.len());
                &prefix[..upto]
            };
            if migration_intent && requested_versions.iter().any(|r| r.as_slice() >= cmp_at(r)) {
                return false;
            }
            return requested_versions.iter().all(|r| r.as_slice() < cmp_at(r));
        }
    }
    static MIGRATION2_RE: OnceLock<Regex> = OnceLock::new();
    let migration_re = MIGRATION2_RE.get_or_init(|| {
        Regex::new(r"\b(?:migrat\w*|upgrad\w*|replac\w*|instead|modern|current)\b").unwrap()
    });
    if migration_re.is_match(&normalized) {
        return false;
    }
    static LEGACY_RE: OnceLock<Regex> = OnceLock::new();
    let legacy_re = LEGACY_RE.get_or_init(|| Regex::new(r"\b(?:legacy|deprecated)\b").unwrap());
    legacy_re.is_match(&normalized)
}

/// `_stack_row_filter` → (filter, variant label).
fn stack_row_filter<'a>(rows: &'a [Row], query: &str, stack: &str) -> (RowFilter<'a>, String) {
    let statuses: HashSet<&str> = rows
        .iter()
        .map(|r| r.get("Status").map(|s| s.as_str()).unwrap_or("unverified"))
        .collect();
    let has_legacy = statuses.contains("deprecated");
    let requests_legacy = stack_query_requests_legacy(query, stack);

    let status_filter: RowFilter;
    let variant: String;
    if has_legacy && requests_legacy {
        status_filter =
            Box::new(|row: &Row| row.get("Status").map(|s| s.as_str()) == Some("deprecated"));
        variant = "legacy-only".to_string();
    } else if requests_legacy && stack_current_versions().contains_key(stack) {
        return (Box::new(|_: &Row| false), "legacy-unavailable".to_string());
    } else if statuses.contains("active") {
        status_filter =
            Box::new(|row: &Row| row.get("Status").map(|s| s.as_str()) == Some("active"));
        variant = "current-only".to_string();
    } else {
        status_filter = Box::new(|row: &Row| {
            row.get("Status")
                .map(|s| s.as_str())
                .unwrap_or("unverified")
                != "deprecated"
        });
        variant = "non-legacy".to_string();
    }

    if stack != "shadcn" {
        return (status_filter, variant);
    }
    let normalized = normalize(&query.to_lowercase());
    let requested_base = if normalized.contains("base ui") {
        Some("base")
    } else if normalized.contains("react aria") {
        Some("aria")
    } else if normalized.contains("radix") || normalized.contains("aschild") {
        Some("radix")
    } else {
        None
    };
    let base = match requested_base {
        Some(b) => b,
        None => return (status_filter, variant),
    };
    static BASE_RE: OnceLock<Regex> = OnceLock::new();
    let base_re = BASE_RE.get_or_init(|| Regex::new(r"\bbase=([^;]+)").unwrap());
    let base_owned = base.to_string();
    let filter = Box::new(move |row: &Row| {
        let applies = row
            .get("Applies To")
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        let bases: Vec<String> = match base_re.captures(&applies) {
            Some(c) => c[1].split('|').map(|s| s.to_string()).collect(),
            None => vec![],
        };
        status_filter(row) && bases.contains(&base_owned)
    });
    (filter, format!("{};base={}", variant, base))
}

fn exact_stack_identifier(rows: &[Row], query: &str, row_filter: &RowFilter) -> Option<Row> {
    let identifier = query.trim();
    if identifier.len() < 6 || identifier.chars().any(|c| c.is_whitespace()) {
        return None;
    }
    let re = Regex::new(&format!("(?i){}", regex::escape(identifier))).unwrap();
    let fields = [
        "Guideline",
        "Description",
        "Do",
        "Don't",
        "Code Good",
        "Code Bad",
    ];
    let ident_char = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut matches: Vec<Row> = Vec::new();
    for row in rows {
        if !row_filter(row) {
            continue;
        }
        let hit = fields.iter().any(|field| {
            let text = row.get(*field).map(|s| s.as_str()).unwrap_or("");
            re.find_iter(text).any(|m| {
                let prev_ok = m.start() == 0 || !ident_char(text.as_bytes()[m.start() - 1]);
                let next_ok = m.end() >= text.len() || !ident_char(text.as_bytes()[m.end()]);
                prev_ok && next_ok
            })
        });
        if hit {
            matches.push(row.clone());
        }
    }
    if matches.len() == 1 {
        Some(matches.remove(0))
    } else {
        None
    }
}

fn legacy_successor_guidance(
    rows: &[Row],
    query: &str,
    stack: &str,
    row_filter: &RowFilter,
) -> Option<Row> {
    let normalized = normalize(&query.to_lowercase());
    static NEW_APP_RE: OnceLock<Regex> = OnceLock::new();
    let new_app = NEW_APP_RE.get_or_init(|| {
        Regex::new(r"\b(?:brand new|new)\s+(?:app|application|project)\b").unwrap()
    });
    if !legacy_only_stacks().contains(stack) || !new_app.is_match(&normalized) {
        return None;
    }
    static SUCCESSOR_RE: OnceLock<Regex> = OnceLock::new();
    let successor = SUCCESSOR_RE.get_or_init(|| {
        Regex::new(r"\b(?:prefer|choose|use)\b.*\bnew (?:apps?|projects?)\b").unwrap()
    });
    let mut matches: Vec<Row> = Vec::new();
    for row in rows {
        if !row_filter(row) {
            continue;
        }
        let text = format!(
            "{} {} {}",
            row.get("Guideline").cloned().unwrap_or_default(),
            row.get("Description").cloned().unwrap_or_default(),
            row.get("Do").cloned().unwrap_or_default()
        )
        .to_lowercase();
        if successor.is_match(&text) {
            matches.push(row.clone());
        }
    }
    if matches.len() == 1 {
        Some(matches.remove(0))
    } else {
        None
    }
}

/// `search_stack(query, stack, max_results, diagnostics)`.
pub fn search_stack(
    catalog: &Catalog,
    query: &str,
    stack: &str,
    max_results: i64,
    diagnostics: bool,
) -> Value {
    if !valid_max_results(max_results) {
        return json!({
            "error": "max_results must be an integer from 1 to 20",
            "stack": stack,
        });
    }
    if !STACK_CONFIG.iter().any(|(name, _)| *name == stack) {
        return json!({
            "error": format!("Unknown stack: {}. Available: {}", stack, available_stacks().join(", ")),
        });
    }
    let file = STACK_CONFIG
        .iter()
        .find(|(name, _)| *name == stack)
        .map(|(_, f)| *f)
        .unwrap();
    if !catalog.path(file).exists() {
        return json!({
            "error": format!("Stack file not found: {}", catalog.path(file).display()),
            "stack": stack,
        });
    }

    let rows = catalog.rows_or_empty(file);
    let (row_filter, variant) = stack_row_filter(&rows, query, stack);
    let threshold = if variant == "legacy-only" {
        NO_THRESHOLD
    } else {
        STACK_THRESHOLD
    };

    let results: Vec<Map<String, Value>>;
    let mut bm25: Option<Bm25> = None;
    let diagnostic: Map<String, Value>;
    let exact = legacy_successor_guidance(&rows, query, stack, &row_filter)
        .or_else(|| exact_stack_identifier(&rows, query, &row_filter));
    if let Some(row) = exact {
        results = vec![project_row(&row, STACK_OUTPUT_COLS)];
        diagnostic = exact_match_diagnostic(query, "exact-identifier");
    } else {
        let detailed = search_csv_detailed(
            catalog,
            file,
            STACK_SEARCH_COLS,
            STACK_OUTPUT_COLS,
            query,
            max_results as usize,
            threshold,
            None,
            Some(&row_filter),
        );
        results = detailed.results;
        bm25 = detailed.index;
        diagnostic = detailed.diagnostic;
    }

    let mut out = Map::new();
    out.insert("domain".into(), json!("stack"));
    out.insert("stack".into(), json!(stack));
    out.insert("query".into(), json!(query));
    out.insert("file".into(), json!(file));
    out.insert("count".into(), json!(results.len()));
    out.insert(
        "results".into(),
        Value::Array(results.iter().map(|r| Value::Object(r.clone())).collect()),
    );
    if let Some(err) = diagnostic.get("error") {
        out.insert("error".into(), err.clone());
    }
    if results.is_empty() {
        out.insert(
            "suggestions".into(),
            json!(suggest_terms(bm25.as_ref(), query, 6, Some(threshold))),
        );
    }
    if diagnostics {
        out.insert("diagnostics".into(), Value::Object(diagnostic));
    }
    Value::Object(out)
}
