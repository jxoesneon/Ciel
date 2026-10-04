//! Port of `skills/procedural-audio/scripts/motif_dna_generator.py` —
//! motivic transformations, 32-bar golden-ratio journey, zero-dep MIDI export.
//! `ciel-audio motif` runs the __main__ demo (Beethoven + Williams seeds,
//! writes /tmp/*.mid) and `ciel-audio motif-midi` exports via CLI args.

use crate::common::cli::{self, ArgSpec};
use serde_json::{json, Map, Value};
use std::fs;

const MODES: [(&str, &[i64]); 10] = [
    ("ionian", &[0, 2, 4, 5, 7, 9, 11]),
    ("dorian", &[0, 2, 3, 5, 7, 9, 10]),
    ("phrygian", &[0, 1, 3, 5, 7, 8, 10]),
    ("lydian", &[0, 2, 4, 6, 7, 9, 11]),
    ("mixolydian", &[0, 2, 4, 5, 7, 9, 10]),
    ("aeolian", &[0, 2, 3, 5, 7, 8, 10]),
    ("locrian", &[0, 1, 3, 5, 6, 8, 10]),
    ("harmonic_minor", &[0, 2, 3, 5, 7, 8, 11]),
    ("hirajoshi", &[0, 2, 3, 7, 8]),
    ("pentatonic_major", &[0, 2, 4, 7, 9]),
];

#[derive(Clone)]
pub struct NoteEvent {
    pub pitch: i64,
    pub duration: f64,
    pub velocity: i64,
    pub bar: f64,
}

impl NoteEvent {
    fn new(pitch: i64, duration: f64, velocity: i64) -> Self {
        NoteEvent {
            pitch,
            duration,
            velocity,
            bar: 0.0,
        }
    }
}

pub struct MotifDNAGenerator {
    root_note: i64,
    scale: &'static [i64],
    tempo: i64,
}

impl MotifDNAGenerator {
    pub fn new(root_note: i64, mode: &str, tempo: i64) -> Self {
        let scale = MODES
            .iter()
            .find(|(n, _)| *n == mode)
            .map(|(_, s)| *s)
            .unwrap_or(MODES[1].1); // dorian default
        MotifDNAGenerator {
            root_note,
            scale,
            tempo,
        }
    }

    fn midi_to_scale_degree(&self, pitch: i64) -> (i64, i64) {
        let pitch_class = (pitch - self.root_note).rem_euclid(12);
        let octave = (pitch - self.root_note).div_euclid(12);
        let mut closest_deg = 0i64;
        let mut min_diff = 99i64;
        for (i, &semitone) in self.scale.iter().enumerate() {
            let diff = (pitch_class - semitone).abs();
            if diff < min_diff {
                min_diff = diff;
                closest_deg = i as i64;
            }
        }
        (octave, closest_deg)
    }

    fn scale_degree_to_midi(&self, octave: i64, degree: i64) -> i64 {
        let scale_len = self.scale.len() as i64;
        let oct_offset = degree.div_euclid(scale_len);
        let deg_idx = degree.rem_euclid(scale_len);
        self.root_note + (octave + oct_offset) * 12 + self.scale[deg_idx as usize]
    }

    fn diatonic_transpose(&self, motif: &[NoteEvent], degree_steps: i64) -> Vec<NoteEvent> {
        motif
            .iter()
            .map(|n| {
                let (octave, deg) = self.midi_to_scale_degree(n.pitch);
                let new_pitch = self.scale_degree_to_midi(octave, deg + degree_steps);
                NoteEvent {
                    pitch: new_pitch,
                    duration: n.duration,
                    velocity: n.velocity,
                    bar: n.bar,
                }
            })
            .collect()
    }

    fn modal_inversion(&self, motif: &[NoteEvent], axis_pitch: Option<i64>) -> Vec<NoteEvent> {
        if motif.is_empty() {
            return Vec::new();
        }
        let axis = axis_pitch.unwrap_or(motif[0].pitch);
        let (axis_oct, axis_deg) = self.midi_to_scale_degree(axis);
        let axis_total_deg = axis_oct * self.scale.len() as i64 + axis_deg;

        motif
            .iter()
            .map(|n| {
                let (n_oct, n_deg) = self.midi_to_scale_degree(n.pitch);
                let n_total_deg = n_oct * self.scale.len() as i64 + n_deg;
                let inverted_deg = 2 * axis_total_deg - n_total_deg;
                NoteEvent {
                    pitch: self.scale_degree_to_midi(0, inverted_deg),
                    duration: n.duration,
                    velocity: n.velocity,
                    bar: n.bar,
                }
            })
            .collect()
    }

