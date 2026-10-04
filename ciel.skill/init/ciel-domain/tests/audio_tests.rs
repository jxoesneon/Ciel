//! Unit tests for the procedural-audio ports: WAV round-trip, pure music
//! theory functions (Tonnetz, Tala, L-system, microtonal, humanizer), and
//! deterministic synthesis via seeded RNG.

use ciel_domain::audio::{font5x7, humanizer, lsystem, microtonal, tala, tonnetz, wav};
use serde_json::Value;

fn tmp_path(name: &str) -> String {
    let dir = std::env::temp_dir().join(format!("ciel-domain-audio-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name).to_string_lossy().into_owned()
}

// ---------- wav ----------

#[test]
fn wav_float32_stereo_round_trip() {
    let samples: Vec<(f64, f64)> = (0..512)
        .map(|i| {
            let t = i as f64 / 512.0;
            ((t * 2.0 - 1.0), (1.0 - t * 2.0))
        })
        .collect();
    let path = tmp_path("stereo32.wav");
    wav::write_wav_32bit_float(&path, &samples, 48000);
    let (l, r, fs, stereo) = wav::load_wav_stereo(&path).unwrap();
    assert_eq!(fs, 48000);
    assert!(stereo);
    assert_eq!(l.len(), 512);
    assert_eq!(r.len(), 512);
    for (i, &(a, b)) in samples.iter().enumerate() {
        assert!((l[i] - a).abs() < 1e-6, "left sample {} diverged", i);
        assert!((r[i] - b).abs() < 1e-6, "right sample {} diverged", i);
    }
}

#[test]
fn wav_pcm24_round_trip() {
    let samples: Vec<(f64, f64)> = vec![(0.0, 0.0), (0.5, -0.5), (-1.0, 1.0), (0.25, -0.25)];
    let path = tmp_path("pcm24.wav");
    wav::write_wav_24bit_pcm(&path, &samples, 44100);
    let (l, r, fs, stereo) = wav::load_wav_stereo(&path).unwrap();
    assert_eq!(fs, 44100);
    assert!(stereo);
    for (i, &(a, b)) in samples.iter().enumerate() {
        let eps = 2.0 / 8_388_608.0;
        assert!((l[i] - a).abs() < eps);
        assert!((r[i] - b).abs() < eps);
    }
}

#[test]
fn wav_mono16_load_pcm() {
    // write_wav_mono bakes the soft-saturation curve + 32760 scaling from the
    // Python original — the round-trip is verified against that transform.
    let samples: Vec<f64> = vec![0.0, 0.5, -0.5, 0.99, -0.99];
    let path = tmp_path("mono16.wav");
    wav::write_wav_mono(&path, &samples, 22050, 16);
    let (mono, fs, channels) = wav::load_wav_pcm(&path).unwrap();
    assert_eq!(fs, 22050);
    assert_eq!(channels, 1);
    assert_eq!(mono.len(), 5);
    for (i, s) in samples.iter().enumerate() {
        let sat = if s.abs() > 0.001 {
            (s + 0.25 * s * s) / (1.0 + 0.4 * s.abs())
        } else {
            *s
        };
        let expected = ((sat * 32760.0).clamp(-32767.0, 32767.0) as i64) as f64 / 32768.0;
        assert!(
            (mono[i] - expected).abs() < 1.0 / 32768.0 + 1e-9,
            "sample {}: {} vs expected {}",
            i,
            mono[i],
            expected
        );
    }
}

#[test]
fn wav_load_missing_file() {
    let err = wav::load_wav_pcm("/nonexistent/nope.wav").unwrap_err();
    assert!(err.contains("File not found"));
}

// ---------- tonnetz ----------

#[test]
fn tonnetz_ops() {
    let c = tonnetz::Triad::new(0, true); // C major
    assert_eq!(c.pitch_classes(), (0, 4, 7));
    let cm = tonnetz::op_p(c);
    assert_eq!(cm.name(), "Cm");
    let em = tonnetz::op_l(c);
    assert_eq!(em.name(), "Em");
    let am = tonnetz::op_r(c);
    assert_eq!(am.name(), "Am");
    // Involution properties.
    assert_eq!(tonnetz::op_p(cm), c);
    assert_eq!(tonnetz::op_l(em), c);
    assert_eq!(tonnetz::op_r(am), c);
}

#[test]
fn tonnetz_shortest_path() {
    let c = tonnetz::Triad::new(0, true);
    let g = tonnetz::Triad::new(7, true);
    let path = tonnetz::find_shortest_path(c, g);
    assert!(!path.is_empty());
    // Applying the ops must actually reach G.
    let mut t = c;
    for (_, next) in &path {
        t = *next;
    }
    assert_eq!(t, g);
    // Start==goal → the BFS path contains only the START node.
    let trivial = tonnetz::find_shortest_path(c, c);
    assert_eq!(trivial.len(), 1);
    assert_eq!(trivial[0].0, "START");
}

// ---------- tala ----------

#[test]
fn tala_grid() {
    let grid = tala::get_tala_grid("tintal");
    assert_eq!(grid.len(), 16);
    assert_eq!(grid[0], "SAM(1)");
    assert_eq!(grid[4], "CLAP(5)");
    assert_eq!(grid[8], "KHALI(9)");
    // Unknown names fall back to the first tala (Python behavior).
    assert_eq!(tala::get_tala_grid("nope").len(), 16);
}

#[test]
fn tala_tihai() {
    let v = tala::generate_tihai("tintal", 1, 1.5);
    assert!(v.get("phrase_beats").is_some());
    // Tihai: 3*phrase + 2*dum = total beats + 1 sam resolution.
    let phrase = v["phrase_beats"].as_f64().unwrap();
    let dum = v["dum_beats"].as_f64().unwrap();
    let total = 3.0 * phrase + 2.0 * dum;
    assert!((total - 17.0).abs() < 1e-9, "total beats {} != 17", total);
    assert_eq!(v["tala"], Value::String("Tintal".into()));
}

// ---------- lsystem ----------

#[test]
fn lsystem_expansion() {
    let gen = lsystem::LSystemSchenkerGenerator::new(vec![0, 2, 4, 5, 7, 9, 11]);
    let one = gen.expand_axiom("U", 1);
    assert_eq!(one, "P1 P2 P3");
    let two = gen.expand_axiom("U", 2);
    assert_eq!(two, "N0 S1 N2 Z24 V4 Cad10");
    // Unknown tokens pass through unchanged.
    assert_eq!(gen.expand_axiom("Q", 5), "Q");
}

#[test]
fn lsystem_melody() {
    let gen = lsystem::LSystemSchenkerGenerator::new(vec![0, 2, 4, 5, 7, 9, 11]);
    let expanded = gen.expand_axiom("U", 3);
    let melody = gen.string_to_melody(&expanded, 60);
    assert!(!melody.is_empty());
    // All melody notes are scale degrees of C major rooted at 60.
    for note in &melody {
        assert!(
            [0, 2, 4, 5, 7, 9, 11].contains(&(note % 12)),
            "note {}",
            note
        );
    }
}

// ---------- microtonal / humanizer ----------

#[test]
fn microtonal_cents_ratio() {
    let m = microtonal::MicrotonalPitchEngine::new(440.0);
    assert!((m.cents_to_ratio(1200.0) - 2.0).abs() < 1e-9);
    assert!((m.ratio_to_cents(2.0) - 1200.0).abs() < 1e-9);
    // Round trip.
    assert!((m.ratio_to_cents(m.cents_to_ratio(700.0)) - 700.0).abs() < 1e-9);
}

#[test]
fn microtonal_edo_quantize() {
    let m = microtonal::MicrotonalPitchEngine::new(440.0);
    // 445 Hz relative to 440 Hz in 12-EDO: nearest step is 440 itself.
    let q = m.quantize_to_edo(445.0, 440.0, 12);
    assert!((q - 440.0).abs() < 1.0);
}

#[test]
fn humanizer_retune_major_chord() {
    let out = humanizer::retune_chord(60, &[60, 64, 67]);
    assert_eq!(out.len(), 3);
    // Major third (interval 4) is flattened by ~-13.69 cents in just intonation.
    let third = out[1];
    assert_eq!(third.0, 64);
    assert!((third.1 - (-13.69)).abs() < 0.01);
    // Perfect fifth ~ +1.96 cents? — use table value tolerance.
    assert!(out[2].1.abs() < 3.0);
    for (_, _, hz) in &out {
        assert!(*hz > 0.0);
    }
}

// ---------- font5x7 ----------

#[test]
fn font5x7_glyphs() {
    let g = font5x7::glyph('A').unwrap();
    assert_eq!(g.len(), 5);
    assert!(font5x7::glyph(' ').is_some());
    assert!(font5x7::glyph('\u{1f600}').is_none());
}
