//! Port of `skills/brag/scripts/analyze_music_cues.py` — music cue metadata
//! (tempo, beat grid, strong cues) → JSON + Markdown presets.
//!
//! Parity note: the Python source depends on librosa. This port implements
//! equivalent DSP natively: spectral-flux onset strength (log-power mel),
//! RMS, 30–180 Hz bass band energy, autocorrelation tempo estimation, and
//! librosa-style onset peak-picking. The JSON schema, keys, rounding, file
//! contract, and CLI are preserved exactly; numeric feature values may differ
//! slightly from librosa internals.

use crate::audio::wav;
use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use rustfft::num_complex::Complex;
use serde_json::{json, Map, Value};
use std::f64::consts::PI;
use std::fs;
use std::path::Path;
use std::process::Command;

const HOP_LENGTH: usize = 512;
const FRAME_LENGTH: usize = 2048;
const BASS_N_FFT: usize = 4096;
const BASS_MIN_HZ: f64 = 30.0;
const BASS_MAX_HZ: f64 = 180.0;

fn finite_round(v: f64, digits: i32) -> f64 {
    if !v.is_finite() {
        0.0
    } else {
        crate::common::py::round_py(v, digits)
    }
}

fn percentile98(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut s: Vec<f64> = values.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pos = 0.98 * (s.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = (lo + 1).min(s.len() - 1);
    s[lo] + (s[hi] - s[lo]) * (pos - lo as f64)
}

fn normalize(values: &[f64]) -> Vec<f64> {
    let clean: Vec<f64> = values
        .iter()
        .map(|&v| if v.is_finite() { v.max(0.0) } else { 0.0 })
        .collect();
    if clean.is_empty() {
        return clean;
    }
    let mut high = percentile98(&clean);
    if high <= 1e-12 {
        high = clean.iter().cloned().fold(0.0, f64::max);
    }
    if high <= 1e-12 {
        return vec![0.0; clean.len()];
    }
    clean.iter().map(|&v| (v / high).clamp(0.0, 1.0)).collect()
}

fn feature_at(feature: &[f64], frame: i64) -> f64 {
    if feature.is_empty() {
        return 0.0;
    }
    let idx = frame.max(0).min(feature.len() as i64 - 1) as usize;
    feature[idx]
}

fn local_contrast(onset_norm: &[f64], frame: usize, sr: u32) -> f64 {
    let radius = ((0.5 * sr as f64 / HOP_LENGTH as f64).round() as i64).max(1);
    let start = (frame as i64 - radius).max(0) as usize;
    let end = (frame as i64 + radius + 1).min(onset_norm.len() as i64) as usize;
    let local = &onset_norm[start.min(end)..end.max(start.min(end) + 1).min(onset_norm.len())];
    if local.is_empty() {
        return 0.0;
    }
    let mut sorted = local.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[sorted.len() / 2];
    (feature_at(onset_norm, frame as i64) - median).max(0.0)
}

// --- DSP core ---------------------------------------------------------------

fn hann_sym(n: usize) -> Vec<f64> {
    (0..n)
        .map(|k| 0.5 - 0.5 * (2.0 * PI * k as f64 / (n - 1) as f64).cos())
        .collect()
}

fn stft_power(
    y: &[f64],
    n_fft: usize,
    hop: usize,
    fft: &std::sync::Arc<dyn rustfft::Fft<f64>>,
) -> Vec<Vec<f64>> {
    // centered frames, zero-pad edges
    let window = hann_sym(n_fft);
    let pad = n_fft / 2;
    let n_frames = y.len() / hop + 1;
    let mut out = Vec::with_capacity(n_frames);
    let mut buf = vec![Complex::new(0.0, 0.0); n_fft];
    for f in 0..n_frames {
        let center = f * hop;
        for x in buf.iter_mut() {
            *x = Complex::new(0.0, 0.0);
        }
        for k in 0..n_fft {
            let gi = center as i64 + k as i64 - pad as i64;
            if gi >= 0 && (gi as usize) < y.len() {
                buf[k] = Complex::new(y[gi as usize] * window[k], 0.0);
            }
        }
        fft.process(&mut buf);
        out.push(buf[..n_fft / 2 + 1].iter().map(|c| c.norm_sqr()).collect());
    }
    out
}

fn hz_to_mel(f: f64) -> f64 {
    2595.0 * (1.0 + f / 700.0).log10()
}
fn mel_to_hz(m: f64) -> f64 {
    700.0 * (10f64.powf(m / 2595.0) - 1.0)
}

fn mel_filterbank(sr: u32, n_fft: usize, n_mels: usize) -> Vec<Vec<f64>> {
    let fmin = 0.0;
    let fmax = sr as f64 / 2.0;
    let m0 = hz_to_mel(fmin);
    let m1 = hz_to_mel(fmax);
    let pts: Vec<f64> = (0..n_mels + 2)
        .map(|i| mel_to_hz(m0 + (m1 - m0) * i as f64 / (n_mels + 1) as f64))
        .collect();
    let bins: Vec<usize> = pts
        .iter()
        .map(|&h| ((n_fft + 1) as f64 * h / sr as f64).floor() as usize)
        .collect();
    let n_freqs = n_fft / 2 + 1;
    let mut fb = vec![vec![0.0f64; n_freqs]; n_mels];
    for m in 1..=n_mels {
        let (a, b, c) = (bins[m - 1], bins[m], bins[m + 1]);
        for (k, v) in fb[m - 1]
            .iter_mut()
            .enumerate()
            .take(b.min(n_freqs))
            .skip(a)
        {
            if b != a {
                *v = (k - a) as f64 / (b - a) as f64;
            }
        }
        for (k, v) in fb[m - 1]
            .iter_mut()
            .enumerate()
            .take(c.min(n_freqs))
            .skip(b)
        {
            if c != b {
                *v = (c - k) as f64 / (c - b) as f64;
            }
        }
    }
    fb
}

/// librosa.onset.onset_strength equivalent: log-mel spectral flux.
fn onset_strength(y: &[f64], sr: u32) -> Vec<f64> {
    let n_fft = 2048usize;
    let mut planner = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(n_fft);
    let power = stft_power(y, n_fft, HOP_LENGTH, &fft);
    let n_mels = 128usize;
    let fb = mel_filterbank(sr, n_fft, n_mels);
    // log-power mel spec (ref = global max), then positive first-difference sum
    let mut mel: Vec<Vec<f64>> = power
        .iter()
        .map(|frame| {
            (0..n_mels)
                .map(|m| fb[m].iter().zip(frame.iter()).map(|(&w, &p)| w * p).sum())
                .collect()
        })
        .collect();
    let maxv = mel
        .iter()
        .flat_map(|f| f.iter().cloned())
        .fold(0.0f64, f64::max)
        .max(1e-10);
    for frame in mel.iter_mut() {
        for v in frame.iter_mut() {
            *v = 10.0 * (*v / maxv).max(1e-10).log10();
        }
    }
    let mut onset = vec![0.0f64; mel.len()];
    for t in 1..mel.len() {
        let s: f64 = mel[t]
            .iter()
            .zip(mel[t - 1].iter())
            .take(n_mels)
            .map(|(a, b)| (a - b).max(0.0))
            .sum();
        onset[t] = s;
    }
    onset
}

fn rms_feature(y: &[f64]) -> Vec<f64> {
    // librosa.feature.rms: frame_length 2048, hop 512, centered
    let pad = FRAME_LENGTH / 2;
    let n_frames = y.len() / HOP_LENGTH + 1;
    let mut out = Vec::with_capacity(n_frames);
    for f in 0..n_frames {
        let center = f * HOP_LENGTH;
        let mut acc = 0.0;
        let mut cnt = 0usize;
        for k in 0..FRAME_LENGTH {
            let gi = center as i64 + k as i64 - pad as i64;
            if gi >= 0 && (gi as usize) < y.len() {
                acc += y[gi as usize].powi(2);
                cnt += 1;
            }
        }
        out.push((acc / cnt.max(1) as f64).sqrt());
    }
    out
}

fn bass_energy(y: &[f64], sr: u32) -> Vec<f64> {
    let mut planner = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(BASS_N_FFT);
    let spec = stft_power(y, BASS_N_FFT, HOP_LENGTH, &fft);
    spec.iter()
        .map(|frame| {
            let mut acc = 0.0;
            let mut cnt = 0usize;
            for (bin, &p) in frame.iter().enumerate() {
                let f = bin as f64 * sr as f64 / BASS_N_FFT as f64;
                if (BASS_MIN_HZ..=BASS_MAX_HZ).contains(&f) {
                    acc += p.sqrt();
                    cnt += 1;
                }
            }
            if cnt > 0 {
                acc / cnt as f64
            } else {
                0.0
            }
        })
        .collect()
}

/// Autocorrelation tempo estimate in the 60–240 BPM window (log-normal
/// prior at 120 BPM approximating librosa's default aggregate).
fn estimate_tempo(onset: &[f64], sr: u32) -> (f64, Vec<usize>) {
    if onset.len() < 4 {
        return (120.0, Vec::new());
    }
    let hop_sec = HOP_LENGTH as f64 / sr as f64;
    let min_lag = (60.0 / 240.0 / hop_sec) as usize; // 240 BPM
    let max_lag = ((1.0 / hop_sec) as usize).min(onset.len() / 2); // 60.0/60.0 s/beat at 60 BPM
    let mean: f64 = onset.iter().sum::<f64>() / onset.len() as f64;
    let mut best_lag = 0usize;
    let mut best_score = f64::NEG_INFINITY;
    for lag in min_lag.max(1)..max_lag.max(min_lag + 2) {
        let mut c = 0.0;
        for i in 0..onset.len() - lag {
            c += (onset[i] - mean) * (onset[i + lag] - mean);
        }
        // weak 120 BPM log-normal prior
        let bpm = 60.0 / (lag as f64 * hop_sec);
        let prior = -0.5 * ((bpm / 120.0).log2().powi(2));
        if c / lag as f64 + prior > best_score {
            best_score = c / lag as f64 + prior;
            best_lag = lag;
        }
    }
    let tempo = if best_lag > 0 {
        60.0 / (best_lag as f64 * hop_sec)
    } else {
        120.0
    };
    // Beat grid: seeded greedy at tempo interval starting at first strong onset
    let mut beats = Vec::new();
    if best_lag > 0 {
        let start = onset
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
            .min(onset.len() / 4);
        let mut f = start;
        while f < onset.len() {
            // snap to local max within ±2 frames
            let lo = f.saturating_sub(2);
            let hi = (f + 3).min(onset.len());
            let snap = (lo..hi)
                .max_by(|&a, &b| onset[a].partial_cmp(&onset[b]).unwrap())
                .unwrap_or(f);
            if beats.last().copied() != Some(snap) {
                beats.push(snap);
            }
            f = snap + best_lag;
        }
    }
    (tempo, beats)
}

/// librosa.onset.onset_detect default peak-picking (backtrack=False).
fn onset_detect(onset: &[f64]) -> Vec<usize> {
    let pre_max = 3usize;
    let post_max = 3usize;
    let pre_avg = 3usize;
    let post_avg = 5usize;
    let delta = 0.07f64;
    let wait = 3usize;
    let n = onset.len();
    let mut peaks = Vec::new();
    let mut last: Option<usize> = None;
    for i in 0..n {
        let pm_lo = i.saturating_sub(pre_max);
        let pm_hi = (i + post_max + 1).min(n);
        let is_max = onset[i] >= (pm_lo..pm_hi).fold(0.0f64, |a, j| a.max(onset[j]));
        if !is_max || onset[i] <= 0.0 {
            continue;
        }
        let pa_lo = i.saturating_sub(pre_avg);
        let pa_hi = (i + post_avg + 1).min(n);
        let mean = (pa_lo..pa_hi).map(|j| onset[j]).sum::<f64>() / (pa_hi - pa_lo) as f64;
        if onset[i] < mean + delta {
            continue;
        }
        if let Some(l) = last {
            if i - l <= wait {
                if onset[i] > onset[l] {
                    peaks.pop();
                } else {
                    continue;
                }
            }
        }
        peaks.push(i);
        last = Some(i);
    }
    peaks
}

// --- Audio loading ----------------------------------------------------------

fn decode_audio(input: &Path, sr: u32) -> Result<(Vec<f64>, u32), String> {
    if input.extension().and_then(|e| e.to_str()) == Some("wav") {
        if let Ok((samples, fs, _ch)) = wav::load_wav_pcm(&input.to_string_lossy()) {
            if fs == sr {
                return Ok((samples, sr));
            }
            // fall through to ffmpeg resample for non-native sr
        }
    }
    let out = Command::new("ffmpeg")
        .args([
            "-i",
            &input.to_string_lossy(),
            "-vn",
            "-ac",
            "1",
            "-ar",
            &sr.to_string(),
            "-f",
            "s16le",
            "-acodec",
            "pcm_s16le",
            "-loglevel",
            "error",
            "pipe:1",
        ])
        .output()
        .map_err(|e| format!("ffmpeg failed to launch: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "ffmpeg error: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let samples: Vec<f64> = out
        .stdout
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]) as f64 / 32768.0)
        .collect();
    Ok((samples, sr))
}

