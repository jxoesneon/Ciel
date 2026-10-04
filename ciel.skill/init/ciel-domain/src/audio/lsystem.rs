//! Port of `skills/procedural-audio/scripts/lsystem_schenker_generator.py` —
//! context-free Ursatz expansion → M(n) scale-degree melody.

pub struct LSystemSchenkerGenerator {
    pub scale: Vec<i64>,
}

impl LSystemSchenkerGenerator {
    pub fn new(scale: Vec<i64>) -> Self {
        LSystemSchenkerGenerator { scale }
    }

    pub fn expand_axiom(&self, axiom: &str, iterations: usize) -> String {
        let rule = |tok: &str| -> &str {
            match tok {
                "U" => "P1 P2 P3",
                "P1" => "N0 S1 N2",
                "P2" => "Z24 V4",
                "P3" => "Cad10",
                "N0" => "M(0) M(1) M(0)",
                "S1" => "M(2) M(4)",
                "N2" => "M(4) M(5) M(4)",
                "Z24" => "M(2) M(3) M(4)",
                "V4" => "M(4) M(6)",
                "Cad10" => "M(1) M(0)",
                _ => "",
            }
        };
        let mut current = axiom.to_string();
        for _ in 0..iterations {
            let mut next: Vec<String> = Vec::new();
            for token in current.split_whitespace() {
                let r = rule(token);
                if r.is_empty() {
                    next.push(token.to_string());
                } else {
                    next.push(r.to_string());
                }
            }
            current = next.join(" ");
        }
        current
    }

    pub fn string_to_melody(&self, lsys_str: &str, root_midi: i64) -> Vec<i64> {
        let mut melody = Vec::new();
        for token in lsys_str.split_whitespace() {
            if let Some(rest) = token.strip_prefix("M(") {
                if let Some(num) = rest.strip_suffix(')') {
                    if let Ok(deg) = num.parse::<i64>() {
                        let scale_len = self.scale.len() as i64;
                        let pitch = root_midi
                            + deg.div_euclid(scale_len) * 12
                            + self.scale[deg.rem_euclid(scale_len) as usize];
                        melody.push(pitch);
                    }
                }
            }
        }
        melody
    }

    pub fn generate_themed_phrase(&self, root_midi: i64, iterations: usize) -> Vec<i64> {
        let lsys = self.expand_axiom("U", iterations);
        self.string_to_melody(&lsys, root_midi)
    }
}

pub fn run(_argv: &[String]) -> i32 {
    let gen = LSystemSchenkerGenerator::new(vec![0, 2, 4, 5, 7, 9, 11]);
    let lsys_text = gen.expand_axiom("U", 3);
    println!("--- GENERATED L-SYSTEM STRING ---");
    println!("{}", lsys_text);

    let melody = gen.string_to_melody(&lsys_text, 60);
    println!("\n--- SYNTHESIZED SCHENKERIAN MELODY (MIDI Notes) ---");
    let items: Vec<String> = melody.iter().map(|m| m.to_string()).collect();
    println!("[{}]", items.join(", "));
    0
}
