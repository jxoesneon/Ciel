//! `difflib.SequenceMatcher.ratio()` port — Ratcliff/Obershelp gestalt
//! matching used by the UI/UX suggest-similar-terms path. Implemented on
//! byte-less char vectors; autojunk disabled like the Python call sites here
//! (SequenceMatcher(None, a, b) on short tokens).

use std::collections::HashMap;
use std::hash::Hash;

struct Matcher<'a, T: Hash + Eq> {
    a: &'a [T],
    #[allow(dead_code)]
    b: &'a [T],
    b2j: HashMap<&'a T, Vec<usize>>,
}

impl<'a, T: Hash + Eq> Matcher<'a, T> {
    fn new(a: &'a [T], b: &'a [T]) -> Self {
        let mut b2j: HashMap<&T, Vec<usize>> = HashMap::new();
        for (i, item) in b.iter().enumerate() {
            b2j.entry(item).or_default().push(i);
        }
        Matcher { a, b, b2j }
    }

    /// Longest contiguous match in a[alo..ahi], b[blo..bhi] → (i, j, size).
    fn find_longest_match(
        &self,
        alo: usize,
        ahi: usize,
        blo: usize,
        bhi: usize,
    ) -> (usize, usize, usize) {
        let mut best = (alo, blo, 0usize);
        // j2len[j] = length of longest run ending at position j for the
        // current a-window; rebuilt per i (difflib keeps it incremental).
        let mut j2len: HashMap<usize, usize> = HashMap::new();
        for i in alo..ahi {
            let mut new_j2len: HashMap<usize, usize> = HashMap::new();
            if let Some(indices) = self.b2j.get(&self.a[i]) {
                for &j in indices {
                    if j < blo {
                        continue;
                    }
                    if j >= bhi {
                        break;
                    }
                    let k = j2len.get(&(j.wrapping_sub(1))).copied().unwrap_or(0) + 1;
                    new_j2len.insert(j, k);
                    if k > best.2 {
                        // i+1-k / j+1-k — avoids the usize underflow of
                        // i-k+1 when k == i+1 (match starting at alo).
                        best = (i + 1 - k, j + 1 - k, k);
                    }
                }
            }
            j2len = new_j2len;
        }
        best
    }

    fn matching_blocks(
        &self,
        alo: usize,
        ahi: usize,
        blo: usize,
        bhi: usize,
        out: &mut Vec<(usize, usize, usize)>,
    ) {
        let (i, j, k) = self.find_longest_match(alo, ahi, blo, bhi);
        if k == 0 {
            return;
        }
        if alo < i && blo < j {
            self.matching_blocks(alo, i, blo, j, out);
        }
        out.push((i, j, k));
        if i + k < ahi && j + k < bhi {
            self.matching_blocks(i + k, ahi, j + k, bhi, out);
        }
    }
}

/// `SequenceMatcher(None, a, b).ratio()` on char sequences.
/// ratio = 2*M / (len(a)+len(b)); 1.0 when both empty.
pub fn ratio_chars(a: &[char], b: &[char]) -> f64 {
    let total = a.len() + b.len();
    if total == 0 {
        return 1.0;
    }
    let m = Matcher::new(a, b);
    let mut blocks = Vec::new();
    m.matching_blocks(0, a.len(), 0, b.len(), &mut blocks);
    let matches: usize = blocks.iter().map(|b| b.2).sum();
    2.0 * matches as f64 / total as f64
}

pub fn ratio(a: &str, b: &str) -> f64 {
    let av: Vec<char> = a.chars().collect();
    let bv: Vec<char> = b.chars().collect();
    ratio_chars(&av, &bv)
}
