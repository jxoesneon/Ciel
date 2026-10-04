//! Port of `skills/procedural-audio/scripts/procedural_audio_generator.py`.
//! CLI: `--sfx {click,whoosh,metal,wood,laser,explosion,drone,ir}`
//!      `--format {float32,pcm24} --out output.wav --sr 48000 --duration 2.0`

use crate::audio::wav;
use crate::common::cli::{self, ArgSpec};
use crate::common::rng::Rng;
use std::f64::consts::PI;

// --- DSP MATHEMATICAL PRIMITIVES ---

struct PaulKelletPinkNoise {
    b0: f64,
    b1: f64,
    b2: f64,
    b3: f64,
    b4: f64,
    b5: f64,
    b6: f64,
    rng: Rng,
}

impl PaulKelletPinkNoise {
    fn new(rng: Rng) -> Self {
        PaulKelletPinkNoise {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
            rng,
        }
    }
    fn next_sample(&mut self) -> f64 {
        let white = self.rng.random() * 2.0 - 1.0;
        self.b0 = 0.99886 * self.b0 + white * 0.0555179;
        self.b1 = 0.99332 * self.b1 + white * 0.0750759;
        self.b2 = 0.96900 * self.b2 + white * 0.1538520;
        self.b3 = 0.86650 * self.b3 + white * 0.3104856;
        self.b4 = 0.55000 * self.b4 + white * 0.5329522;
        self.b5 = -0.7616 * self.b5 - white * 0.0168980;
        let pink =
            (self.b0 + self.b1 + self.b2 + self.b3 + self.b4 + self.b5 + self.b6 + white * 0.5362)
                * 0.11;
        self.b6 = white * 0.115926;
        pink
    }
}

struct StateVariableFilter {
    fs: f64,
    s1: f64,
    s2: f64,
}

impl StateVariableFilter {
    fn new(sample_rate: f64) -> Self {
        StateVariableFilter {
            fs: sample_rate,
            s1: 0.0,
            s2: 0.0,
        }
    }
    /// Returns (lowpass, bandpass, highpass)
    fn process(&mut self, x: f64, cutoff_hz: f64, q: f64) -> (f64, f64, f64) {
        let cutoff_hz = cutoff_hz.max(10.0).min(0.48 * self.fs);
        let q = q.max(0.1);
        let g = (PI * cutoff_hz / self.fs).tan();
        let k = 1.0 / q;
        let hp = (x - (2.0 * k + g) * self.s1 - self.s2) / (1.0 + 2.0 * k * g + g * g);
        let bp = g * hp + self.s1;
        self.s1 = g * hp + bp;
        let lp = g * bp + self.s2;
        self.s2 = g * bp + lp;
        (lp, bp, hp)
    }
}

// --- PROCEDURAL SYNTHESIZERS ---

fn synthesize_ui_click(sample_rate: u32) -> Vec<(f64, f64)> {
    let duration = 0.035;
    let num_samples = (duration * sample_rate as f64) as usize;
    let mut out = Vec::with_capacity(num_samples);
    let (mut p1, mut p2) = (0.0f64, 0.0f64);
    for i in 0..num_samples {
        let env = (1.0 - (i as f64 / num_samples as f64)).powf(1.5);
        p1 += 2.0 * PI * 2400.0 / sample_rate as f64;
        p2 += 2.0 * PI * 4800.0 / sample_rate as f64;
        let sample = (p1.sin() * 0.65 + p2.sin() * 0.35) * env * 0.4;
        out.push((sample, sample));
    }
    out
}

fn synthesize_ui_whoosh(sample_rate: u32, rng: Rng) -> Vec<(f64, f64)> {
    let duration = 0.22;
    let num_samples = (duration * sample_rate as f64) as usize;
    let mut out = Vec::with_capacity(num_samples);
    let mut svf = StateVariableFilter::new(sample_rate as f64);
    let mut pink = PaulKelletPinkNoise::new(rng);
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let norm_t = t / duration;
        let cutoff = 300.0 + 1600.0 * (PI * norm_t).sin().powi(2);
        let noise = pink.next_sample();
        let (_, bp, _) = svf.process(noise, cutoff, 3.2);
        let env = (PI * norm_t).sin().powi(2);
        let pan_angle = norm_t * (PI / 2.0);
        let left = bp * env * pan_angle.cos() * 0.7;
        let right = bp * env * pan_angle.sin() * 0.7;
        out.push((left, right));
    }
    out
}