    fn retrograde(&self, motif: &[NoteEvent]) -> Vec<NoteEvent> {
        if motif.is_empty() {
            return Vec::new();
        }
        let pitches: Vec<i64> = motif.iter().rev().map(|n| n.pitch).collect();
        let velocities: Vec<i64> = motif.iter().rev().map(|n| n.velocity).collect();
        motif
            .iter()
            .zip(pitches.iter().zip(velocities.iter()))
            .map(|(n, (&p, &v))| NoteEvent::new(p, n.duration, v))
            .collect()
    }

    fn augment(&self, motif: &[NoteEvent], factor: f64) -> Vec<NoteEvent> {
        motif
            .iter()
            .map(|n| NoteEvent::new(n.pitch, n.duration * factor, n.velocity))
            .collect()
    }

    fn diminish(&self, motif: &[NoteEvent], factor: f64) -> Vec<NoteEvent> {
        motif
            .iter()
            .map(|n| NoteEvent::new(n.pitch, (n.duration * factor).max(0.125), n.velocity))
            .collect()
    }

    fn intervallic_expansion(&self, motif: &[NoteEvent], multiplier: f64) -> Vec<NoteEvent> {
        if motif.is_empty() {
            return Vec::new();
        }
        let anchor = motif[0].pitch;
        let mut result = vec![motif[0].clone()];
        for n in &motif[1..] {
            let interval = n.pitch - anchor;
            let expanded_interval = (interval as f64 * multiplier).round() as i64;
            result.push(NoteEvent::new(
                anchor + expanded_interval,
                n.duration,
                ((n.velocity as f64 * 1.1) as i64).min(127),
            ));
        }
        result
    }

    fn liquidate(&self, motif: &[NoteEvent], keep_notes: usize) -> Vec<NoteEvent> {
        let truncated: &[NoteEvent] = if motif.len() >= keep_notes {
            &motif[motif.len() - keep_notes..]
        } else {
            motif
        };
        self.diminish(truncated, 0.5)
    }

    pub fn generate_thematic_journey(
        &self,
        seed_pitches: &[i64],
        seed_durations: &[f64],
    ) -> (Value, Vec<NoteEvent>) {
        let base_motif: Vec<NoteEvent> = seed_pitches
            .iter()
            .zip(seed_durations.iter())
            .map(|(&p, &d)| NoteEvent::new(p, d, 85))
            .collect();

        let m_statement = base_motif.clone();
        let m_step_up = self.diatonic_transpose(&base_motif, 1);
        let m_inversion = self.modal_inversion(&base_motif, None);
        let m_cadence = self.diatonic_transpose(&m_inversion, -1);
        let m_frag = self.liquidate(&base_motif, 2);
        let m_seq1 = self.diatonic_transpose(&m_frag, 2);
        let m_seq2 = self.diatonic_transpose(&m_frag, 4);
        let m_retro = self.retrograde(&base_motif);
        let m_augmented = self.augment(&base_motif, 1.5);
        let mut m_heroic_climax = self.intervallic_expansion(&base_motif, 1.8);
        for n in &mut m_heroic_climax {
            n.velocity = 125;
            n.pitch += 12;
        }
        let m_liquidated = self.liquidate(&base_motif, 1);
        let m_final_tonic = vec![NoteEvent::new(self.root_note, 4.0, 70)];

        let phrase_sections: Vec<Vec<NoteEvent>> = vec![
            [m_statement, m_step_up].concat(),
            [m_inversion, m_cadence].concat(),
            [m_frag, m_seq1, m_seq2, m_retro].concat(),
            [m_augmented, m_heroic_climax].concat(),
            [m_liquidated, m_final_tonic].concat(),
        ];

        let mut full_melody: Vec<NoteEvent> = Vec::new();
        let mut current_beat = 0.0f64;
        for section in phrase_sections {
            for mut note in section {
                note.bar = current_beat;
                current_beat += note.duration;
                full_melody.push(note);
            }
        }

        let mut meta = Map::new();
        meta.insert("total_notes".into(), json!(full_melody.len()));
        meta.insert("total_beats".into(), json!(current_beat));
        meta.insert("estimated_bars".into(), json!(current_beat / 4.0));
        meta.insert(
            "climax_peak_beat".into(),
            json!(crate::common::py::round_py(current_beat * 0.618, 2)),
        );
        (Value::Object(meta), full_melody)
    }

