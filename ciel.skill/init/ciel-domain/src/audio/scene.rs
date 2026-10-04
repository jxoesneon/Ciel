//! Port of `skills/procedural-audio/scripts/scene_context_analyzer.py` —
//! .tscn / HTML scene analysis → AudioSceneManifest JSON.
//! CLI: `--file PATH --mood auto --out audio_scene_manifest.json`

use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use regex::Regex;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

fn analyze_godot_tscn(content: &str) -> Value {
    let node_re = Regex::new(r#"\[node\s+name="([^"]+)"\s+type="([^"]+)""#).unwrap();
    let mut nodes: Vec<Value> = Vec::new();
    let mut buttons: Vec<String> = Vec::new();
    let mut rigid_bodies: Vec<String> = Vec::new();
    let mut particles: Vec<String> = Vec::new();

    for line in content.lines() {
        if let Some(m) = node_re.captures(line) {
            let name = m[1].to_string();
            let node_type = m[2].to_string();
            nodes.push(json!({"name": name, "type": node_type}));
            if node_type.contains("Button")
                || node_type.contains("LineEdit")
                || node_type.contains("Slider")
            {
                buttons.push(name.clone());
            } else if node_type.contains("RigidBody") || node_type.contains("CharacterBody") {
                rigid_bodies.push(name.clone());
            } else if node_type.contains("Particle") || node_type.contains("CPUParticles") {
                particles.push(name.clone());
            }
        }
    }

    let color_re = Regex::new(r#"Color\(\s*["'](#[0-9a-fA-F]+)["']"#).unwrap();
    let mut hex_colors: Vec<String> = color_re
        .captures_iter(content)
        .map(|c| c[1].to_string())
        .collect();
    if hex_colors.is_empty() {
        hex_colors = vec!["#00FFFF".into(), "#FF00FF".into(), "#0A0A1A".into()];
    }
    hex_colors.truncate(5);

    json!({
        "engine": "GODOT_4",
        "total_nodes": nodes.len(),
        "buttons": buttons,
        "rigid_bodies": rigid_bodies,
        "particles": particles,
        "detected_colors": hex_colors
    })
}

fn analyze_html_dom(content: &str) -> Value {
    let btn_re = Regex::new(r"(?i)<button[^>]*>(.*?)</button>").unwrap();
    let input_re = Regex::new(r"(?i)<input[^>]*>").unwrap();
    let color_re = Regex::new(r"#([0-9a-fA-F]{6}|[0-9a-fA-F]{3})").unwrap();

    let buttons: Vec<String> = btn_re
        .captures_iter(content)
        .map(|c| c[1].chars().take(20).collect())
        .collect();
    let n_inputs = input_re.find_iter(content).count();
    let mut hex_colors: Vec<String> = color_re
        .captures_iter(content)
        .map(|c| format!("#{}", &c[1]))
        .collect();
    hex_colors.truncate(5);
    if hex_colors.is_empty() {
        hex_colors = vec!["#1E293B".into(), "#38BDF8".into(), "#F43F5E".into()];
    }

    json!({
        "engine": "WEB_DOM",
        "total_interactive": buttons.len() + n_inputs,
        "buttons": buttons,
        "detected_colors": hex_colors
    })
}

fn generate_manifest(analysis: &Value, prompt_mood: &str) -> Value {
    let colors = analysis
        .get("detected_colors")
        .cloned()
        .unwrap_or_else(|| json!(["#00FFFF", "#FF00FF"]));

    let mut mode = "Dorian";
    let mut key = "D";
    let mut valence = 0.0;
    let mut arousal = 0.4;
    let dominance = 0.2;

    let mood = prompt_mood.to_lowercase();
    if mood.contains("dark") || mood.contains("horror") {
        mode = "Phrygian";
        key = "C";
        valence = -0.7;
        arousal = 0.8;
    } else if mood.contains("peaceful") || mood.contains("pastoral") {
        mode = "Ionian";
        key = "G";
        valence = 0.8;
        arousal = 0.1;
    } else if mood.contains("cyber") || mood.contains("neon") {
        mode = "Dorian";
        key = "D";
        valence = 0.2;
        arousal = 0.6;
    }

    let mut layer3: Vec<Value> = Vec::new();
    if let Some(btns) = analysis.get("buttons").and_then(|b| b.as_array()) {
        for btn in btns {
            layer3.push(json!({
                "event_trigger": "pressed",
                "source_node_path": btn,
                "synth_model": "FM_CLICK",
                "parameters": {
                    "base_frequency": 2400.0,
                    "decay_seconds": 0.035,
                    "harmonicity": 2.0
                }
            }));
        }
    }
    if let Some(bodies) = analysis.get("rigid_bodies").and_then(|b| b.as_array()) {
        for body in bodies {
            layer3.push(json!({
                "event_trigger": "body_entered",
                "source_node_path": body,
                "synth_model": "PHYSICAL_IMPACT",
                "parameters": {
                    "base_frequency": 440.0,
                    "decay_seconds": 0.4,
                    "harmonicity": 1.414
                }
            }));
        }
    }

    let mut manifest = Map::new();
    manifest.insert("manifest_version".into(), json!("2.0.0"));
    manifest.insert(
        "scene_metadata".into(),
        json!({
            "scene_id": "auto_generated_scene",
            "scene_type": if analysis.get("engine") == Some(&json!("GODOT_4")) { "2D_GAME" } else { "WEB_APP" },
            "ambient_zone_name": "PrimaryZone"
        }),
    );
    manifest.insert(
        "synesthetic_profile".into(),
        json!({
            "dominant_palette": colors,
            "harmonic_mode": mode.to_uppercase(),
            "base_key": key,
            "tempo_bpm": if arousal > 0.5 { 118.0 } else { 84.0 },
            "vad_vector": {
                "valence": valence,
                "arousal": arousal,
                "dominance": dominance
            }
        }),
    );
    manifest.insert(
        "spatial_acoustic_bus_layout".into(),
        json!({
            "master_volume_db": 0.0,
            "reverb_room_size": 0.65,
            "reverb_damping": 0.35,
            "reverb_wet_send_db": -12.0,
            "occlusion_raycast_enabled": true
        }),
    );
    manifest.insert(
        "procedural_layers".into(),
        json!({
            "layer_1_ambience": {
                "sub_bass_drone": {
                    "carrier_freq_hz": 55.0,
                    "mod_index": 1.5,
                    "lfo_rate_hz": 0.12
                },
                "biome_granular_texture": {
                    "noise_type": "PINK",
                    "filter_cutoff_hz": 850.0,
                    "resonance_q": 2.2
                }
            },
            "layer_2_dynamic_music": {
                "chords": [
                    {"root_midi": 50, "intervals": [0, 3, 7, 10], "duration_beats": 4.0},
                    {"root_midi": 53, "intervals": [0, 4, 7, 11], "duration_beats": 4.0},
                    {"root_midi": 48, "intervals": [0, 4, 7, 10], "duration_beats": 4.0},
                    {"root_midi": 45, "intervals": [0, 3, 7, 10], "duration_beats": 4.0}
                ],
                "arpeggio_pattern": [0, 2, 3, 5, 7, 9, 10, 12],
                "reactive_stems": [
                    {"stem_id": "drone", "dti_threshold_min": 0.0, "dti_threshold_max": 1.0, "timbre_type": "SUB_WARMTH"},
                    {"stem_id": "pad", "dti_threshold_min": 0.2, "dti_threshold_max": 1.0, "timbre_type": "MODAL_STRINGS"},
                    {"stem_id": "arpeggio", "dti_threshold_min": 0.45, "dti_threshold_max": 1.0, "timbre_type": "FM_PLUCK"},
                    {"stem_id": "percussion", "dti_threshold_min": 0.65, "dti_threshold_max": 1.0, "timbre_type": "EUCLIDEAN_DRUMS"},
                    {"stem_id": "tension_stab", "dti_threshold_min": 0.85, "dti_threshold_max": 1.0, "timbre_type": "DISSONANT_BRASS"}
                ]
            },
            "layer_3_interactive_sfx": layer3
        }),
    );
    Value::Object(manifest)
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-audio scene",
        argv,
        &[
            ArgSpec::value("file", None, "file"),
            ArgSpec::value("mood", None, "mood").def("auto"),
            ArgSpec::value("out", None, "out").def("audio_scene_manifest.json"),
        ],
    );

    let file = args.get("file").map(|s| s.to_string());
    let mood = args.get_or("mood", "auto");
    let out = args.get_or("out", "audio_scene_manifest.json");

    let analysis = match file.as_deref() {
        Some(f) if Path::new(f).exists() => {
            let content = fs::read_to_string(f).unwrap_or_else(|_| {
                fs::read(f)
                    .map(|b| String::from_utf8_lossy(&b).into_owned())
                    .unwrap_or_default()
            });
            if f.ends_with(".tscn") {
                analyze_godot_tscn(&content)
            } else {
                analyze_html_dom(&content)
            }
        }
        _ => {
            println!("[SceneContextAnalyzer] No file specified, using generic default environment context.");
            json!({
                "engine": "GODOT_4",
                "total_nodes": 8,
                "buttons": ["StartButton", "OptionsButton", "QuitButton"],
                "rigid_bodies": ["Player", "Crate1"],
                "particles": ["AmbientParticles"],
                "detected_colors": ["#00FFFF", "#FF00FF", "#0A0A20"]
            })
        }
    };

    let manifest = generate_manifest(&analysis, &mood);
    if let Err(e) = fs::write(&out, jsonfmt::dumps_indent(&manifest, 2)) {
        eprintln!("Error writing {}: {}", out, e);
        return 1;
    }
    println!(
        "[SceneContextAnalyzer] Manifest generated successfully at '{}'.",
        out
    );
    0
}
