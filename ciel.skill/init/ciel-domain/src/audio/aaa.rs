//! Port of `skills/procedural-audio/scripts/aaa_audio_generator.py` (v3.0.0).
//! CLI: `--sfx {bullet,v8,creature,rain} --out output.wav --depth {16,24}
//!       --duration 3.0`

use crate::audio::wav;
use crate::common::cli::{self, ArgSpec};
use crate::common::rng::Rng;
use std::f64::consts::PI;

const SAMPLE_RATE: u32 = 44100;
const SOUND_SPEED: f64 = 343.0; // m/s

// 1. DICE SUPERSONIC BULLET CRACK & MUZZLE BLAST
fn bake_supersonic_bullet(distance_m: f64, miss_dist_m: f64, speed_mps: f64) -> Vec<f64> {
    let mut rng = Rng::new();
    let mach = speed_mps / SOUND_SPEED;
    let theta_m = (1.0 / mach).asin();

    let x_travel = distance_m - miss_dist_m / theta_m.tan();
    let t_crack = (x_travel / speed_mps) + (miss_dist_m / (theta_m.cos() * SOUND_SPEED));
    let t_muzzle = distance_m / SOUND_SPEED;

    let total_dur = t_muzzle + 0.8;
    let num_samples = (total_dur * SAMPLE_RATE as f64) as usize;
    let mut samples = vec![0.0f64; num_samples];

    // 1. Whitham N-Wave Crack
    let n_crack_start = (t_crack * SAMPLE_RATE as f64) as usize;
    let n_wave_len = (0.00045 * SAMPLE_RATE as f64) as usize; // 450 us
    let crack_peak = 1.0 / miss_dist_m.max(0.2).powf(0.75);

    for i in 0..n_wave_len {
        let idx = n_crack_start + i;
        if idx < num_samples {
            let t_norm = i as f64 / n_wave_len as f64;
            let n_val = (1.0 - 2.0 * t_norm) * crack_peak;
            let win = (PI * t_norm).sin();
            samples[idx] += n_val * win * 0.9;
        }
    }

    // 2. Subsonic Muzzle Boom
    let n_muzzle_start = (t_muzzle * SAMPLE_RATE as f64) as usize;
    let muzzle_dur = (0.6 * SAMPLE_RATE as f64) as usize;
    for i in 0..muzzle_dur {
        let idx = n_muzzle_start + i;
        if idx < num_samples {
            let t = i as f64 / SAMPLE_RATE as f64;
            let env = (-t * 12.0).exp();
            let boom = (2.0 * PI * 65.0 * t * (-t * 8.0).exp()).sin() * 0.7;
            let rumble = (rng.random() * 2.0 - 1.0) * (-t * 6.0).exp() * 0.3;
            samples[idx] += (boom + rumble) * env * (1.0 / (distance_m * 0.1).sqrt());
        }
    }
    samples
}

// 2. PHYSICAL V8 COMBUSTION ENGINE REV
fn bake_v8_engine_rev(duration: f64) -> Vec<f64> {
    let num_samples = (duration * SAMPLE_RATE as f64) as usize;
    let mut samples = vec![0.0f64; num_samples];

    let firing_offsets = [
        0.0,
        PI * 0.5,
        PI * 1.5,
        PI,
        PI * 2.5,
        PI * 2.0,
        PI * 3.5,
        PI * 3.0,
    ];
    let mut crank_angle = 0.0f64;
    let mut lpf_state = 0.0f64;

    for (n, s) in samples.iter_mut().enumerate() {
        let t = n as f64 / SAMPLE_RATE as f64;
        let (rpm, throttle) = if t < 0.5 {
            (900.0, 0.05)
        } else if t < 2.0 {
            (
                900.0 + (6500.0 - 900.0) * (((t - 0.5) / 1.5) * (PI / 2.0)).sin(),
                0.95,
            )
        } else {
            (1200.0 + (6500.0 - 1200.0) * (-(t - 2.0) * 2.5).exp(), 0.02)
        };

        let rad_per_sec = (rpm / 60.0) * (2.0 * PI);
        crank_angle += rad_per_sec / SAMPLE_RATE as f64;
        if crank_angle >= 4.0 * PI {
            crank_angle -= 4.0 * PI;
        }

        let mut combustion_sum = 0.0;
        for &phi_offset in &firing_offsets {
            let phi = (crank_angle + phi_offset).rem_euclid(4.0 * PI);
            if phi < 1.85 {
                let norm = phi / 1.85;
                let pulse = (PI * norm).sin().powf(2.2) * (-3.5 * norm).exp();
                combustion_sum += pulse * (0.3 + 0.7 * throttle);
            }
        }

        let non_linear = combustion_sum
            + 0.35 * (combustion_sum * combustion_sum.abs() - combustion_sum.powi(3) * 0.1);
        let fc = 180.0 + (rpm / 7000.0) * 950.0 + throttle * 400.0;
        let alpha = (-2.0 * PI * fc / SAMPLE_RATE as f64).exp();
        lpf_state = (1.0 - alpha) * non_linear + alpha * lpf_state;

        *s = lpf_state * 1.2;
    }
    samples
}

