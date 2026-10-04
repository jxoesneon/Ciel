//! Port of `skills/procedural-audio/scripts/microtonal_pitch_engine.py` —
//! 22 Shrutis, Maqam Rast/Bayati, Slendro stretched octave, N-EDO quantizer,
//! cubic Hermite meend. `ciel-audio microtonal` runs the __main__ demo.

pub struct MicrotonalPitchEngine {
    pub concert_a: f64,
    pub shrutis_ratios: [(i64, i64); 22],
}

impl MicrotonalPitchEngine {
    pub fn new(concert_a: f64) -> Self {
        MicrotonalPitchEngine {
            concert_a,
            shrutis_ratios: [
                (1, 1),
                (256, 243),
                (16, 15),
                (10, 9),
                (9, 8),
                (32, 27),
                (6, 5),
                (5, 4),
                (81, 64),
                (4, 3),
                (27, 20),
                (45, 32),
                (729, 512),
                (3, 2),
                (128, 81),
                (8, 5),
                (5, 3),
                (27, 16),
                (16, 9),
                (9, 5),
                (15, 8),
                (243, 128),
            ],
        }
    }

    pub fn cents_to_ratio(&self, cents: f64) -> f64 {
        2f64.powf(cents / 1200.0)
    }

    pub fn ratio_to_cents(&self, ratio: f64) -> f64 {
        1200.0 * ratio.log2()
    }

    pub fn get_shruti_freq(&self, root_freq: f64, shruti_idx: i64, octave: i64) -> f64 {
        let idx = shruti_idx.rem_euclid(22) as usize;
        let (num, den) = self.shrutis_ratios[idx];
        let oct_shift = octave + shruti_idx.div_euclid(22);
        root_freq * (num as f64 / den as f64) * 2f64.powi(oct_shift as i32)
    }

    pub fn get_maqam_rast_freqs(&self, root_freq: f64) -> Vec<f64> {
        let rast_cents = [0.0, 204.0, 355.0, 498.0, 702.0, 906.0, 1057.0, 1200.0];
        rast_cents
            .iter()
            .map(|&c| root_freq * self.cents_to_ratio(c))
            .collect()
    }

    pub fn get_maqam_bayati_freqs(&self, root_freq: f64) -> Vec<f64> {
        let bayati_cents = [0.0, 150.0, 300.0, 500.0, 702.0, 800.0, 1000.0, 1200.0];
        bayati_cents
            .iter()
            .map(|&c| root_freq * self.cents_to_ratio(c))
            .collect()
    }

    pub fn get_gamelan_slendro_freqs(&self, root_freq: f64, stretch_cents: f64) -> Vec<f64> {
        let step = stretch_cents / 5.0;
        (0..6)
            .map(|i| root_freq * 2f64.powf((i as f64 * step) / 1200.0))
            .collect()
    }

    pub fn get_gamelan_ombak_pair(&self, base_freq: f64, beating_hz: f64) -> (f64, f64) {
        (base_freq - beating_hz / 2.0, base_freq + beating_hz / 2.0)
    }

    pub fn quantize_to_edo(&self, freq: f64, root_freq: f64, edo: i64) -> f64 {
        let cents_from_root = self.ratio_to_cents(freq / root_freq);
        let step_size = 1200.0 / edo as f64;
        let nearest_step = (cents_from_root / step_size).round();
        let quantized_cents = nearest_step * step_size;
        root_freq * self.cents_to_ratio(quantized_cents)
    }

    pub fn continuous_meend_interpolation(&self, f_start: f64, f_end: f64, tau: f64) -> f64 {
        let tau_c = tau.clamp(0.0, 1.0);
        let blend = 3.0 * tau_c.powi(2) - 2.0 * tau_c.powi(3);
        f_start + (f_end - f_start) * blend
    }
}

/// Python list-of-floats `print([round(f,2) ...])` formatting.
fn fmt_py_float_list(vals: &[f64]) -> String {
    let items: Vec<String> = vals
        .iter()
        .map(|&v| crate::common::py::py_float(crate::common::py::round_py(v, 2)))
        .collect();
    format!("[{}]", items.join(", "))
}

pub fn run(_argv: &[String]) -> i32 {
    let engine = MicrotonalPitchEngine::new(440.0);
    println!("--- 22 INDIAN SHRUTIS (Root C4 = 261.63 Hz) ---");
    for i in 0..22 {
        let freq = engine.get_shruti_freq(261.63, i, 0);
        println!(
            "Shruti {:02}: {:.2} Hz ({:.1} cents)",
            i,
            freq,
            engine.ratio_to_cents(freq / 261.63)
        );
    }

    println!("\n--- MAQAM RAST FREQUENCIES ---");
    println!(
        "{}",
        fmt_py_float_list(&engine.get_maqam_rast_freqs(261.63))
    );

    println!("\n--- GAMELAN SLENDRO WITH STRETCHED OCTAVE ---");
    println!(
        "{}",
        fmt_py_float_list(&engine.get_gamelan_slendro_freqs(270.0, 1215.0))
    );
    0
}