fn synthesize_metal_impact(sample_rate: u32, rng: &mut Rng) -> Vec<(f64, f64)> {
    let f0 = 440.0f64;
    let duration = 2.2;
    let num_samples = (duration * sample_rate as f64) as usize;
    let mut out = Vec::with_capacity(num_samples);
    let mode_ratios = [1.0, 1.414, 2.142, 2.761, 3.824, 5.123];
    let mode_gains = [1.0, 0.75, 0.55, 0.40, 0.25, 0.15];
    let mode_t60 = [2.0, 1.6, 1.2, 0.8, 0.45, 0.2];
    let mut phases = [0.0f64; 6];
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let mut sample = 0.0;
        for k in 0..mode_ratios.len() {
            let freq = f0 * mode_ratios[k];
            if freq < 0.48 * sample_rate as f64 {
                let decay = (-3.0 * t / mode_t60[k]).exp();
                phases[k] += 2.0 * PI * freq / sample_rate as f64;
                sample += mode_gains[k] * decay * phases[k].sin();
            }
        }
        if i < (0.003 * sample_rate as f64) as usize {
            sample += (rng.random() * 2.0 - 1.0) * 0.6;
        }
        let sat = (sample * 1.5).tanh() * 0.75;
        out.push((sat, sat));
    }
    out
}

fn synthesize_wood_thud(sample_rate: u32, rng: &mut Rng) -> Vec<(f64, f64)> {
    let base_pitch = 110.0f64;
    let duration = 0.65;
    let num_samples = (duration * sample_rate as f64) as usize;
    let mut out = Vec::with_capacity(num_samples);
    let mut body_phase = 0.0f64;
    let mode_ratios = [1.0, 1.84, 2.72, 3.91];
    let mode_gains = [0.8, 0.5, 0.3, 0.15];
    let mode_decay = [0.08, 0.05, 0.025, 0.012];
    let mut phases = [0.0f64; 4];

    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let pitch = 55.0 + (base_pitch * 2.0) * (-t / 0.012).exp();
        body_phase += 2.0 * PI * pitch / sample_rate as f64;
        let mut sample = body_phase.sin() * (-t / 0.045).exp();
        for k in 0..mode_ratios.len() {
            phases[k] += 2.0 * PI * (base_pitch * mode_ratios[k]) / sample_rate as f64;
            sample += mode_gains[k] * phases[k].sin() * (-t / mode_decay[k]).exp();
        }
        if i < (0.004 * sample_rate as f64) as usize {
            sample += (rng.random() * 2.0 - 1.0) * 0.4;
        }
        let sat = (sample * 1.8).tanh() * 0.8;
        out.push((sat, sat));
    }
    out
}

fn synthesize_laser_blaster(sample_rate: u32) -> Vec<(f64, f64)> {
    let duration = 0.35;
    let num_samples = (duration * sample_rate as f64) as usize;
    let mut out = Vec::with_capacity(num_samples);
    let mut phase = 0.0f64;
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let env = (-8.0 * t).exp();
        let freq = 80.0 + 3120.0 * (-16.0 * t).exp();
        phase += 2.0 * PI * freq / sample_rate as f64;
        let norm_phase = (phase / (2.0 * PI)).rem_euclid(1.0);
        let saw = 2.0 * norm_phase - 1.0;
        let sample = (saw * env * 3.0).tanh() * 0.75;
        out.push((sample, sample * 0.95));
    }
    out
}

fn synthesize_explosion(sample_rate: u32, duration: f64, rng: &mut Rng) -> Vec<(f64, f64)> {
    let num_samples = (duration * sample_rate as f64) as usize;
    let mut out = Vec::with_capacity(num_samples);
    let mut sub_phase = 0.0f64;
    let mut svf = StateVariableFilter::new(sample_rate as f64);
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let noise_env = (-3.2 * t).exp();
        let sub_env = (-1.5 * t).exp() * (1.0 - (-18.0 * t).exp());
        let sub_freq = 30.0 + 45.0 * (-3.5 * t).exp();
        sub_phase += 2.0 * PI * sub_freq / sample_rate as f64;
        let sub = sub_phase.sin() * sub_env * 0.85;
        let white = rng.random() * 2.0 - 1.0;
        let cutoff = 150.0 + 3800.0 * (-5.5 * t).exp();
        let (lp, _, _) = svf.process(white, cutoff, 0.8);
        let mixed = (lp * noise_env * 1.3) + sub;
        let sat = (mixed * 1.6).tanh() * 0.85;
        out.push((sat, sat));
    }
    out
}

