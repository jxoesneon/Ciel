//! Port of `skills/procedural-audio/scripts/audiogram_generator.py` —
//! six-panel VLM diagnostic audiogram rendered directly to PNG (the Python
//! uses matplotlib; this port rasterises the same panels natively).
//! Usage parity: `ciel-audio audiogram <input.wav> [output.png]`.

use crate::audio::font5x7;
use crate::audio::wav;
use crate::common::py;
use rustfft::num_complex::Complex;
use rustfft::Fft;
use std::f64::consts::PI;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// RGB canvas primitives
// ---------------------------------------------------------------------------

type Rgb = (u8, u8, u8);

fn hex(s: &str) -> Rgb {
    let s = s.trim_start_matches('#');
    let v = |a: usize| u8::from_str_radix(&s[a..a + 2], 16).unwrap_or(0);
    (v(0), v(2), v(4))
}

struct Canvas {
    w: usize,
    h: usize,
    px: Vec<u8>,
}

impl Canvas {
    fn new(w: usize, h: usize, bg: Rgb) -> Self {
        let mut px = vec![0u8; w * h * 3];
        for i in 0..w * h {
            px[i * 3] = bg.0;
            px[i * 3 + 1] = bg.1;
            px[i * 3 + 2] = bg.2;
        }
        Canvas { w, h, px }
    }
    fn set(&mut self, x: i64, y: i64, c: Rgb) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            let i = (y as usize * self.w + x as usize) * 3;
            self.px[i] = c.0;
            self.px[i + 1] = c.1;
            self.px[i + 2] = c.2;
        }
    }
    fn fill_rect(&mut self, x: i64, y: i64, w: i64, h: i64, c: Rgb) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }
    fn rect(&mut self, x: i64, y: i64, w: i64, h: i64, c: Rgb) {
        for xx in x..x + w {
            self.set(xx, y, c);
            self.set(xx, y + h - 1, c);
        }
        for yy in y..y + h {
            self.set(x, yy, c);
            self.set(x + w - 1, yy, c);
        }
    }
    fn hline(&mut self, x0: i64, x1: i64, y: i64, c: Rgb) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.set(x, y, c);
        }
    }
    fn vline(&mut self, x: i64, y0: i64, y1: i64, c: Rgb) {
        for y in y0.min(y1)..=y0.max(y1) {
            self.set(x, y, c);
        }
    }
    fn circle(&mut self, cx: i64, cy: i64, r: i64, c: Rgb) {
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y <= r * r {
                    self.set(cx + x, cy + y, c);
                }
            }
        }
    }
    /// Draw text (uppercased) at 5x7 px per glyph scaled by `scale`.
    fn text(&mut self, x: i64, y: i64, s: &str, c: Rgb, scale: i64) {
        let mut cx = x;
        for ch in s.to_uppercase().chars() {
            if ch == '\n' {
                cx = x;
                continue;
            }
            if let Some(cols) = font5x7::glyph(ch) {
                for (col_i, col) in cols.iter().enumerate() {
                    for row in 0..7 {
                        if col & (1 << row) != 0 {
                            self.fill_rect(
                                cx + col_i as i64 * scale,
                                y + row as i64 * scale,
                                scale,
                                scale,
                                c,
                            );
                        }
                    }
                }
            }
            cx += 6 * scale;
        }
    }
    fn save_png(&self, path: &str) -> Result<(), String> {
        let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
        let w = std::io::BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, self.w as u32, self.h as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(&self.px).map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Colormaps (matplotlib magma / inferno / viridis approximations)
// ---------------------------------------------------------------------------

fn lerp(a: Rgb, b: Rgb, t: f64) -> Rgb {
    (
        (a.0 as f64 + (b.0 as f64 - a.0 as f64) * t) as u8,
        (a.1 as f64 + (b.1 as f64 - a.1 as f64) * t) as u8,
        (a.2 as f64 + (b.2 as f64 - a.2 as f64) * t) as u8,
    )
}

fn cmap(stops: &[Rgb], v: f64) -> Rgb {
    let v = v.clamp(0.0, 1.0);
    let pos = v * (stops.len() - 1) as f64;
    let i = (pos as usize).min(stops.len() - 2);
    lerp(stops[i], stops[i + 1], pos - i as f64)
}

fn magma(v: f64) -> Rgb {
    cmap(
        &[
            hex("000004"),
            hex("1C1044"),
            hex("4F127B"),
            hex("812581"),
            hex("B5367A"),
            hex("E55C30"),
            hex("FBA40A"),
            hex("FCFDBF"),
        ],
        v,
    )
}
fn inferno(v: f64) -> Rgb {
    cmap(
        &[
            hex("000004"),
            hex("1F0C48"),
            hex("560F6D"),
            hex("8A226A"),
            hex("BA3655"),
            hex("E25633"),
            hex("F98C0A"),
            hex("FCFFA4"),
        ],
        v,
    )
}
fn viridis(v: f64) -> Rgb {
    cmap(
        &[
            hex("440154"),
            hex("482878"),
            hex("3F4788"),
            hex("31688E"),
            hex("26828E"),
            hex("1F9E89"),
            hex("35B779"),
            hex("6ECE58"),
            hex("B5DE2B"),
            hex("FDE725"),
        ],
        v,
    )
}

// ---------------------------------------------------------------------------
// DSP helpers
// ---------------------------------------------------------------------------

fn hann(n: usize) -> Vec<f64> {
    // scipy.signal.windows.hann(n, sym=False): w[k] = 0.5-0.5*cos(2*pi*k/n)
    (0..n)
        .map(|k| 0.5 - 0.5 * (2.0 * PI * k as f64 / n as f64).cos())
        .collect()
}

/// STFT magnitude frames matching scipy.signal.stft(boundary='zeros').
fn stft_mag(signal: &[f64], n_fft: usize, hop: usize, fft: &Arc<dyn Fft<f64>>) -> Vec<Vec<f64>> {
    let window = hann(n_fft);
    let pad = n_fft / 2;
    let padded_len = signal.len() + 2 * pad;
    let n_frames = (padded_len / hop).max(1);
    let n_bins = n_fft / 2 + 1;
    let mut out = Vec::with_capacity(n_frames);
    let mut buf: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); n_fft];
    for f in 0..n_frames {
        let start = f * hop;
        for x in buf.iter_mut() {
            *x = Complex::new(0.0, 0.0);
        }
        for k in 0..n_fft {
            let gi = start + k;
            if gi >= pad && gi - pad < signal.len() {
                buf[k] = Complex::new(signal[gi - pad] * window[k], 0.0);
            }
        }
        fft.process(&mut buf);
        out.push(buf[..n_bins].iter().map(|c| c.norm()).collect());
    }
    out
}

