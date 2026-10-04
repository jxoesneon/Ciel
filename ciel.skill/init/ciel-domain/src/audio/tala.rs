//! Port of `skills/procedural-audio/scripts/indian_tala_engine.py` —
//! Tala cycles and the Tihai solver `3*Phrase + 2*Dum = Cycles*TalaLen + 1`.
//! `ciel-audio tala` runs the __main__ demo printing grids + tihai solutions.

use serde_json::{json, Map, Value};

struct Tala {
    name: &'static str,
    beats: i64,
    /// Mirrors the Python TALAS table; retained for fidelity though the
    /// grid/tihai paths only consume name/beats/claps/khali.
    #[allow(dead_code)]
    vibhags: &'static [i64],
    claps: &'static [i64],
    khali: &'static [i64],
}

const TALAS: [Tala; 5] = [
    Tala {
        name: "Tintal",
        beats: 16,
        vibhags: &[4, 4, 4, 4],
        claps: &[1, 5, 13],
        khali: &[9],
    },
    Tala {
        name: "Jhaptal",
        beats: 10,
        vibhags: &[2, 3, 2, 3],
        claps: &[1, 3, 8],
        khali: &[6],
    },
    Tala {
        name: "Rupak",
        beats: 7,
        vibhags: &[3, 2, 2],
        claps: &[4, 6],
        khali: &[1],
    },
    Tala {
        name: "Ektaal",
        beats: 12,
        vibhags: &[2, 2, 2, 2, 2, 2],
        claps: &[1, 5, 9, 11],
        khali: &[3, 7],
    },
    Tala {
        name: "Keherwa",
        beats: 8,
        vibhags: &[4, 4],
        claps: &[1],
        khali: &[5],
    },
];

fn get_tala(tala_name: &str) -> &'static Tala {
    let lower = tala_name.to_lowercase();
    TALAS
        .iter()
        .find(|t| t.name.to_lowercase() == lower)
        .unwrap_or(&TALAS[0])
}

/// TalaEngine.generate_tihai — same dict shape as Python.
pub fn generate_tihai(tala_name: &str, target_cycle: i64, preferred_dum: f64) -> Value {
    let tala = get_tala(tala_name);
    let l = tala.beats;
    let total_target_beats = target_cycle * l + 1;

    let mut best: Option<Map<String, Value>> = None;
    let mut min_dum_diff = 999.0f64;

    for dum_steps in 0..16 {
        let dum = dum_steps as f64 * 0.5;
        let remaining = total_target_beats as f64 - 2.0 * dum;
        let third = remaining / 3.0;
        if remaining > 0.0 && third == crate::common::py::round_py(third, 4) {
            let phrase = third;
            let diff = (dum - preferred_dum).abs();
            if diff < min_dum_diff {
                min_dum_diff = diff;
                let mut m = Map::new();
                m.insert("phrase_beats".into(), json!(phrase));
                m.insert("dum_beats".into(), json!(dum));
                m.insert("total_target_beats".into(), json!(total_target_beats));
                m.insert("tala".into(), json!(tala.name));
                m.insert(
                    "start_beat_in_cycle".into(),
                    json!((l - ((total_target_beats - 1) % l)) % l + 1),
                );
                best = Some(m);
            }
        }
    }
    match best {
        Some(m) => Value::Object(m),
        None => json!({"error": "No integer solution found"}),
    }
}

/// TalaEngine.get_tala_grid
pub fn get_tala_grid(tala_name: &str) -> Vec<String> {
    let tala = get_tala(tala_name);
    let mut grid = Vec::new();
    for b in 1..=tala.beats {
        if b == 1 {
            grid.push("SAM(1)".to_string());
        } else if tala.claps.contains(&b) {
            grid.push(format!("CLAP({})", b));
        } else if tala.khali.contains(&b) {
            grid.push(format!("KHALI({})", b));
        } else {
            grid.push(format!("beat({})", b));
        }
    }
    grid
}

/// Python `print(dict)` formatting: single quotes, `key: value`.
fn fmt_py_dict(v: &Value) -> String {
    match v {
        Value::Object(m) => {
            let parts: Vec<String> = m
                .iter()
                .map(|(k, val)| format!("'{}': {}", k, fmt_py_scalar(val)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        _ => fmt_py_scalar(v),
    }
}

fn fmt_py_scalar(v: &Value) -> String {
    match v {
        Value::String(s) => format!("'{}'", s),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 && n.is_f64() {
                    crate::common::py::py_float(f)
                } else {
                    crate::common::py::py_num(f)
                }
            } else {
                n.to_string()
            }
        }
        Value::Bool(b) => {
            if *b {
                "True".into()
            } else {
                "False".into()
            }
        }
        other => other.to_string(),
    }
}

fn fmt_py_str_list(items: &[String]) -> String {
    let inner: Vec<String> = items.iter().map(|s| format!("'{}'", s)).collect();
    format!("[{}]", inner.join(", "))
}

pub fn run(_argv: &[String]) -> i32 {
    for t_name in ["tintal", "jhaptal", "rupak", "ektaal"] {
        let tala = get_tala(t_name);
        println!("\n--- TALA: {} (Beats: {}) ---", tala.name, tala.beats);
        println!("Metric Grid: {}", fmt_py_str_list(&get_tala_grid(t_name)));
        let tihai = generate_tihai(t_name, 1, 1.0);
        println!("Tihai 1-Cycle Resolution: {}", fmt_py_dict(&tihai));
    }
    0
}