// --- Driver -----------------------------------------------------------------

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-audio cues",
        argv,
        &[
            ArgSpec::positional("input").req(),
            ArgSpec::value("output-json", None, "output-json").req(),
            ArgSpec::value("output-md", None, "output-md").req(),
            ArgSpec::float("window-start", None, "window-start").def("0.0"),
            ArgSpec::float("window-duration", None, "window-duration").def("25.0"),
            ArgSpec::int("top-cues", None, "top-cues").def("10"),
            ArgSpec::int("sr", None, "sr").def("44100"),
        ],
    );

    let input = args.get_or("input", "");
    let out_json = args.get_or("output-json", "");
    let out_md = args.get_or("output-md", "");
    let window_start = args.float("window-start", 0.0);
    let window_duration = args.float("window-duration", 25.0);
    let top_cues = args.int("top-cues", 10).max(0) as usize;
    let sr = args.int("sr", 44100) as u32;

    let input_path = Path::new(&input);
    let (y, actual_sr) = match decode_audio(input_path, sr) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let duration = y.len() as f64 / actual_sr as f64;
    let window_end = (window_start + window_duration).min(duration);

    let onset_env = onset_strength(&y, actual_sr);
    let onset_norm = normalize(&onset_env);

    let local: Vec<f64> = (0..onset_norm.len())
        .map(|f| local_contrast(&onset_norm, f, actual_sr))
        .collect();
    let contrast_norm = normalize(&local);

    let rms = rms_feature(&y);
    let rms_norm = normalize(&rms);

    let bass = bass_energy(&y, actual_sr);
    let bass_norm = normalize(&bass);

    let (tempo, beat_frames) = estimate_tempo(&onset_env, actual_sr);
    let frames_to_time = |f: usize| -> f64 { f as f64 * HOP_LENGTH as f64 / actual_sr as f64 };

    let score_frame = |frame: usize| -> Map<String, Value> {
        let onset = feature_at(&onset_norm, frame as i64);
        let contrast = feature_at(&contrast_norm, frame as i64);
        let r = feature_at(&rms_norm, frame as i64);
        let b = feature_at(&bass_norm, frame as i64);
        let intensity = (0.45 * onset + 0.25 * contrast + 0.20 * r + 0.10 * b).clamp(0.0, 1.0);
        let mut m = Map::new();
        m.insert("intensity".into(), json!(intensity));
        m.insert("onsetStrength".into(), json!(onset));
        m.insert("localOnsetContrast".into(), json!(contrast));
        m.insert("rms".into(), json!(r));
        m.insert("bassEnergy".into(), json!(b));
        m
    };

    let mut beats: Vec<Value> = Vec::new();
    let mut cue_candidates: Vec<Value> = Vec::new();
    for &frame in &beat_frames {
        let time = frames_to_time(frame);
        let sc = score_frame(frame);
        let feat = json!({
            "onsetStrength": finite_round(sc["onsetStrength"].as_f64().unwrap_or(0.0), 4),
            "localOnsetContrast": finite_round(sc["localOnsetContrast"].as_f64().unwrap_or(0.0), 4),
            "rms": finite_round(sc["rms"].as_f64().unwrap_or(0.0), 4),
            "bassEnergy": finite_round(sc["bassEnergy"].as_f64().unwrap_or(0.0), 4)
        });
        let beat = json!({
            "time": finite_round(time, 4),
            "intensity": finite_round(sc["intensity"].as_f64().unwrap_or(0.0), 4),
            "features": feat
        });
        beats.push(beat.clone());
        let mut c = beat.as_object().unwrap().clone();
        c.insert("kind".into(), json!("strong_beat"));
        cue_candidates.push(Value::Object(c));
    }

    for frame in onset_detect(&onset_env) {
        let time = frames_to_time(frame);
        let sc = score_frame(frame);
        cue_candidates.push(json!({
            "time": finite_round(time, 4),
            "intensity": finite_round(sc["intensity"].as_f64().unwrap_or(0.0), 4),
            "kind": "onset_peak",
            "features": {
                "onsetStrength": finite_round(sc["onsetStrength"].as_f64().unwrap_or(0.0), 4),
                "localOnsetContrast": finite_round(sc["localOnsetContrast"].as_f64().unwrap_or(0.0), 4),
                "rms": finite_round(sc["rms"].as_f64().unwrap_or(0.0), 4),
                "bassEnergy": finite_round(sc["bassEnergy"].as_f64().unwrap_or(0.0), 4)
            }
        }));
    }

    // dedupe by intensity-desc, min gap 0.18s, then time-sort
    let mut sorted_by_strength = cue_candidates.clone();
    sorted_by_strength.sort_by(|a, b| {
        b["intensity"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["intensity"].as_f64().unwrap_or(0.0))
            .unwrap()
    });
    let mut accepted: Vec<Value> = Vec::new();
    for cue in sorted_by_strength {
        let t = cue["time"].as_f64().unwrap_or(0.0);
        if accepted
            .iter()
            .all(|e| (t - e["time"].as_f64().unwrap_or(0.0)).abs() >= 0.18)
        {
            accepted.push(cue);
        }
    }
    accepted.sort_by(|a, b| {
        a["time"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&b["time"].as_f64().unwrap_or(0.0))
            .unwrap()
    });

    let mut strong: Vec<Value> = accepted
        .into_iter()
        .filter(|c| c["intensity"].as_f64().unwrap_or(0.0) >= 0.45)
        .collect();
    strong.sort_by(|a, b| {
        b["intensity"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["intensity"].as_f64().unwrap_or(0.0))
            .unwrap()
    });
    strong.truncate(64);
    strong.sort_by(|a, b| {
        a["time"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&b["time"].as_f64().unwrap_or(0.0))
            .unwrap()
    });

    let stem = input_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let fname = input_path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| input.clone());

    let data = json!({
        "schemaVersion": 1,
        "source": {
            "filename": fname,
            "trackStem": stem,
        },
        "duration": finite_round(duration, 3),
        "tempo": finite_round(tempo, 2),
        "analysis": {
            "sampleRate": actual_sr,
            "hopLength": HOP_LENGTH,
            "windowStart": finite_round(window_start, 3),
            "windowDuration": finite_round(window_duration, 3),
            "windowEnd": finite_round(window_end, 3),
        },
        "scoring": {
            "intensityFormula": "0.45*onset_strength + 0.25*local_onset_contrast + 0.20*rms + 0.10*bass_energy",
            "features": ["onset_strength", "local_onset_contrast", "rms", "bass_energy"],
            "normalization": "Per-track robust normalization to 0-1 using the 98th percentile, then clamped.",
            "strongCueMinimumIntensity": 0.45,
            "strongCueMaxCount": 64,
        },
        "beats": beats,
        "strongCues": strong,
    });

    // Markdown sidecar
    let window_beats: Vec<&Value> = beats
        .iter()
        .filter(|b| {
            let t = b["time"].as_f64().unwrap_or(0.0);
            window_start <= t && t <= window_end
        })
        .collect();
    let mut window_cues: Vec<Value> = strong
        .iter()
        .filter(|c| {
            let t = c["time"].as_f64().unwrap_or(0.0);
            window_start <= t && t <= window_end
        })
        .cloned()
        .collect();
    window_cues.sort_by(|a, b| {
        b["intensity"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["intensity"].as_f64().unwrap_or(0.0))
            .unwrap()
    });
    window_cues.truncate(top_cues);
    let reveal: Vec<&Value> = window_cues.iter().take(5).collect();

    let compact_times = |items: &[&Value]| -> String {
        let subset: Vec<String> = items
            .iter()
            .take(48)
            .map(|b| format!("{:.2}", b["time"].as_f64().unwrap_or(0.0)))
            .collect();
        let mut text = subset.join(", ");
        if items.len() > 48 {
            text += &format!(", ... (+{} more)", items.len() - 48);
        }
        if text.is_empty() {
            "none".into()
        } else {
            text
        }
    };
    let fmt_cue = |c: &Value| -> String {
        format!(
            "{:.2}s ({:.2}, {})",
            c["time"].as_f64().unwrap_or(0.0),
            c["intensity"].as_f64().unwrap_or(0.0),
            c["kind"].as_str().unwrap_or("")
        )
    };

    let mut md = String::new();
    md.push_str(&format!("# Music Cues: {}\n\n", stem));
    md.push_str(&format!("- Track: `{}`\n", fname));
    md.push_str(&format!("- Duration: {:.2}s\n", duration));
    md.push_str(&format!("- Estimated tempo: {:.2} BPM\n", tempo));
    md.push_str(&format!(
        "- Planning window: {:.2}-{:.2}s\n\n",
        window_start, window_end
    ));
    md.push_str("## Useful Beat Grid\n\n");
    md.push_str(&compact_times(&window_beats));
    md.push_str("\n\n## Strong Cues In Window\n\n");
    if window_cues.is_empty() {
        md.push_str("- none\n");
    } else {
        for c in &window_cues {
            md.push_str(&format!("- {}\n", fmt_cue(c)));
        }
    }
    md.push_str("\n## Reveal Candidates\n\n");
    if reveal.is_empty() {
        md.push_str("- none\n");
    } else {
        for c in &reveal {
            md.push_str(&format!("- {}\n", fmt_cue(c)));
        }
    }
    md.push_str("\n## Use Policy\n\n");
    md.push_str("Use these as optional timing hints. Major reveals may move toward a nearby strong cue within about 0.15s; smaller entrances may align to nearby beats within about 0.10s. Ignore cues when they harm story, readability, or pacing.\n\n");

    if let Some(parent) = Path::new(&out_json).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }
    if let Some(parent) = Path::new(&out_md).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }
    if let Err(e) = fs::write(&out_json, jsonfmt::dumps_indent(&data, 2) + "\n") {
        eprintln!("Error writing {}: {}", out_json, e);
        return 1;
    }
    if let Err(e) = fs::write(&out_md, md) {
        eprintln!("Error writing {}: {}", out_md, e);
        return 1;
    }
    0
}
