//! Port of `skills/procedural-audio/scripts/advanced_humanizer.py` —
//! Hermode dynamic just intonation, Voss-McCartney pink noise, OU timing
//! drift, instrument pockets, velocity-dependent swing, salience hierarchy.
//! The original has no CLI; `ciel-audio humanize` runs its __main__ test bench.

use crate::common::rng::Rng;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::f64::consts::PI;

const JUST_RATIOS: [(f64, f64, f64); 12] = [
    (1.0, 1.0, 0.0),
    (16.0, 15.0, 11.73),
    (9.0, 8.0, 3.91),
    (6.0, 5.0, 15.64),
    (5.0, 4.0, -13.69),
    (4.0, 3.0, -1.96),
    (45.0, 32.0, -9.78),
    (3.0, 2.0, 1.96),
    (8.0, 5.0, 13.69),
    (5.0, 3.0, -15.64),
    (9.0, 5.0, 17.60),
    (15.0, 8.0, -11.73),
];

/// HermodeTuningEngine.retune_chord → (midi, cents_correction, tuned_hz)
pub fn retune_chord(root_midi: i64, chord_midi_notes: &[i64]) -> Vec<(i64, f64, f64)> {
    let mut results = Vec::new();
    for &note in chord_midi_notes {
        let interval = ((note - root_midi) % 12 + 12) % 12;
        let correction_cents = JUST_RATIOS[interval as usize].2;
        let freq_tet = 440.0 * 2f64.powf((note as f64 - 69.0) / 12.0);
        let freq_just = freq_tet * 2f64.powf(correction_cents / 1200.0);
        results.push((
            note,
            correction_cents,
            crate::common::py::round_py(freq_just, 3),
        ));
    }
    results
}

struct VossMcCartneyPinkNoise {
    dice: Vec<f64>,
    running_sum: f64,
    counter: u64,
    rng: Rng,
}

impl VossMcCartneyPinkNoise {
    fn new(num_dice: usize, rng: &mut Rng) -> Self {
        let dice: Vec<f64> = (0..num_dice).map(|_| rng.uniform(-1.0, 1.0)).collect();
        let running_sum = dice.iter().sum();
        VossMcCartneyPinkNoise {
            dice,
            running_sum,
            counter: 0,
            rng: Rng::new(),
        }
    }
    fn next_sample(&mut self) -> f64 {
        self.counter += 1;
        let tz = (self.counter & self.counter.wrapping_neg()).trailing_zeros() as usize;
        let die_idx = tz % self.dice.len();
        let old_val = self.dice[die_idx];
        let new_val = self.rng.uniform(-1.0, 1.0);
        self.dice[die_idx] = new_val;
        self.running_sum += new_val - old_val;
        self.running_sum / self.dice.len() as f64
    }
}

struct EnsembleGrooveHumanizer {
    step_sec: f64,
    pink_noise: VossMcCartneyPinkNoise,
    drift_x: f64,
    theta: f64,
    sigma: f64,
    rng: Rng,
}

impl EnsembleGrooveHumanizer {
    fn new(bpm: f64) -> Self {
        let mut rng = Rng::new();
        let pink = VossMcCartneyPinkNoise::new(6, &mut rng);
        EnsembleGrooveHumanizer {
            step_sec: (60.0 / bpm) / 4.0,
            pink_noise: pink,
            drift_x: 0.0,
            theta: 0.4,
            sigma: 0.0035,
            rng,
        }
    }

