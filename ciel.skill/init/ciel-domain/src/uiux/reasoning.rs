//! Port of `skills/ui-ux-pro-max/scripts/reasoning_contract.py` — the closed,
//! non-executable grammar for design-system decision rules.
//!
//! `parse_decision_rules` validates a `condition -> [action]` JSON object and
//! `apply_decision_rules` deterministically expands the activated conditions
//! into style ids / constraints / pattern / mode. Neither ever executes data.

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fmt;
use std::sync::OnceLock;

/// Condition signals, in Python dict order.
pub const CONDITION_SIGNALS: &[(&str, &[&str])] = &[
    ("if_booking", &["booking", "appointment", "calendar"]),
    ("if_boutique", &["boutique"]),
    ("if_casual", &["casual", "playful"]),
    ("if_checkout", &["checkout", "payment", "purchase"]),
    ("if_children", &["child", "children", "kids"]),
    (
        "if_collaboration",
        &["collaboration", "multiplayer", "co-edit"],
    ),
    ("if_competitive", &["competitive", "leaderboard"]),
    (
        "if_content_focused",
        &["content", "article", "reading", "documentation"],
    ),
    (
        "if_conversion_focused",
        &["conversion", "sales", "signup", "purchase"],
    ),
    ("if_creative_field", &["creative", "artist", "portfolio"]),
    ("if_crop_focused", &["crop", "farm", "agriculture"]),
    ("if_dashboard", &["dashboard", "operations", "monitoring"]),
    (
        "if_data_heavy",
        &["data heavy", "data-heavy", "analytics", "large dataset"],
    ),
    ("if_delivery", &["delivery", "courier", "shipping"]),
    (
        "if_discovery_focused",
        &["discover", "discovery", "browse", "directory"],
    ),
    (
        "if_engagement_metric",
        &["engagement", "retention", "contribution"],
    ),
    (
        "if_experience_focused",
        &["experience", "immersive", "journey"],
    ),
    ("if_gamification", &["gamification", "badges", "streak"]),
    ("if_health", &["health", "medical", "patient"]),
    ("if_hero_needed", &["hero", "showcase", "launch"]),
    (
        "if_large_dataset",
        &["large dataset", "thousands", "millions"],
    ),
    ("if_light_mode_needed", &["light mode", "light theme"]),
    (
        "if_low_performance",
        &["low performance", "low-end", "slow device"],
    ),
    ("if_luxury", &["luxury", "premium", "high-end"]),
    ("if_medication", &["medication", "medicine", "prescription"]),
    ("if_meditation", &["meditation", "breathing", "mindfulness"]),
    (
        "if_minimal_portfolio",
        &["minimal portfolio", "simple portfolio"],
    ),
    (
        "if_mobile",
        &["mobile", "phone", "tablet", "ios", "android"],
    ),
    (
        "if_personalized",
        &["personalized", "personalised", "recommendation"],
    ),
    (
        "if_pre_launch",
        &["pre-launch", "prelaunch", "coming soon", "waitlist"],
    ),
    (
        "if_salary_focused",
        &["salary", "compensation", "pay range"],
    ),
    (
        "if_team_collaboration",
        &["team collaboration", "team workspace"],
    ),
    (
        "if_trust_needed",
        &["trust", "secure", "verified", "authority"],
    ),
    (
        "if_ux_focused",
        &["ux", "usability", "accessibility", "accessible"],
    ),
    (
        "if_video_ready",
        &["product video", "demo video", "video ready"],
    ),
];

const ACTION_PREFIXES: &[&str] = &["constraint", "style", "pattern", "mode"];
const TOKEN_ACTION_PREFIXES: &[&str] = &["constraint", "style"];

fn condition_allowed(condition: &str) -> bool {
    condition == "must_have" || CONDITION_SIGNALS.iter().any(|(name, _)| *name == condition)
}

fn token_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"\A[a-z0-9]+(?:-[a-z0-9]+)*\z").unwrap())
}

fn condition_patterns(condition: &str) -> Vec<regex::Regex> {
    CONDITION_SIGNALS
        .iter()
        .find(|(name, _)| *name == condition)
        .map(|(_, signals)| {
            signals
                .iter()
                .map(|signal| {
                    // (?<!\w)signal(?!\w) — implemented with consuming
                    // boundaries; is_match semantics are identical.
                    regex::Regex::new(&format!(r"(?:^|[^\w]){}(?:[^\w]|$)", regex::escape(signal)))
                        .unwrap()
                })
                .collect()
        })
        .unwrap_or_default()
}

/// JSON value that rejects duplicate object keys at any depth, matching
/// Python's `json.loads(raw, object_pairs_hook=_object_without_duplicates)`.
struct NoDupValue(Value);

impl<'de> Deserialize<'de> for NoDupValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(NoDupVisitor)
    }
}

struct NoDupVisitor;

