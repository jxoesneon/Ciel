//! Port of `skills/procedural-audio/scripts/audio_manifest_extractor.py` —
//! 3-tier AudioStructuralManifest JSON from a WAV file.
//! Usage parity: `ciel-audio manifest <input.wav>` → JSON on stdout.

use crate::audio::wav;
use crate::common::jsonfmt;
use crate::common::py::round_py;
use serde_json::{json, Value};

fn generate_sparkline(arr: &[f64], length: usize) -> String {
    let blocks: Vec<char> = " ▂▃▄▅▆▇█".chars().collect();
    if arr.is_empty() {
        return " ".repeat(length);
    }
    let min_v = arr.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_v = arr.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if max_v == min_v {
        return blocks[3].to_string().repeat(length);
    }
    let mut out = String::new();
    for i in 0..length {
        let idx = ((i as f64 / length as f64) * (arr.len() - 1) as f64) as usize;
        let v = arr[idx];
        let norm = ((v - min_v) / (max_v - min_v)).clamp(0.0, 1.0);
        out.push(blocks[(norm * 7.0) as usize]);
    }
    out
}

pub fn analyze_audio(file_path: &str) -> Result<Value, String> {
    let (samples, fs, n_channels) = wav::load_wav_pcm(file_path)?;
    if samples.is_empty() {
        return Ok(json!({"error": "Empty audio file"}));
    }

    let duration = samples.len() as f64 / fs as f64;

    // 1. Dynamics & Energy
    let peak_amp = samples.iter().fold(0.0f64, |a, &s| a.max(s.abs()));
    let peak_dbfs = 20.0 * peak_amp.max(1e-6).log10();
    let rms = (samples.iter().map(|s| s * s).sum::<f64>() / samples.len() as f64).sqrt();
    let rms_dbfs = 20.0 * rms.max(1e-6).log10();
    let crest_factor = round_py(peak_dbfs - rms_dbfs, 2);
    let integrated_lufs = round_py(rms_dbfs - 0.691, 2);

    // 2. Chunk-wise RMS sparkline (100ms chunks)
    let chunk_size = ((fs as f64 * 0.10) as usize).max(1);
    let mut chunk_rms = Vec::new();
    for chunk in samples.chunks(chunk_size) {
        let r = (chunk.iter().map(|x| x * x).sum::<f64>() / chunk.len() as f64).sqrt();
        chunk_rms.push(20.0 * r.max(1e-5).log10());
    }
    let spark_loudness = generate_sparkline(&chunk_rms, 12);

    // 3. ZCR
    let zc_count = (1..samples.len())
        .filter(|&i| (samples[i] >= 0.0) != (samples[i - 1] >= 0.0))
        .count();
    let zcr = round_py(zc_count as f64 / samples.len() as f64, 4);

    // 4. Spectral tilt approximation
    let lf_energy: f64 = samples.iter().step_by(2).map(|s| s * s).sum();
    let hf_energy: f64 = (1..samples.len())
        .map(|i| (samples[i] - samples[i - 1]).powi(2))
        .sum();
    let spectral_tilt_ratio = hf_energy / (lf_energy + 1e-8);
    let est_centroid_hz = round_py(
        (400.0 + spectral_tilt_ratio * 1200.0).clamp(120.0, 8000.0),
        1,
    );

    // 5. Autocorrelation pitch tracking
    let corr_len = samples.len().min((fs as f64 * 0.5) as usize);
    let corr_samples = &samples[..corr_len];
    let min_lag = (fs as f64 / 800.0) as usize;
    let max_lag = (fs as f64 / 60.0) as usize;

    let mut best_lag = 0usize;
    let mut best_corr = -1.0f64;
    let zero_lag_energy = corr_samples.iter().map(|x| x * x).sum::<f64>() + 1e-9;

    let lag_end = max_lag.min(corr_samples.len() / 2);
    let mut lag = min_lag;
    while lag < lag_end {
        let mut c = 0.0;
        for i in 0..(corr_samples.len() - lag) {
            c += corr_samples[i] * corr_samples[i + lag];
        }
        let norm_c = c / zero_lag_energy;
        if norm_c > best_corr {
            best_corr = norm_c;
            best_lag = lag;
        }
        lag += 2;
    }

    let (f0, note_str, modality, hnr_db, flatness) = if best_corr > 0.40 && best_lag > 0 {
        let f0 = round_py(fs as f64 / best_lag as f64, 1);
        let midi_val = 69.0 + 12.0 * (f0 / 440.0).log2();
        let notes = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let rounded = midi_val.round() as i64;
        let cents = ((midi_val - rounded as f64) * 100.0).round() as i64;
        let note_str = format!(
            "{}{}{}{}c",
            notes[rounded.rem_euclid(12) as usize],
            rounded.div_euclid(12) - 1,
            if cents >= 0 { "+" } else { "" },
            cents
        );
        let hnr = round_py(
            10.0 * (best_corr / (1.0 - best_corr.min(0.99))).max(1e-2).log10(),
            1,
        );
        let flat = round_py(0.015 + (1.0 - best_corr) * 0.1, 4);
        (f0, note_str, "monophonic_tonal", hnr, flat)
    } else {
        (
            0.0,
            "Aperiodic / Unpitched".to_string(),
            "percussive_noise",
            4.2,
            0.35,
        )
    };

    // 6. Semantic tags
    let mut tags: Vec<&str> = Vec::new();
    if est_centroid_hz < 500.0 {
        tags.push("Sub/Deep Bass");
    } else if est_centroid_hz < 1500.0 {
        tags.push("Warm / Body / Dark");
    } else if est_centroid_hz < 3500.0 {
        tags.push("Present / Mid-Focused");
    } else {
        tags.push("Bright / Crisp / Airy");
    }
    if flatness < 0.05 {
        tags.push("Resonant / Pure Tonal");
    } else if flatness < 0.20 {
        tags.push("Harmonic / Textured");
    } else {
        tags.push("Aperiodic / Noise / Transient");
    }
    if crest_factor > 14.0 {
        tags.push("Impulsive Transient Attack");
    } else if crest_factor > 8.0 {
        tags.push("Articulated Dynamics");
    } else {
        tags.push("Sustained Pad / Compressed");
    }

    let dense_caption = format!(
        "An acoustic signal characterized by {}, with dynamic crest factor of {} dBFS and an estimated room reverberation tail.",
        tags.join(", "),
        crate::common::py::py_num(crest_factor)
    );

    Ok(json!({
        "audio_metadata": {
            "duration_seconds": round_py(duration, 3),
            "sample_rate": fs,
            "channels": n_channels,
            "file_path": file_path
        },
        "tier_1_semantic_context": {
            "dense_caption": dense_caption,
            "audioset_ontology_tags": [
                {"label": "Sound Effect / Musical Event", "id": "/m/04rlf", "confidence": 0.92},
                {"label": tags[0], "id": "/m/02mscn", "confidence": 0.88}
            ],
            "environmental_acoustics": {
                "estimated_rt60_seconds": 0.85,
                "direct_to_reverberant_ratio_db": 1.5,
                "acoustic_space": "Studio Acoustic Enclosure"
            }
        },
        "tier_2_mir_physical_dsp": {
            "loudness_and_dynamics": {
                "integrated_lufs": integrated_lufs,
                "loudness_range_lu": 4.2,
                "true_peak_dbtp": round_py(peak_dbfs, 2),
                "crest_factor_db": crest_factor,
                "loudness_contour_sparkline_lufs": spark_loudness
            },
            "spectral_timbre": {
                "spectral_centroid_hz": est_centroid_hz,
                "spectral_spread_hz": 1200.0,
                "spectral_flatness": flatness,
                "spectral_rolloff_85_hz": round_py(est_centroid_hz * 1.8, 1),
                "hnr_db": hnr_db,
                "inharmonicity": 0.008,
                "zero_crossing_rate": zcr,
                "attack_time_ms": if crest_factor > 12.0 { 12.0 } else { 45.0 },
                "formants_f1_f4_hz": [500.0, 1500.0, 2500.0, 3500.0],
                "semantic_tags": tags
            },
            "pitch_profile": {
                "modality": modality,
                "estimated_key": if f0 > 0.0 { "D minor" } else { "Unpitched" },
                "pitch_f0_median_hz": f0,
                "note_range": note_str,
                "vibrato": {
                    "presence": f0 > 0.0,
                    "rate_hz": if f0 > 0.0 { 5.5 } else { 0.0 },
                    "depth_cents": if f0 > 0.0 { 30.0 } else { 0.0 }
                },
                "intonation_drift_cents_per_sec": -0.1,
                "pitch_contour_sparkline": if f0 > 0.0 { "▂▃▅▆▇▇▆▅▄▅▇█" } else { "            " }
            },
            "rhythmic_profile": {
                "estimated_bpm": 120.0,
                "bpm_confidence": 0.90,
                "meter": "4/4",
                "swing_ratio_pct": 58.5,
                "groove_jitter_ms": 4.8,
                "mean_grid_offset_ms": 3.0,
                "rhythmic_feel": "Human pocket with light swing"
            }
        },
        "tier_3_symbolic_music": {
            "detected_key": if f0 > 0.0 { "D minor" } else { "N/A" },
            "tempo_bpm": 120.0,
            "time_signature": "4/4",
            "abc_notation": "X:1\nT:Extracted Take\nM:4/4\nL:1/8\nQ:1/4=120\nK:Dmin\n[V:1] D2 F2 A2 d2 | f4 d4 |]",
            "midi_events": [
                {"onset_s": 0.0, "pitch_midi": 62, "note": "D4", "velocity": 90, "duration_s": 0.5},
                {"onset_s": 0.5, "pitch_midi": 65, "note": "F4", "velocity": 95, "duration_s": 0.5}
            ]
        }
    }))
}

pub fn run(argv: &[String]) -> i32 {
    // sys.argv parity: positional wav path; no argparse in the Python source.
    let file = match argv.first() {
        Some(f) if !f.starts_with('-') => f.clone(),
        _ => {
            println!("Usage: python3 audio_manifest_extractor.py <input.wav>");
            return 0;
        }
    };
    match analyze_audio(&file) {
        Ok(v) => {
            println!("{}", jsonfmt::dumps_indent(&v, 2));
            0
        }
        Err(e) => {
            // Python raises FileNotFoundError → traceback, exit 1.
            eprintln!("{}", e);
            1
        }
    }
}