    /// Zero-dependency Type-0 MIDI writer matching export_to_midi.
    pub fn export_to_midi(&self, notes: &[NoteEvent], output_filename: &str) {
        let ticks_per_quarter: i64 = 480;
        let mut track_data: Vec<u8> = Vec::new();

        let us_per_quarter = 60_000_000i64 / self.tempo;
        track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03]);
        track_data.extend_from_slice(&us_per_quarter.to_be_bytes()[5..8]);

        fn write_vlq(mut value: i64) -> Vec<u8> {
            let mut buf = vec![(value & 0x7F) as u8];
            value >>= 7;
            while value > 0 {
                buf.push(((value & 0x7F) | 0x80) as u8);
                value >>= 7;
            }
            buf.reverse();
            buf
        }

        let mut events: Vec<(i64, u8, i64, i64)> = Vec::new();
        for n in notes {
            let start_tick = (n.bar * ticks_per_quarter as f64) as i64;
            let end_tick = ((n.bar + n.duration) * ticks_per_quarter as f64) as i64;
            events.push((start_tick, 0x90, n.pitch, n.velocity));
            events.push((end_tick, 0x80, n.pitch, 0));
        }
        events.sort_by_key(|e| (e.0, if e.1 == 0x80 { 0 } else { 1 }));

        let mut current_tick = 0i64;
        for (tick, status, pitch, vel) in events {
            let delta = (tick - current_tick).max(0);
            current_tick = tick;
            track_data.extend_from_slice(&write_vlq(delta));
            track_data.extend_from_slice(&[status, (pitch & 0x7F) as u8, (vel & 0x7F) as u8]);
        }
        track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

        let mut out = Vec::new();
        out.extend_from_slice(b"MThd");
        out.extend_from_slice(&6u32.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&1u16.to_be_bytes());
        out.extend_from_slice(&(ticks_per_quarter as u16).to_be_bytes());
        out.extend_from_slice(b"MTrk");
        out.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
        out.extend_from_slice(&track_data);
        let _ = fs::write(output_filename, out);
        println!(
            "[SUCCESS] Exported {} notes to MIDI: {}",
            notes.len(),
            output_filename
        );
    }
}

/// Python `__main__`: Beethoven + Williams demo writing /tmp/*.mid.
pub fn run(argv: &[String]) -> i32 {
    // Optional CLI parity: motif-midi accepts seeds/output override.
    let args = cli::parse(
        "ciel-audio motif",
        argv,
        &[
            ArgSpec::value("root", None, "root").def("60"),
            ArgSpec::value("mode", None, "mode").def("dorian"),
            ArgSpec::value("tempo", None, "tempo").def("120"),
            ArgSpec::multi("seed-pitches", None, "seed-pitches"),
            ArgSpec::multi("seed-durations", None, "seed-durations"),
            ArgSpec::value("out", None, "out"),
        ],
    );

    if args.get("out").is_some() {
        let root = args.int("root", 60);
        let mode = args.get_or("mode", "dorian");
        let tempo = args.int("tempo", 120);
        let pitches: Vec<i64> = args
            .multi("seed-pitches")
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        let durations: Vec<f64> = args
            .multi("seed-durations")
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        let out = args.get_or("out", "motif.mid");
        let engine = MotifDNAGenerator::new(root, &mode, tempo);
        let (_, melody) = engine.generate_thematic_journey(&pitches, &durations);
        engine.export_to_midi(&melody, &out);
        return 0;
    }

    println!("======================================================================");
    println!("MOTIVIC DNA GENERATOR: MASTERWORK COMPOSITION ENGINE");
    println!("======================================================================");

    let beethoven_engine = MotifDNAGenerator::new(60, "aeolian", 108);
    let (b_meta, b_melody) =
        beethoven_engine.generate_thematic_journey(&[67, 67, 67, 63], &[0.5, 0.5, 0.5, 2.0]);
    println!(
        "Generated Beethoven Theme: {} notes across {} bars.",
        b_meta["total_notes"],
        crate::common::py::py_num(b_meta["estimated_bars"].as_f64().unwrap_or(0.0))
    );
    println!(
        "Climax Target: Beat {}",
        crate::common::py::py_num(b_meta["climax_peak_beat"].as_f64().unwrap_or(0.0))
    );
    beethoven_engine.export_to_midi(&b_melody, "/tmp/beethoven_motif_dna.mid");

    let williams_engine = MotifDNAGenerator::new(58, "mixolydian", 126);
    let (w_meta, w_melody) =
        williams_engine.generate_thematic_journey(&[58, 65, 70], &[1.0, 1.0, 2.0]);
    println!(
        "Generated Williams Theme: {} notes across {} bars.",
        w_meta["total_notes"],
        crate::common::py::py_num(w_meta["estimated_bars"].as_f64().unwrap_or(0.0))
    );
    williams_engine.export_to_midi(&w_melody, "/tmp/williams_motif_dna.mid");
    0
}