impl<'de> Visitor<'de> for NoDupVisitor {
    type Value = NoDupValue;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("any JSON value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<NoDupValue, E> {
        Ok(NoDupValue(Value::Bool(v)))
    }
    fn visit_i64<E>(self, v: i64) -> Result<NoDupValue, E> {
        Ok(NoDupValue(Value::from(v)))
    }
    fn visit_u64<E>(self, v: u64) -> Result<NoDupValue, E> {
        Ok(NoDupValue(Value::from(v)))
    }
    fn visit_f64<E>(self, v: f64) -> Result<NoDupValue, E> {
        Ok(NoDupValue(Value::from(v)))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<NoDupValue, E> {
        Ok(NoDupValue(Value::from(v)))
    }
    fn visit_string<E>(self, v: String) -> Result<NoDupValue, E> {
        Ok(NoDupValue(Value::from(v)))
    }
    fn visit_none<E>(self) -> Result<NoDupValue, E> {
        Ok(NoDupValue(Value::Null))
    }
    fn visit_unit<E>(self) -> Result<NoDupValue, E> {
        Ok(NoDupValue(Value::Null))
    }
    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<NoDupValue, D::Error> {
        NoDupValue::deserialize(d)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<NoDupValue, A::Error> {
        let mut items = Vec::new();
        while let Some(NoDupValue(v)) = seq.next_element::<NoDupValue>()? {
            items.push(v);
        }
        Ok(NoDupValue(Value::Array(items)))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<NoDupValue, A::Error> {
        let mut out = Map::new();
        let mut seen = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !seen.insert(key.clone()) {
                return Err(de::Error::custom(format!(
                    "duplicate decision-rule key: {}",
                    key
                )));
            }
            let NoDupValue(value) = map.next_value::<NoDupValue>()?;
            out.insert(key, value);
        }
        Ok(NoDupValue(Value::Object(out)))
    }
}

fn validate_action(action: &Value) -> Result<(), String> {
    let text = action.as_str().unwrap_or("");
    if !action.is_string() || !text.contains(':') {
        return Err(format!(
            "action must use a known prefix: {}",
            action_repr(action)
        ));
    }
    let (prefix, value) = text.split_once(':').unwrap();
    if !ACTION_PREFIXES.contains(&prefix) {
        return Err(format!("unknown decision-rule action: {}", text));
    }
    if TOKEN_ACTION_PREFIXES.contains(&prefix) && !token_regex().is_match(value) {
        return Err(format!("invalid {} action value: {}", prefix, value));
    }
    if prefix == "pattern" && value.trim().is_empty() {
        return Err("pattern action must name a pattern".to_string());
    }
    if prefix == "mode" && value != "dark" && value != "light" {
        return Err("mode action must be dark or light".to_string());
    }
    Ok(())
}

fn action_repr(action: &Value) -> String {
    action
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| crate::common::jsonfmt::dumps(action))
}

/// `parse_decision_rules(raw)` — returns the ordered condition->actions map.
/// Errors mirror the Python `ValueError` messages.
pub fn parse_decision_rules(raw: &str) -> Result<Map<String, Value>, String> {
    let text = if raw.is_empty() { "{}" } else { raw };
    let parsed = serde_json::from_str::<NoDupValue>(text).map_err(|e| {
        if let Some(msg) = extract_dup_key(&e) {
            return msg;
        }
        format!("invalid decision-rule JSON: {}", e)
    })?;
    let rules = match parsed.0 {
        Value::Object(map) => map,
        _ => return Err("decision rules must be a JSON object".to_string()),
    };
    for (condition, actions) in &rules {
        if !condition_allowed(condition) {
            return Err(format!("unknown decision-rule condition: {}", condition));
        }
        let list = match actions.as_array() {
            Some(list) if !list.is_empty() => list,
            _ => {
                return Err(format!(
                    "{} must map to a non-empty action array",
                    condition
                ))
            }
        };
        for action in list {
            validate_action(action)?;
        }
        let mut seen = HashSet::new();
        for action in list {
            if !seen.insert(action.as_str().unwrap_or("")) {
                return Err(format!("{} contains duplicate actions", condition));
            }
        }
    }
    Ok(rules)
}

/// serde reports our duplicate-key error as a custom message with a
/// " at line N column M" suffix; surface the Python-style ValueError text.
fn extract_dup_key(err: &serde_json::Error) -> Option<String> {
    let msg = err.to_string();
    let start = msg.find("duplicate decision-rule key:")?;
    let rest = &msg[start..];
    let end = rest.find(" at line ").unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

pub struct AppliedRules {
    /// [{"condition": ..., "actions": [...]}] in rule order.
    pub activated: Vec<Value>,
    pub style_ids: Vec<String>,
    pub constraints: Vec<String>,
    pub pattern: Option<String>,
    pub mode: Option<String>,
}

/// `apply_decision_rules(rules, query)` — deterministic expansion with an
/// audit trail. Later `pattern:`/`mode:` actions overwrite earlier ones.
pub fn apply_decision_rules(rules: &Map<String, Value>, query: &str) -> AppliedRules {
    let normalized = query.to_lowercase();
    let mut result = AppliedRules {
        activated: Vec::new(),
        style_ids: Vec::new(),
        constraints: Vec::new(),
        pattern: None,
        mode: None,
    };
    for (condition, actions) in rules {
        let actions = actions.as_array().cloned().unwrap_or_default();
        let active = condition == "must_have"
            || condition_patterns(condition)
                .iter()
                .any(|pattern| pattern.is_match(&normalized));
        if !active {
            continue;
        }
        result.activated.push(serde_json::json!({
            "condition": condition,
            "actions": actions,
        }));
        for action in &actions {
            let text = action.as_str().unwrap_or("");
            let (prefix, value) = match text.split_once(':') {
                Some(pair) => pair,
                None => continue,
            };
            match prefix {
                "style" if !result.style_ids.iter().any(|v| v == value) => {
                    result.style_ids.push(value.to_string());
                }
                "constraint" if !result.constraints.iter().any(|v| v == value) => {
                    result.constraints.push(value.to_string());
                }
                "pattern" => result.pattern = Some(value.to_string()),
                "mode" => result.mode = Some(value.to_string()),
                _ => {}
            }
        }
    }
    result
}
