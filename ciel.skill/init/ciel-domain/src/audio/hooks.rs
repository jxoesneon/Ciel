//! Ports of `skills/procedural-audio/hooks/*.py` — five hooks dispatched as
//! `ciel-audio hook-{auto-wire,post-synthesize,pre-invocation,pre-synthesize,stop}`.

use crate::common::jsonfmt;
use regex::Regex;
use serde_json::{json, Value};
use std::fs;
use std::io::Read;
use std::path::Path;

fn read_stdin_json() -> Value {
    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_err() {
        return json!({});
    }
    serde_json::from_str(&buf).unwrap_or_else(|_| json!({}))
}

/// auto_wire_hook.py — `auto-wire <path_to_scene_file>` → JSON.
pub fn run_auto_wire(argv: &[String]) -> i32 {
    let file = match argv.first() {
        Some(f) => f.clone(),
        None => {
            println!("Usage: python3 auto_wire_hook.py <path_to_scene_file>");
            return 0;
        }
    };

    let result = if !Path::new(&file).exists() {
        json!({"error": format!("File not found: {}", file)})
    } else {
        let content = fs::read(&file)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default();
        let has_buttons =
            Regex::new(r#"(?i)type="(Button|TextureButton|CheckButton)"|<button|class=".*btn.*""#)
                .unwrap()
                .is_match(&content);
        let has_physics = Regex::new(
            r#"(?i)type="(RigidBody2D|RigidBody3D|CharacterBody2D|CharacterBody3D)"|rigidbody"#,
        )
        .unwrap()
        .is_match(&content);
        let has_particles = Regex::new(
            r#"(?i)type="(GPUParticles2D|GPUParticles3D|CPUParticles2D|CPUParticles3D)"|particle"#,
        )
        .unwrap()
        .is_match(&content);

        json!({
            "target_file": file,
            "interactive_buttons_found": has_buttons,
            "physics_bodies_found": has_physics,
            "particle_emitters_found": has_particles,
            "recommended_wiring": {
                "ui_click": if has_buttons { "Connect BaseButton.pressed to procedural click generator" } else { "None" },
                "physics_impact": if has_physics { "Connect RigidBody.body_entered to modal impact generator" } else { "None" },
                "weather_ambience": if has_particles { "Attach granular particle audio synthesizer" } else { "Standard Drone" }
            }
        })
    };
    println!("{}", jsonfmt::dumps_indent(&result, 2));
    0
}

/// post_synthesize_hook.py — `.wav` positional arg → audit JSON; else stdin
/// hook mode prints `{}`.
pub fn run_post_synthesize(argv: &[String]) -> i32 {
    if let Some(f) = argv.first() {
        if f.ends_with(".wav") {
            let report = audit_wav(f);
            println!("{}", jsonfmt::dumps_indent(&report, 2));
            return 0;
        }
    }
    let _ = read_stdin_json();
    println!("{}", jsonfmt::dumps(&json!({})));
    0
}

fn audit_wav(file_path: &str) -> Value {
    if !Path::new(file_path).exists() {
        return json!({"valid": false, "error": format!("File does not exist: {}", file_path)});
    }
    match audit_inner(file_path) {
        Ok(v) => v,
        Err(e) => json!({"valid": false, "error": e}),
    }
}

fn audit_inner(file_path: &str) -> Result<Value, String> {
    let data = fs::read(file_path).map_err(|e| e.to_string())?;
    if data.len() < 44 {
        return Err("file does not start with RIFF id".to_string());
    }
    // header fields (Python `wave` semantics: fmt tag, channels, sampwidth, rate, frames)
    let mut pos = 12usize;
    let mut fmt_tag = 1u16;
    let mut n_channels = 1usize;
    let mut framerate = 44100u32;
    let mut sampwidth = 2usize;
    let mut n_frames = 0usize;
    while pos + 8 <= data.len() {
        let id = &data[pos..pos + 4];
        let size = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
            as usize;
        let body = pos + 8;
        if id == b"fmt " && body + 16 <= data.len() {
            fmt_tag = u16::from_le_bytes([data[body], data[body + 1]]);
            n_channels = u16::from_le_bytes([data[body + 2], data[body + 3]]) as usize;
            framerate = u32::from_le_bytes([
                data[body + 4],
                data[body + 5],
                data[body + 6],
                data[body + 7],
            ]);
            let bits = u16::from_le_bytes([data[body + 14], data[body + 15]]) as usize;
            sampwidth = bits / 8;
        } else if id == b"data" {
            let frame_bytes = size.min(data.len().saturating_sub(body));
            let frame_w = (n_channels * sampwidth).max(1);
            n_frames = frame_bytes / frame_w;
        }
        pos = body + size + (size % 2);
    }
    if n_frames == 0 {
        return Ok(json!({"valid": false, "error": "WAV file contains 0 frames."}));
    }

    // Decode via shared loader, but audit needs raw channel samples — decode here.
    let raw = fs::read(file_path).map_err(|e| e.to_string())?;
    let mut data_off = 0usize;
    let mut data_len = 0usize;
    let mut p = 12usize;
    while p + 8 <= raw.len() {
        let id = &raw[p..p + 4];
        let size = u32::from_le_bytes([raw[p + 4], raw[p + 5], raw[p + 6], raw[p + 7]]) as usize;
        if id == b"data" {
            data_off = p + 8;
            data_len = size.min(raw.len().saturating_sub(p + 8));
        }
        p = p + 8 + size + (size % 2);
    }
    let bytes = &raw[data_off..data_off + data_len];

    let mut samples: Vec<f64> = Vec::new();
    if sampwidth == 2 {
        for c in bytes.chunks_exact(2) {
            samples.push(i16::from_le_bytes([c[0], c[1]]) as f64 / 32768.0);
        }
    } else if sampwidth == 4 {
        // Python unpacks "<N f" for any sampwidth-4 file (fmt_tag ignored).
        let _ = fmt_tag;
        for c in bytes.chunks_exact(4) {
            samples.push(f32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64);
        }
    } else {
        return Ok(json!({
            "valid": true,
            "note": format!("Unsupported sampwidth {} for detailed sample analysis", sampwidth)
        }));
    }

    let max_abs = samples.iter().fold(0.0f64, |a, &s| a.max(s.abs()));
    let peak_dbfs = 20.0 * max_abs.max(1e-6).log10();
    let dc_offset = samples.iter().sum::<f64>() / samples.len().max(1) as f64;
    let has_nan_inf = samples.iter().any(|s| s.is_nan() || s.is_infinite());
    let clipping_count = samples.iter().filter(|&&s| s.abs() >= 0.999).count();

    Ok(json!({
        "valid": !has_nan_inf && peak_dbfs <= 0.5,
        "peak_dbfs": crate::common::py::round_py(peak_dbfs, 2),
        "dc_offset": crate::common::py::round_py(dc_offset, 6),
        "has_nan_inf": has_nan_inf,
        "clipping_samples": clipping_count,
        "duration_sec": crate::common::py::round_py(n_frames as f64 / framerate as f64, 2),
        "sample_rate": framerate
    }))
}

/// pre_invocation_hook.py — reads stdin JSON, always emits injectSteps.
pub fn run_pre_invocation(_argv: &[String]) -> i32 {
    let _ = read_stdin_json();
    let response = json!({
        "injectSteps": [
            {
                "ephemeralMessage": "[Procedural Audio Pre-Hook Active] Remember to inspect the scene AST, calculate the continuous synesthesia vector, and use zero-allocation, lock-free synthesis architectures with hardware FTZ/DAZ denormal prevention."
            }
        ]
    });
    println!("{}", jsonfmt::dumps(&response));
    0
}

/// pre_synthesize_hook.py — validates bake command output paths.
pub fn run_pre_synthesize(_argv: &[String]) -> i32 {
    let input = read_stdin_json();
    let cmd = input
        .pointer("/toolCall/args/CommandLine")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let is_audio = [
        "aaa_audio_generator.py",
        "procedural_audio_generator.py",
        "motif_dna_generator.py",
    ]
    .iter()
    .any(|k| cmd.contains(k));
    if is_audio && cmd.contains("--out /") && !cmd.contains("/tmp/") && !cmd.contains("/Users/") {
        let response = json!({
            "decision": "deny",
            "reason": "Procedural Audio Pre-Hook: Audio bakes must output to /tmp/ or user workspace directory."
        });
        println!("{}", jsonfmt::dumps(&response));
        return 0;
    }
    println!("{}", jsonfmt::dumps(&json!({"decision": "allow"})));
    0
}

/// stop_check_hook.py — always allows.
pub fn run_stop_check(_argv: &[String]) -> i32 {
    let _ = read_stdin_json();
    println!("{}", jsonfmt::dumps(&json!({"decision": "allow"})));
    0
}
