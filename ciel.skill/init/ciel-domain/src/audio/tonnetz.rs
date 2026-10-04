//! Port of `skills/procedural-audio/scripts/neo_riemannian_tonnetz.py` —
//! P/L/R/S/N/H operators and BFS shortest path across the 24-triad torus.

use std::collections::{HashSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Triad {
    pub root: i64,
    pub is_major: bool,
}

impl Triad {
    pub fn new(root: i64, is_major: bool) -> Self {
        Triad {
            root: root.rem_euclid(12),
            is_major,
        }
    }
    pub fn pitch_classes(&self) -> (i64, i64, i64) {
        let third = if self.is_major {
            (self.root + 4) % 12
        } else {
            (self.root + 3) % 12
        };
        let fifth = (self.root + 7) % 12;
        (self.root, third, fifth)
    }
    pub fn name(&self) -> String {
        let names = [
            "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
        ];
        format!(
            "{}{}",
            names[self.root as usize],
            if self.is_major { "" } else { "m" }
        )
    }
}

pub fn op_p(t: Triad) -> Triad {
    Triad::new(t.root, !t.is_major)
}
pub fn op_l(t: Triad) -> Triad {
    if t.is_major {
        Triad::new(t.root + 4, false)
    } else {
        Triad::new(t.root + 8, true)
    }
}
pub fn op_r(t: Triad) -> Triad {
    if t.is_major {
        Triad::new(t.root + 9, false)
    } else {
        Triad::new(t.root + 3, true)
    }
}
pub fn op_s(t: Triad) -> Triad {
    op_l(op_p(op_r(t)))
}
pub fn op_h(t: Triad) -> Triad {
    op_l(op_p(op_l(t)))
}

/// BFS shortest transformation path — mirrors `find_shortest_path`.
pub fn find_shortest_path(start: Triad, goal: Triad) -> Vec<(String, Triad)> {
    let mut queue: VecDeque<Vec<(String, Triad)>> = VecDeque::new();
    queue.push_back(vec![("START".to_string(), start)]);
    let mut visited: HashSet<Triad> = HashSet::new();
    visited.insert(start);

    while let Some(path) = queue.pop_front() {
        let current = path.last().unwrap().1;
        if current == goal {
            return path;
        }
        for (op_name, op_func) in [
            ("P", op_p as fn(Triad) -> Triad),
            ("L", op_l),
            ("R", op_r),
            ("S", op_s),
            ("H", op_h),
        ] {
            let neighbor = op_func(current);
            if !visited.contains(&neighbor) {
                visited.insert(neighbor);
                let mut new_path = path.clone();
                new_path.push((op_name.to_string(), neighbor));
                queue.push_back(new_path);
            }
        }
    }
    Vec::new()
}

pub fn run(_argv: &[String]) -> i32 {
    let c_maj = Triad::new(0, true);
    let ab_min = Triad::new(8, false);
    let f_sharp = Triad::new(6, true);

    println!("--- NEO-RIEMANNIAN BASIC TRANSFORMATIONS ---");
    println!("C Major -> P: {}", op_p(c_maj).name());
    println!("C Major -> L: {}", op_l(c_maj).name());
    println!("C Major -> R: {}", op_r(c_maj).name());
    println!("C Major -> S (Slide): {}", op_s(c_maj).name());
    println!("C Major -> H (Hexatonic Pole): {}", op_h(c_maj).name());

    println!("\n--- SHORTEST TONNETZ MODULATION PATH: C Major -> Ab Minor ---");
    let path = find_shortest_path(c_maj, ab_min);
    let segs: Vec<String> = path
        .iter()
        .map(|(op, t)| format!("{}({})", op, t.name()))
        .collect();
    println!("{}", segs.join(" -> "));

    println!("\n--- SHORTEST TONNETZ MODULATION PATH: C Major -> F# Major (Tritone) ---");
    let path2 = find_shortest_path(c_maj, f_sharp);
    let segs2: Vec<String> = path2
        .iter()
        .map(|(op, t)| format!("{}({})", op, t.name()))
        .collect();
    println!("{}", segs2.join(" -> "));
    0
}
