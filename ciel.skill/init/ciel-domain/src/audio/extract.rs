//! Port of `skills/hyperframes-creative/scripts/extract-audio-data.py` —
//! per-frame RMS + logarithmic FFT band extraction via ffmpeg decode.
//! CLI: `ciel-audio extract <input> [-o audio-data.json] [--fps 30] [--bands 16]`

use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use rustfft::num_complex::Complex;
use serde_json::json;
use std::f64::consts::PI;
use std::fs;
use std::process::Command;

const SAMPLE_RATE: u32 = 44100;
const FFT_SIZE: usize = 4096;
const MIN_FREQ: f64 = 30.0;
const MAX_FREQ: f64 = 16000.0;

fn decode_audio(path: &str) -> Result<Vec<f64>, String> {
    let out = Command::new("ffmpeg")
        .args([
            "-i",
            path,
            "-vn",
            "-ac",
            "1",
            "-ar",
            "44100",
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
    Ok(out
        .stdout
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]) as f64 / 32768.0)
        .collect())
}

fn compute_band_edges(n_bands: usize) -> Vec<f64> {
    (0..=n_bands)
        .map(|i| MIN_FREQ * (MAX_FREQ / MIN_FREQ).powf(i as f64 / n_bands as f64))
        .collect()
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-audio extract",
        argv,
        &[
            ArgSpec::positional("input").req(),
            ArgSpec::value("output", Some('o'), "output").def("audio-data.json"),
            ArgSpec::int("fps", None, "fps").def("30"),
            ArgSpec::int("bands", None, "bands").def("16"),
        ],
    );

    let input = args.get_or("input", "");
    let output = args.get_or("output", "audio-data.json");
    let fps = args.int("fps", 30);
    let n_bands = args.int("bands", 16).max(1) as usize;
    if fps < 1 {
        eprintln!("ciel-audio extract: error: --fps must be at least 1");
        return 2;
    }

    eprintln!("Decoding audio from {}...", input);
    let samples = match decode_audio(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let duration = samples.len() as f64 / SAMPLE_RATE as f64;
    let frame_step = (SAMPLE_RATE / fps.max(1) as u32) as usize;
    let total_frames = (duration * fps as f64) as usize;

    eprintln!(
        "Duration: {:.1}s, {} frames at {}fps",
        duration, total_frames, fps
    );
    eprintln!(
        "FFT window: {} samples ({:.1} Hz/bin)",
        FFT_SIZE,
        SAMPLE_RATE as f64 / FFT_SIZE as f64
    );
    eprintln!(
        "Frequency range: {}-{} Hz, {} bands",
        MIN_FREQ as i64, MAX_FREQ as i64, n_bands
    );

    let hann: Vec<f64> = (0..FFT_SIZE)
        .map(|i| 0.5 - 0.5 * (2.0 * PI * i as f64 / (FFT_SIZE - 1) as f64).cos())
        .collect();
    let band_edges = compute_band_edges(n_bands);
    let freq_per_bin = SAMPLE_RATE as f64 / FFT_SIZE as f64;
    let n_bins = FFT_SIZE / 2 + 1;
    let half_fft = FFT_SIZE / 2;

    let mut planner = rustfft::FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

    let mut rms_values = vec![0.0f64; total_frames];
    let mut band_values = vec![vec![0.0f64; n_bands]; total_frames];

    for f in 0..total_frames {
        let rms_start = f * frame_step;
        let rms_end = (rms_start + frame_step).min(samples.len());
        if rms_end > rms_start {
            let slice = &samples[rms_start..rms_end];
            rms_values[f] = (slice.iter().map(|s| s * s).sum::<f64>() / slice.len() as f64).sqrt();
        }

        let center = rms_start + frame_step / 2;
        let win_start = center as i64 - half_fft as i64;
        let mut buf: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); FFT_SIZE];
        for k in 0..FFT_SIZE {
            let gi = win_start + k as i64;
            if gi >= 0 && (gi as usize) < samples.len() {
                buf[k] = Complex::new(samples[gi as usize] * hann[k], 0.0);
            }
        }
        fft.process(&mut buf);
        let mags: Vec<f64> = buf[..n_bins].iter().map(|c| c.norm()).collect();

        for b in 0..n_bands {
            let mut low_bin = ((band_edges[b] / freq_per_bin) as usize).min(n_bins - 1);
            let mut high_bin = ((band_edges[b + 1] / freq_per_bin) as usize).min(n_bins);
            if high_bin <= low_bin {
                high_bin = low_bin + 1;
            }
            low_bin = low_bin.min(n_bins - 1);
            high_bin = high_bin.min(n_bins);
            band_values[f][b] = mags[low_bin..high_bin].iter().cloned().fold(0.0, f64::max);
        }
    }

    // normalize
    let peak_rms = rms_values.iter().cloned().fold(0.0f64, f64::max);
    if peak_rms > 0.0 {
        for v in rms_values.iter_mut() {
            *v /= peak_rms;
        }
    }
    for b in 0..n_bands {
        let peak = band_values.iter().map(|f| f[b]).fold(0.0f64, f64::max);
        let peak = if peak == 0.0 { 1.0 } else { peak };
        for f in band_values.iter_mut() {
            f[b] /= peak;
        }
    }

    let frames: Vec<serde_json::Value> = (0..total_frames)
        .map(|f| {
            json!({
                "time": crate::common::py::round_py(f as f64 / fps as f64, 4),
                "rms": crate::common::py::round_py(rms_values[f], 4),
                "bands": band_values[f]
                    .iter()
                    .map(|&b| crate::common::py::round_py(b, 4))
                    .collect::<Vec<f64>>()
            })
        })
        .collect();

    let data = json!({
        "duration": crate::common::py::round_py(duration, 4),
        "fps": fps,
        "bands": n_bands,
        "totalFrames": total_frames,
        "frames": frames,
    });

    if let Err(e) = fs::write(&output, jsonfmt::dumps(&data)) {
        eprintln!("Error writing {}: {}", output, e);
        return 1;
    }
    eprintln!(
        "Wrote {} ({} frames, {} bands)",
        output, total_frames, n_bands
    );
    0
}