fn synthesize_ambient_drone(sample_rate: u32, duration: f64, rng: Rng) -> Vec<(f64, f64)> {
    let root_hz = 55.0f64;
    let num_samples = (duration * sample_rate as f64) as usize;
    let mut out = Vec::with_capacity(num_samples);
    let (mut p1, mut p2, mut p3, mut p_lfo) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let mut pink = PaulKelletPinkNoise::new(rng);
    let mut svf = StateVariableFilter::new(sample_rate as f64);
    for _i in 0..num_samples {
        p_lfo += 2.0 * PI * 0.12 / sample_rate as f64;
        let lfo = p_lfo.sin();
        let f1 = root_hz + lfo * 1.5;
        let f2 = root_hz * 1.5 + (p_lfo * 0.7).cos() * 0.8;
        let f3 = root_hz * 2.0;
        p1 += 2.0 * PI * f1 / sample_rate as f64;
        p2 += 2.0 * PI * f2 / sample_rate as f64;
        p3 += 2.0 * PI * f3 / sample_rate as f64;
        let drone = p1.sin() * 0.4 + p2.sin() * 0.25 + p3.sin() * 0.15;
        let noise = pink.next_sample();
        let (lp_noise, _, _) = svf.process(noise, 450.0, 0.707);
        let left = (drone + lp_noise * 0.08) * 0.7;
        let right = (drone + lp_noise * 0.075) * 0.7;
        out.push((left, right));
    }
    out
}

fn synthesize_procedural_impulse_response(
    sample_rate: u32,
    duration: f64,
    rng: &mut Rng,
) -> Vec<(f64, f64)> {
    let decay = 2.0f64;
    let num_samples = (duration * sample_rate as f64) as usize;
    let early_taps: [(usize, f64); 4] = [
        ((sample_rate as f64 * 0.012) as usize, 0.7),
        ((sample_rate as f64 * 0.024) as usize, 0.5),
        ((sample_rate as f64 * 0.038) as usize, 0.35),
        ((sample_rate as f64 * 0.055) as usize, 0.25),
    ];
    let mut left_raw = vec![0.0f64; num_samples];
    let mut right_raw = vec![0.0f64; num_samples];

    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let env = (-decay * t).exp();
        let damp = (-4.5 * t).exp();
        let u1 = rng.random().max(1e-12);
        let u2 = rng.random().max(1e-12);
        let n_l = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
        let n_r = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).sin();
        left_raw[i] = n_l * env * damp * 0.5;
        right_raw[i] = n_r * env * damp * 0.5;
    }

    for &(delay_idx, gain) in &early_taps {
        if delay_idx < num_samples {
            left_raw[delay_idx] += gain * 0.4;
        }
        if delay_idx + 64 < num_samples {
            right_raw[delay_idx + 64] += gain * 0.38;
        }
    }

    let max_val = left_raw
        .iter()
        .fold(0.0f64, |a, &x| a.max(x.abs()))
        .max(right_raw.iter().fold(0.0f64, |a, &x| a.max(x.abs())))
        .max(1e-6);
    left_raw
        .iter()
        .zip(right_raw.iter())
        .map(|(&l, &r)| (l / max_val * 0.95, r / max_val * 0.95))
        .collect()
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-audio synth",
        argv,
        &[
            ArgSpec::value("sfx", None, "sfx").def("click").choices(&[
                "click",
                "whoosh",
                "metal",
                "wood",
                "laser",
                "explosion",
                "drone",
                "ir",
            ]),
            ArgSpec::value("format", None, "format")
                .def("float32")
                .choices(&["float32", "pcm24"]),
            ArgSpec::value("out", None, "out").def("output.wav"),
            ArgSpec::int("sr", None, "sr").def("48000"),
            ArgSpec::float("duration", None, "duration").def("2.0"),
        ],
    );

    let sfx = args.get_or("sfx", "click");
    let format = args.get_or("format", "float32");
    let out = args.get_or("out", "output.wav");
    let sr = args.int("sr", 48000) as u32;
    let duration = args.float("duration", 2.0);

    println!(
        "[ProceduralAudioGenerator] Synthesizing '{}' at {}Hz ({})...",
        sfx, sr, format
    );

    let mut rng = Rng::new();
    let samples = match sfx.as_str() {
        "click" => synthesize_ui_click(sr),
        "whoosh" => synthesize_ui_whoosh(sr, Rng::new()),
        "metal" => synthesize_metal_impact(sr, &mut rng),
        "wood" => synthesize_wood_thud(sr, &mut rng),
        "laser" => synthesize_laser_blaster(sr),
        "explosion" => synthesize_explosion(sr, duration, &mut rng),
        "drone" => synthesize_ambient_drone(sr, duration, Rng::new()),
        "ir" => synthesize_procedural_impulse_response(sr, duration, &mut rng),
        _ => synthesize_ui_click(sr),
    };

    if format == "float32" {
        wav::write_wav_32bit_float(&out, &samples, sr);
    } else {
        wav::write_wav_24bit_pcm(&out, &samples, sr);
    }
    println!(
        "[ProceduralAudioGenerator] Successfully baked {} frames to {}",
        samples.len(),
        out
    );
    0
}