    fn compute_ensemble_event(
        &mut self,
        instrument: &str,
        step_idx: i64,
        base_velocity: i64,
    ) -> HashMap<String, Value> {
        let dt = self.step_sec;
        let dw = self.rng.gauss(0.0, dt.sqrt());
        self.drift_x += (-self.theta * self.drift_x * dt) + (self.sigma * dw);

        let pink_jitter_ms = self.pink_noise.next_sample() * 2.5;

        let pocket_ms = match instrument.to_lowercase().as_str() {
            "kick" => 0.0,
            "snare" => 8.5,
            "hihat" => -4.0,
            "bass" => 6.0,
            "chords" => -2.0,
            "lead" => 3.0,
            _ => 0.0,
        };

        let is_offbeat = step_idx % 2 == 1;
        let swing_ratio = 0.54 + 0.12 * (base_velocity as f64 / 127.0);
        let swing_offset_sec = if is_offbeat {
            (swing_ratio - 0.5) * (2.0 * self.step_sec)
        } else {
            0.0
        };

        let nominal_time = step_idx as f64 * self.step_sec;
        let total_offset_sec =
            (pocket_ms / 1000.0) + self.drift_x + (pink_jitter_ms / 1000.0) + swing_offset_sec;
        let final_timestamp = (nominal_time + total_offset_sec).max(0.0);

        let salience = if step_idx % 16 == 0 {
            22
        } else if step_idx % 4 == 0 {
            10
        } else if step_idx % 2 == 0 {
            2
        } else {
            -12
        };

        let vel_jitter = self.rng.gauss(0.0, 4.0);
        let final_velocity =
            ((base_velocity + salience) as f64 + vel_jitter).clamp(1.0, 127.0) as i64;

        let gain_trim_db = self.rng.gauss(0.0, 0.35);
        let cutoff_jitter_hz = self.rng.gauss(0.0, 22.0);
        let attack_warp = 1.0 + self.rng.gauss(0.0, 0.04);

        let r = crate::common::py::round_py;
        let mut m = HashMap::new();
        m.insert("instrument".into(), json!(instrument));
        m.insert("step".into(), json!(step_idx));
        m.insert("nominal_sec".into(), json!(r(nominal_time, 5)));
        m.insert("final_sec".into(), json!(r(final_timestamp, 5)));
        m.insert("offset_ms".into(), json!(r(total_offset_sec * 1000.0, 2)));
        m.insert("velocity".into(), json!(final_velocity));
        m.insert("gain_trim_db".into(), json!(r(gain_trim_db, 2)));
        m.insert("cutoff_jitter_hz".into(), json!(r(cutoff_jitter_hz, 1)));
        m.insert("attack_warp".into(), json!(r(attack_warp, 3)));
        m
    }
}

/// Runs the Python `__main__` test bench.
pub fn run(_argv: &[String]) -> i32 {
    println!("=================================================================");
    println!("          ADVANCED HUMANIZER & HERMODE TUNING TEST BENCH         ");
    println!("=================================================================\n");

    println!("--- [1] HERMODE DYNAMIC JUST INTONATION SOLVER ---");
    let c_major_midi = [60i64, 64, 67];
    let tuned_c_major = retune_chord(60, &c_major_midi);
    println!("C Major Triad [C4, E4, G4]:");
    for (note, cents, hz) in &tuned_c_major {
        let tet_hz = 440.0 * 2f64.powf((*note as f64 - 69.0) / 12.0);
        println!(
            "  MIDI {} -> 12-TET: {:.2} Hz | Hermode Cents: {:+6.2}c -> Pure Just: {:.2} Hz",
            note, tet_hz, cents, hz
        );
    }
    println!("  *Notice: E4 is flattened by -13.69c, completely eliminating the 16.8Hz harmonic acoustic beat!*\n");

    println!("--- [2] MULTI-INSTRUMENT ENSEMBLE GROOVE SIMULATION ---");
    let mut humanizer = EnsembleGrooveHumanizer::new(124.0);
    for step in 0..8i64 {
        let kick_ev = humanizer.compute_ensemble_event("kick", step, 100);
        let snare_ev = humanizer.compute_ensemble_event("snare", step, 95);
        let hihat_ev = humanizer.compute_ensemble_event("hihat", step, 85);

        let g = |m: &HashMap<String, Value>, k: &str| m.get(k).cloned().unwrap_or(json!(0));
        let f = |v: &Value| v.as_f64().unwrap_or(0.0);
        println!(
            "Step {:02} (Nominal: {:.4}s):",
            step,
            f(&g(&kick_ev, "nominal_sec"))
        );
        println!(
            "  Kick  : Time={:.4}s (Offset={:+5.1}ms) | Vel={:03} | Cutoff Jitter={:+4.1}Hz",
            f(&g(&kick_ev, "final_sec")),
            f(&g(&kick_ev, "offset_ms")),
            g(&kick_ev, "velocity").as_i64().unwrap_or(0),
            f(&g(&kick_ev, "cutoff_jitter_hz"))
        );
        if step % 4 == 2 {
            println!(
                "  Snare : Time={:.4}s (Offset={:+5.1}ms) | Vel={:03} [Laying Back]",
                f(&g(&snare_ev, "final_sec")),
                f(&g(&snare_ev, "offset_ms")),
                g(&snare_ev, "velocity").as_i64().unwrap_or(0)
            );
        }
        println!(
            "  Hi-Hat: Time={:.4}s (Offset={:+5.1}ms) | Vel={:03} [Driving Forward]",
            f(&g(&hihat_ev, "final_sec")),
            f(&g(&hihat_ev, "offset_ms")),
            g(&hihat_ev, "velocity").as_i64().unwrap_or(0)
        );
    }
    println!("\n=================================================================");
    let _ = PI;
    0
}