// 3. VOCAL TRACT CREATURE ROAR (with chaos bifurcation)
fn bake_vocalien_creature_roar(duration: f64, mass_kg: f64, chaos: f64) -> Vec<f64> {
    let num_samples = (duration * SAMPLE_RATE as f64) as usize;
    let mut samples = vec![0.0f64; num_samples];

    let dt = 1.0 / SAMPLE_RATE as f64;
    let (mut x1, mut x2, mut v1, mut v2) = (0.0001f64, 0.0001f64, 0.0f64, 0.0f64);
    let mut fwd = [0.0f64; 10];
    let mut bwd = [0.0f64; 10];
    let mut area = [1.0f64; 10];
    let mut prev_lip = 0.0f64;

    let mass_scale = (80.0 / mass_kg.max(0.1)).powf(0.38);
    let m1 = 0.12 / mass_scale;
    let m2 = 0.03 / mass_scale;
    let k1 = 80.0 * mass_scale;
    let k2 = 20.0 * mass_scale;
    let kc = 25.0 * mass_scale;
    let b1 = 0.015 * (m1 * k1).sqrt();
    let b2 = 0.015 * (m2 * k2).sqrt();

    for (n, s) in samples.iter_mut().enumerate() {
        let t = n as f64 / SAMPLE_RATE as f64;
        let env = (PI * (t / duration)).sin().powf(0.8);
        let mut ps = 1800.0 * env;
        if chaos > 0.01 && env > 0.3 {
            ps += chaos * 65.0 * (x1 * 1600.0).sin();
        }

        let a1 = (0.014 * (x1 + 0.0002)).max(1e-6);
        let a2 = (0.014 * (x2 + 0.0002)).max(1e-6);
        let a_min = a1.min(a2);

        let ug = ((2.0 * ps) / 1.14).max(0.0).sqrt() * a_min;
        let p1 = ps * (1.0 - (a_min / a1).powi(2));

        let fc1 = if x1 + 0.0002 < 0.0 {
            3.0 * k1 * (x1 + 0.0002)
        } else {
            0.0
        };
        let fc2 = if x2 + 0.0002 < 0.0 {
            3.0 * k2 * (x2 + 0.0002)
        } else {
            0.0
        };

        let acc1 = (p1 * 0.014 - k1 * x1 - kc * (x1 - x2) - b1 * v1 + fc1) / m1;
        let acc2 = (-k2 * x2 - kc * (x2 - x1) - b2 * v2 + fc2) / m2;

        v1 += acc1 * dt;
        v2 += acc2 * dt;
        x1 += v1 * dt;
        x2 += v2 * dt;

        let mouth_open = 0.4 + 0.6 * env;
        for (i, a) in area.iter_mut().enumerate().take(10) {
            let frac = i as f64 / 9.0;
            *a = (1.0 + 0.5 * (frac * PI).sin())
                * (if i == 9 { 0.2 + 1.8 * mouth_open } else { 1.0 });
        }

        fwd[0] = ug + bwd[0] * 0.65;
        for i in 0..9 {
            let r = (area[i + 1] - area[i]) / (area[i + 1] + area[i]);
            let f = fwd[i];
            let b = bwd[i + 1];
            fwd[i + 1] = (1.0 + r) * f - r * b;
            bwd[i] = r * f + (1.0 - r) * b;
        }

        let lip_out = fwd[9] - prev_lip;
        prev_lip = fwd[9];
        *s = (lip_out * 4.5).tanh() * env;
    }
    samples
}

// 4. MICRO-GRANULAR RAIN DOWNPOUR ON HELMET VISOR
fn bake_rain_visor_downpour(duration: f64, drops_per_sec: f64) -> Vec<f64> {
    let mut rng = Rng::new();
    let num_samples = (duration * SAMPLE_RATE as f64) as usize;
    let mut samples = vec![0.0f64; num_samples];

    let mut current_time = 0.0f64;
    while current_time < duration {
        current_time += rng.expovariate(drops_per_sec);
        if current_time >= duration {
            break;
        }

        let f0 = 4800.0 + (rng.random() * 600.0 - 300.0);
        let decay = 450.0;
        let amp = 0.3 + 0.7 * rng.random();
        let start_idx = (current_time * SAMPLE_RATE as f64) as usize;
        let grain_len = (0.015 * SAMPLE_RATE as f64) as usize; // 15 ms

        for i in 0..grain_len {
            let idx = start_idx + i;
            if idx < num_samples {
                let t = i as f64 / SAMPLE_RATE as f64;
                let val = (2.0 * PI * f0 * t).sin() * (-t * decay).exp() * amp;
                samples[idx] += val * 0.4;
            }
        }
    }
    samples
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-audio aaa",
        argv,
        &[
            ArgSpec::value("sfx", None, "sfx")
                .def("bullet")
                .choices(&["bullet", "v8", "creature", "rain"]),
            ArgSpec::value("out", None, "out").def("output.wav"),
            ArgSpec::int("depth", None, "depth")
                .def("16")
                .choices(&["16", "24"]),
            ArgSpec::float("duration", None, "duration").def("3.0"),
        ],
    );

    let sfx = args.get_or("sfx", "bullet");
    let out = args.get_or("out", "output.wav");
    let depth = args.int("depth", 16) as u16;
    let duration = args.float("duration", 3.0);

    let data = match sfx.as_str() {
        "bullet" => bake_supersonic_bullet(120.0, 1.5, 880.0),
        "v8" => bake_v8_engine_rev(duration),
        "creature" => bake_vocalien_creature_roar(duration, 650.0, 0.75),
        "rain" => bake_rain_visor_downpour(duration, 120.0),
        _ => bake_supersonic_bullet(120.0, 1.5, 880.0),
    };

    wav::write_wav_mono(&out, &data, SAMPLE_RATE, depth);
    0
}