fn hz_to_mel(f: f64) -> f64 {
    2595.0 * (1.0 + f / 700.0).log10()
}
fn mel_to_hz(m: f64) -> f64 {
    700.0 * (10f64.powf(m / 2595.0) - 1.0)
}

fn create_mel_filterbank(
    sr: u32,
    n_fft: usize,
    n_mels: usize,
    fmin: f64,
    fmax: f64,
) -> Vec<Vec<f64>> {
    let mel_min = hz_to_mel(fmin);
    let mel_max = hz_to_mel(fmax);
    let mel_points: Vec<f64> = (0..n_mels + 2)
        .map(|i| mel_min + (mel_max - mel_min) * i as f64 / (n_mels + 1) as f64)
        .collect();
    let hz_points: Vec<f64> = mel_points.iter().map(|&m| mel_to_hz(m)).collect();
    let bin_points: Vec<i64> = hz_points
        .iter()
        .map(|&h| ((n_fft + 1) as f64 * h / sr as f64).floor() as i64)
        .collect();
    let n_freqs = n_fft / 2 + 1;
    let mut fb = vec![vec![0.0f64; n_freqs]; n_mels];
    for m in 1..=n_mels {
        let f_m_minus = bin_points[m - 1];
        let f_m = bin_points[m];
        let f_m_plus = bin_points[m + 1];
        for k in f_m_minus..f_m {
            if f_m != f_m_minus && k >= 0 && (k as usize) < n_freqs {
                fb[m - 1][k as usize] = (k - f_m_minus) as f64 / (f_m - f_m_minus) as f64;
            }
        }
        for k in f_m..f_m_plus {
            if f_m_plus != f_m && k >= 0 && (k as usize) < n_freqs {
                fb[m - 1][k as usize] = (f_m_plus - k) as f64 / (f_m_plus - f_m) as f64;
            }
        }
    }
    // enorm normalization (slaney-style area norm as in the Python source)
    for m in 0..n_mels {
        let enorm = 2.0 / (hz_points[m + 2] - hz_points[m]);
        for v in fb[m].iter_mut() {
            *v *= enorm;
        }
    }
    fb
}

