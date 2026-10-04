//! Seeded-from-entropy PRNG matching Python `random`'s *distribution* API
//! (uniform, gauss, expovariate). Exact streams are unseeded in the Python
//! originals so byte-parity is neither possible nor required; distribution
//! shape is what the ports preserve.

use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rng {
    s: [u64; 4],
    gauss_spare: Option<f64>,
}

fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

impl Rng {
    pub fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9e3779b97f4a7c15);
        let mut seed = nanos ^ ((std::process::id() as u64) << 32) ^ 0xA0761D6478BD642F;
        let mut s = [0u64; 4];
        for slot in &mut s {
            *slot = splitmix64(&mut seed);
        }
        Rng {
            s,
            gauss_spare: None,
        }
    }

    /// Deterministic seed for tests and reproducible synthesis — same seed,
    /// same stream.
    pub fn from_seed(seed: u64) -> Self {
        let mut s = [0u64; 4];
        let mut k = seed;
        for slot in &mut s {
            *slot = splitmix64(&mut k);
        }
        Rng {
            s,
            gauss_spare: None,
        }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let (s0, s1, s2, s3) = (self.s[0], self.s[1], self.s[2], self.s[3]);
        let result = ((s1.wrapping_mul(5)) << 7 | (s1.wrapping_mul(5)) >> (64 - 7)).wrapping_mul(9);
        let t = s1 << 17;
        self.s[2] = s2 ^ s0;
        self.s[3] = s3 ^ s1;
        self.s[1] = s1 ^ s2;
        self.s[0] = s0 ^ s3;
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_right(19);
        result
    }

    /// uniform in [0.0, 1.0) — same as Python `random.random()`.
    pub fn random(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9007199254740992.0)
    }

    /// `random.uniform(a, b)` → a + (b-a)*random()
    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.random()
    }

    /// `random.gauss(mu, sigma)` — Box-Muller with spare cache, matching
    /// CPython's gauss shape.
    pub fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
        if let Some(z) = self.gauss_spare.take() {
            return mu + z * sigma;
        }
        let x2pi = self.random() * std::f64::consts::TAU;
        let g2rad = (-2.0 * (1.0 - self.random()).ln()).sqrt();
        let z = g2rad * x2pi.cos();
        self.gauss_spare = Some(g2rad * x2pi.sin());
        mu + z * sigma
    }

    /// `random.expovariate(lambd)`.
    pub fn expovariate(&mut self, lambd: f64) -> f64 {
        -((1.0 - self.random()).ln()) / lambd
    }

    pub fn gen_range_usize(&mut self, n: usize) -> usize {
        (self.random() * n as f64) as usize
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}