// ---------------------------------------------------------------------------
// Main entry
// ---------------------------------------------------------------------------

pub fn run(argv: &[String]) -> i32 {
    let wav_in = match argv.first() {
        Some(f) if !f.starts_with('-') => f.clone(),
        _ => {
            println!("Usage: python3 audiogram_generator.py <input.wav> [output.png]");
            return 0;
        }
    };
    let img_out = argv
        .get(1)
        .cloned()
        .unwrap_or_else(|| "diagnostic_audiogram.png".to_string());

    match generate_vlm_audiogram(&wav_in, &img_out) {
        Ok(()) => {
            println!("[*] VLM Audiogram generated at: {}", img_out);
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    }
}

fn generate_vlm_audiogram(wav_path: &str, output_image_path: &str) -> Result<(), String> {
    let (left, right, sr, is_stereo) = wav::load_wav_stereo(wav_path)?;
    let n = left.len();
    if n == 0 {
        return Err("Empty audio file".to_string());
    }
    let mono: Vec<f64> = left
        .iter()
        .zip(right.iter())
        .map(|(&l, &r)| (l + r) / 2.0)
        .collect();
    let duration = n as f64 / sr as f64;

    // 1. RMS envelope (50ms moving average, mode='same')
    let rms_win = ((sr as f64 * 0.050) as usize).max(1);
    let square: Vec<f64> = mono.iter().map(|x| x * x).collect();
    let mut rms_env = vec![0.0f64; n];
    let half = rms_win / 2;
    let mut acc = 0.0f64;
    for i in 0..n {
        // causal sliding window approximating mode='same'
        acc += square[i];
        if i >= rms_win {
            acc -= square[i - rms_win];
        }
        let idx = i + half;
        if idx < n {
            let _ = idx;
        }
        let len = i.min(rms_win).max(1);
        rms_env[i] = (acc / rms_win as f64).sqrt();
        let _ = len;
    }
    // clip points
    let clip_idx: Vec<usize> = mono
        .iter()
        .enumerate()
        .filter(|(_, &v)| v.abs() >= 0.994)
        .map(|(i, _)| i)
        .collect();

    // 2. Mel spectrogram 2048/512
    let n_fft = 2048usize;
    let hop = 512usize;
    let mut planner = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(n_fft);
    let mag_spec = stft_mag(&mono, n_fft, hop, &fft);
    let n_mels = 128usize;
    let mel_fb = create_mel_filterbank(sr, n_fft, n_mels, 20.0, sr as f64 / 2.0);
    let mel_spec: Vec<Vec<f64>> = mag_spec
        .iter()
        .map(|frame| {
            let power: Vec<f64> = frame.iter().map(|&m| m * m / n_fft as f64).collect();
            (0..n_mels)
                .map(|m| {
                    mel_fb[m]
                        .iter()
                        .zip(power.iter())
                        .map(|(&w, &p)| w * p)
                        .sum::<f64>()
                })
                .collect()
        })
        .collect();
    let ref_power = mel_spec
        .iter()
        .flat_map(|f| f.iter().cloned())
        .fold(0.0f64, f64::max)
        + 1e-8;
    let mel_db: Vec<Vec<f64>> = mel_spec
        .iter()
        .map(|f| {
            f.iter()
                .map(|&p| (10.0 * (p.max(1e-8) / ref_power).log10()).clamp(-80.0, 0.0))
                .collect()
        })
        .collect();

    // 3. Chromagram 4096/1024
    let chroma_fft_len = 4096usize;
    let chroma_hop = 1024usize;
    let chroma_fft = planner.plan_fft_forward(chroma_fft_len);
    let chroma_spec = stft_mag(&mono, chroma_fft_len, chroma_hop, &chroma_fft);
    let n_chroma_frames = chroma_spec.len();
    let mut chroma = vec![vec![0.0f64; n_chroma_frames]; 12];
    for (f_idx, frame) in chroma_spec.iter().enumerate() {
        for (bin, &mag) in frame.iter().enumerate() {
            let freq = bin as f64 * sr as f64 / chroma_fft_len as f64;
            if freq > 27.5 {
                let midi = 69.0 + 12.0 * (freq / 440.0).log2();
                let pc = midi.round() as i64 % 12;
                chroma[pc.rem_euclid(12) as usize][f_idx] += mag * mag;
            }
        }
    }
    for f in 0..n_chroma_frames {
        let norm = chroma.iter().map(|row| row[f] * row[f]).sum::<f64>().sqrt() + 1e-8;
        for row in chroma.iter_mut() {
            row[f] /= norm;
        }
    }

    // 4. SSM from chroma
    let mut ssm = vec![vec![0.0f64; n_chroma_frames]; n_chroma_frames];
    for a in 0..n_chroma_frames {
        for b in 0..n_chroma_frames {
            let dot: f64 = (0..12).map(|r| chroma[r][a] * chroma[r][b]).sum();
            let na: f64 = (0..12).map(|r| chroma[r][a].powi(2)).sum::<f64>().sqrt() + 1e-8;
            let nb: f64 = (0..12).map(|r| chroma[r][b].powi(2)).sum::<f64>().sqrt() + 1e-8;
            ssm[a][b] = (dot / (na * nb)).clamp(0.0, 1.0);
        }
    }

    // 5. Stereo metrics
    let mut sum_lr = 0.0;
    let mut sum_l2 = 0.0;
    let mut sum_r2 = 0.0;
    for i in 0..n {
        sum_lr += left[i] * right[i];
        sum_l2 += left[i] * left[i];
        sum_r2 += right[i] * right[i];
    }
    let phase_corr = sum_lr / ((sum_l2 * sum_r2).sqrt() + 1e-8);

    let max_peak_db = 20.0 * (mono.iter().fold(0.0f64, |a, &v| a.max(v.abs())) + 1e-6).log10();
    let integrated_rms = 20.0 * ((square.iter().sum::<f64>() / n as f64).sqrt() + 1e-6).log10();
    let crest_factor = max_peak_db - integrated_rms;

    // ------------------------------------------------------------------
    // Render 2400x1600 canvas (matplotlib 24x16 @ ~100dpi equivalent)
    // ------------------------------------------------------------------
    const W: usize = 2400;
    const H: usize = 1600;
    let bg = hex("0E1117");
    let panel_bg = hex("161B22");
    let grid = hex("2E3440");
    let text_c = hex("E2E8F0");
    let mut cv = Canvas::new(W, H, bg);

    let margin = 60i64;
    let col2x = (W as i64) * 8 / 105 * 10; // col boundary helper below
    let _ = col2x;
    // width ratios 4:4:2.5 → columns
    let total_w = (W as i64) - 2 * margin;
    let hud_w = total_w * 25 / 105;
    let main_w = total_w - hud_w - 20;
    let main_x0 = margin;
    let main_x1 = main_x0 + main_w;
    let hud_x0 = main_x1 + 20;
    let hud_x1 = hud_x0 + hud_w;

    let total_h = (H as i64) - 2 * margin;
    let rh = [120i64, 220, 140, 180]; // height ratio parts
    let rh_sum: i64 = rh.iter().sum();
    let mut row_y = [0i64; 4];
    let mut acc = margin;
    for i in 0..4 {
        row_y[i] = acc;
        acc += total_h * rh[i] / rh_sum + 14;
    }
    let panel_h = |i: usize| total_h * rh[i] / rh_sum;

    // ---- Panel 1: Waveform ----
    {
        let (x, y, w, h) = (main_x0, row_y[0], main_w, panel_h(0));
        cv.fill_rect(x, y, w, h, panel_bg);
        cv.rect(x, y, w, h, grid);
        cv.text(
            x + 8,
            y + 6,
            &format!(
                "PANEL 1: WAVEFORM ENVELOPE, RMS ENERGY & CLIPPING DIAGNOSTICS (DURATION: {:.2}S)",
                duration
            ),
            hex("38BDF8"),
            2,
        );
        let plot_y0 = y + 26;
        let plot_h = h - 34;
        let mid_y = plot_y0 + plot_h / 2;
        let amp = (plot_h as f64 / 2.0) * 0.95;
        // gridlines
        for gy in 0..=4 {
            let yy = plot_y0 + plot_h * gy / 4;
            for xx in (x..x + w).step_by(8) {
                cv.set(xx, yy, grid);
            }
        }
        // waveform polyline: min/max per column
        for xx in 0..w {
            let s0 = (xx as f64 / w as f64 * n as f64) as usize;
            let s1 = (((xx + 1) as f64 / w as f64 * n as f64) as usize)
                .max(s0 + 1)
                .min(n);
            let mut lo = 1.0f64;
            let mut hi = -1.0f64;
            for &v in &mono[s0..s1.min(n)] {
                if v < lo {
                    lo = v;
                }
                if v > hi {
                    hi = v;
                }
            }
            let y_lo = mid_y - (hi.clamp(-1.05, 1.05) * amp) as i64;
            let y_hi = mid_y - (lo.clamp(-1.05, 1.05) * amp) as i64;
            cv.vline(x + xx, y_lo, y_hi, hex("38BDF8"));
            // RMS envelope lines
            let r = rms_env[s0.min(n - 1)].min(1.05);
            cv.set(x + xx, mid_y - (r * amp) as i64, hex("F59E0B"));
            cv.set(x + xx, mid_y + (r * amp) as i64, hex("F59E0B"));
        }
        // clipping markers (≤50)
        let n_clip = clip_idx.len();
        if n_clip > 0 {
            let step = (n_clip / 50).max(1);
            for &ci in clip_idx.iter().step_by(step) {
                let cx = x + (ci as f64 / n as f64 * w as f64) as i64;
                let cy = mid_y - (mono[ci].clamp(-1.05, 1.05) * amp) as i64;
                cv.circle(cx, cy, 4, hex("EF4444"));
            }
            cv.text(
                x + 8,
                y + h - 14,
                &format!("CLIPPING ({} PTS)", n_clip),
                hex("EF4444"),
                1,
            );
        }
        cv.text(x + 8, y + h - 26, "AMPLITUDE (LINEAR)", text_c, 1);
    }

    // ---- Panel 2: Mel spectrogram ----
    {
        let (x, y, w, h) = (main_x0, row_y[1], main_w, panel_h(1));
        cv.fill_rect(x, y, w, h, panel_bg);
        cv.rect(x, y, w, h, grid);
        cv.text(
            x + 8,
            y + 6,
            "PANEL 2: LOG-FREQUENCY MEL-SPECTROGRAM [-80 DBFS TO 0 DBFS] (MAGMA DYNAMIC COLORMAP)",
            hex("F43F5E"),
            2,
        );
        let plot_x0 = x + 46;
        let plot_y0 = y + 26;
        let plot_w = w - 56;
        let plot_h = h - 40;
        // heatmap
        for xx in 0..plot_w {
            let f_idx = (xx as f64 / plot_w as f64 * mel_db.len() as f64) as usize;
            let f_idx = f_idx.min(mel_db.len().saturating_sub(1));
            for yy in 0..plot_h {
                let mel_row =
                    ((plot_h - 1 - yy) as f64 / plot_h as f64 * (n_mels - 1) as f64) as usize;
                let v = (mel_db[f_idx][mel_row] + 80.0) / 80.0;
                cv.set(plot_x0 + xx, plot_y0 + yy, magma(v));
            }
        }
        // freq tick labels
        for &f in &[
            60.0f64, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
        ] {
            if f > sr as f64 / 2.0 {
                continue;
            }
            let frac = hz_to_mel(f) / hz_to_mel(sr as f64 / 2.0);
            let yy = plot_y0 + plot_h - (frac * plot_h as f64) as i64;
            cv.hline(plot_x0 - 4, plot_x0 - 1, yy, text_c);
            let label = if f < 1000.0 {
                format!("{} HZ", f as i64)
            } else {
                format!("{}K HZ", (f / 1000.0) as i64)
            };
            cv.text(x + 2, yy - 3, &label, text_c, 1);
        }
        // Mud band 200-400Hz
        let band = |lo: f64, hi: f64| -> (i64, i64) {
            let top = hz_to_mel(hi) / hz_to_mel(sr as f64 / 2.0);
            let bot = hz_to_mel(lo) / hz_to_mel(sr as f64 / 2.0);
            (
                plot_y0 + plot_h - (top * plot_h as f64) as i64,
                plot_y0 + plot_h - (bot * plot_h as f64) as i64,
            )
        };
        let (m0, m1) = band(200.0, 400.0);
        cv.rect(plot_x0, m0, plot_w, m1 - m0, hex("3B82F6"));
        cv.text(
            plot_x0 + 6,
            (m0 + m1) / 2 - 3,
            "MUD / LOW-MID (200-400 HZ)",
            hex("93C5FD"),
            1,
        );
        let (s0, s1) = band(6000.0, 8500.0);
        cv.rect(plot_x0, s0, plot_w, s1 - s0, hex("EC4899"));
        cv.text(
            plot_x0 + 6,
            (s0 + s1) / 2 - 3,
            "SIBILANCE / HARSH (6-8.5 KHZ)",
            hex("F472B6"),
            1,
        );
        cv.text(x + 8, y + h - 14, "MEL FREQUENCY SCALE", text_c, 1);
        // colorbar
        let cb_x = x + w - 18;
        for yy in 0..plot_h {
            let v = 1.0 - yy as f64 / plot_h as f64;
            cv.vline(cb_x, plot_y0 + yy, plot_y0 + yy, magma(v));
            cv.vline(cb_x + 1, plot_y0 + yy, plot_y0 + yy, magma(v));
            cv.vline(cb_x + 2, plot_y0 + yy, plot_y0 + yy, magma(v));
            cv.vline(cb_x + 3, plot_y0 + yy, plot_y0 + yy, magma(v));
            cv.vline(cb_x + 4, plot_y0 + yy, plot_y0 + yy, magma(v));
            cv.vline(cb_x + 5, plot_y0 + yy, plot_y0 + yy, magma(v));
        }
        cv.text(cb_x - 4, plot_y0 + plot_h + 4, "DBFS", text_c, 1);
    }

    // ---- Panel 3: Chromagram ----
    {
        let (x, y, w, h) = (main_x0, row_y[2], main_w, panel_h(2));
        cv.fill_rect(x, y, w, h, panel_bg);
        cv.rect(x, y, w, h, grid);
        cv.text(
            x + 8,
            y + 6,
            "PANEL 3: 12-BIN TONAL CHROMAGRAM & HARMONIC DISSONANCE MATRIX",
            hex("F59E0B"),
            2,
        );
        let plot_x0 = x + 30;
        let plot_y0 = y + 26;
        let plot_w = w - 40;
        let plot_h = h - 50;
        for xx in 0..plot_w {
            let f_idx = ((xx as f64 / plot_w as f64) * n_chroma_frames as f64) as usize;
            let f_idx = f_idx.min(n_chroma_frames.saturating_sub(1));
            for yy in 0..plot_h {
                let pc = ((plot_h - 1 - yy) as f64 / plot_h as f64 * 12.0) as usize;
                let v = chroma[pc.min(11)][f_idx];
                cv.set(plot_x0 + xx, plot_y0 + yy, inferno(v));
            }
        }
        let labels = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        for (i, l) in labels.iter().enumerate() {
            let yy = plot_y0 + plot_h - ((i as f64 + 0.5) / 12.0 * plot_h as f64) as i64;
            cv.text(x + 4, yy - 3, l, hex("FCD34D"), 1);
        }
        cv.text(
            x + 8,
            y + h - 16,
            "TIME (SECONDS)  /  PITCH CLASS",
            text_c,
            1,
        );
    }

    // ---- Panel 4A: Goniometer ----
    {
        let row3_w = main_w / 2 - 8;
        let (x, y, w, h) = (main_x0, row_y[3], row3_w, panel_h(3));
        cv.fill_rect(x, y, w, h, panel_bg);
        cv.rect(x, y, w, h, grid);
        cv.text(
            x + 8,
            y + 6,
            "PANEL 4A: STEREO GONIOMETER",
            hex("10B981"),
            2,
        );
        let plot_x0 = x + 12;
        let plot_y0 = y + 26;
        let plot_w = w - 24;
        let plot_h = h - 60;
        let cx = plot_x0 + plot_w / 2;
        let cy = plot_y0 + plot_h / 2;
        cv.hline(plot_x0, plot_x0 + plot_w, cy, grid);
        cv.vline(cx, plot_y0, plot_y0 + plot_h, grid);
        let step = (n / 10000).max(1);
        let mut i = 0usize;
        while i < n {
            let mid = (left[i] + right[i]) / std::f64::consts::SQRT_2;
            let side = (left[i] - right[i]) / std::f64::consts::SQRT_2;
            let px = cx + (side * (plot_w as f64 / 2.1)) as i64;
            let py = cy - (mid * (plot_h as f64 / 2.1)) as i64;
            cv.set(px, py, hex("10B981"));
            i += step;
        }
        let corr_color = if phase_corr >= 0.2 {
            hex("10B981")
        } else if phase_corr >= 0.0 {
            hex("F59E0B")
        } else {
            hex("EF4444")
        };
        cv.text(
            plot_x0 + 6,
            plot_y0 + 6,
            &format!("PHASE CORRELATION: {:+.3}", phase_corr),
            corr_color,
            1,
        );
        cv.text(
            plot_x0 + 6,
            plot_y0 + 16,
            &format!("STEREO: {}", if is_stereo { "YES" } else { "MONO" }),
            corr_color,
            1,
        );
        cv.text(
            x + 8,
            y + h - 16,
            "SIDE (L-R)/WIDTH  VS  MID (L+R)/MONO",
            text_c,
            1,
        );
    }

    // ---- Panel 4B: SSM ----
    {
        let row3_w = main_w / 2 - 8;
        let (x, y, w, h) = (main_x0 + row3_w + 16, row_y[3], row3_w, panel_h(3));
        cv.fill_rect(x, y, w, h, panel_bg);
        cv.rect(x, y, w, h, grid);
        cv.text(
            x + 8,
            y + 6,
            "PANEL 4B: SELF-SIMILARITY STRUCTURAL MATRIX (SSM)",
            hex("6366F1"),
            1,
        );
        let plot_x0 = x + 12;
        let plot_y0 = y + 26;
        let plot_w = w - 24;
        let plot_h = h - 60;
        for xx in 0..plot_w {
            let a = ((xx as f64 / plot_w as f64) * n_chroma_frames as f64) as usize;
            let a = a.min(n_chroma_frames.saturating_sub(1));
            for yy in 0..plot_h {
                let b =
                    ((plot_h - 1 - yy) as f64 / plot_h as f64 * n_chroma_frames as f64) as usize;
                let v = ssm[a][b.min(n_chroma_frames.saturating_sub(1))];
                cv.set(plot_x0 + xx, plot_y0 + yy, viridis(v));
            }
        }
        cv.text(x + 8, y + h - 16, "TIME (S) VS TIME (S)", text_c, 1);
    }

    // ---- Panel 5: HUD ----
    {
        let (x, y, w) = (hud_x0, margin, hud_x1 - hud_x0);
        let h = (H as i64) - 2 * margin;
        cv.fill_rect(x, y, w, h, panel_bg);
        cv.rect(x, y, w, h, hex("38BDF8"));
        let mut cy = y + 10;
        let lh = 12i64;
        let put = |cv: &mut Canvas, cy: &mut i64, s: String| {
            cv.text(x + 10, *cy, &s, text_c, 1);
            *cy += lh;
        };
        put(
            &mut cv,
            &mut cy,
            "+-----------------------------------------+".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            "      VLM ACOUSTIC DIAGNOSTIC HUD".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            "+-----------------------------------------+".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            format!("  * SAMPLE RATE:       {} HZ", py::comma_int(sr as i64)),
        );
        put(
            &mut cv,
            &mut cy,
            format!(
                "  * CHANNELS:          {}",
                if is_stereo {
                    "STEREO (2.0)"
                } else {
                    "MONO (1.0)"
                }
            ),
        );
        put(
            &mut cv,
            &mut cy,
            format!("  * DURATION:          {:.3} SEC", duration),
        );
        put(
            &mut cv,
            &mut cy,
            format!("  * MAX TRUE PEAK:     {:.2} DBFS", max_peak_db),
        );
        put(
            &mut cv,
            &mut cy,
            format!("  * INTEGRATED RMS:    {:.2} DBFS", integrated_rms),
        );
        put(
            &mut cv,
            &mut cy,
            format!("  * DYNAMIC CREST:     {:.2} DB", crest_factor),
        );
        put(
            &mut cv,
            &mut cy,
            format!("  * PHASE CORR (R):    {:+.3}", phase_corr),
        );
        put(
            &mut cv,
            &mut cy,
            format!("  * TOTAL CLIP EVENTS: {}", clip_idx.len()),
        );
        put(
            &mut cv,
            &mut cy,
            "+-----------------------------------------+".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            "         DIAGNOSTIC STATUS FLAGS".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            "+-----------------------------------------+".to_string(),
        );
        let mut warnings: Vec<String> = Vec::new();
        if !clip_idx.is_empty() {
            warnings.push("[!] SEVERE: DIGITAL CLIPPING DETECTED".into());
        }
        if crest_factor < 6.0 {
            warnings.push("[!] CAUTION: OVER-COMPRESSION (SQUASH)".into());
        } else if crest_factor > 18.0 {
            warnings.push("[I] INFO: HIGHLY DYNAMIC CONTENT".into());
        }
        if phase_corr < 0.0 {
            warnings.push("[!] CRITICAL: MONO PHASE INVERSION RISK".into());
        } else if phase_corr < 0.3 && is_stereo {
            warnings.push("[I] NOTICE: WIDE STEREO / WEAK CENTER".into());
        }
        if warnings.is_empty() {
            warnings.push("[OK] ALL METRICS WITHIN NOMINAL TARGETS".into());
        }
        for w_ in warnings {
            put(&mut cv, &mut cy, format!("  {}", w_));
        }
        put(
            &mut cv,
            &mut cy,
            "+-----------------------------------------+".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            "        VLM INSPECTION CHEAT-SHEET".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            "+-----------------------------------------+".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            "  1. MUD CHECK:     HOT BAND @ 200-400 HZ".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            "  2. HARSHNESS:     SPIKES @ 2-5 KHZ".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            "  3. SIBILANCE:     VERTICAL STREAKS @ 6-8K".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            "  4. HARMONICITY:   CHROMA ROW ALIGNMENTS".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            "  5. SECTIONS:      SSM CHECKERBOARD BLOCKS".to_string(),
        );
        put(
            &mut cv,
            &mut cy,
            "+-----------------------------------------+".to_string(),
        );
    }

    cv.save_png(output_image_path)
}
