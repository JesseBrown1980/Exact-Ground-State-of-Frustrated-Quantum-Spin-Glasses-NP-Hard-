/// 108-Lane Finite-Field Number Theoretic Transform (NTT) Prism
///
/// Implements spectral decomposition of coupling rings over a finite field F_q.
/// By projecting frustrated loops into the frequency domain across 108 parallel lanes,
/// destructive phase interference pinpoints topological obstruction nodes,
/// guiding the placement of neutral trits (0) to bypass binary local minima.

pub const NTT_LANES: usize = 108;
// Modulus q = 433, where 433 is prime and (433 - 1) = 432 = 4 * 108.
pub const MODULUS: u64 = 433;

pub struct NttPrism108 {
    pub modulus: u64,
    pub root: u64,
    pub inv_root: u64,
    pub roots_table: [u64; NTT_LANES],
    pub inv_roots_table: [u64; NTT_LANES],
}

impl NttPrism108 {
    pub fn new() -> Self {
        let q = MODULUS;
        // Find primitive 108-th root of unity in F_q
        // For q = 433, primitive root g = 5.
        // g^4 mod 433 has order 432 / 4 = 108.
        let g = Self::find_primitive_root(q);
        let root = Self::mod_pow(g, (q - 1) / (NTT_LANES as u64), q);
        let inv_root = Self::mod_inv(root, q);

        let mut roots_table = [0u64; NTT_LANES];
        let mut inv_roots_table = [0u64; NTT_LANES];

        let mut curr = 1u64;
        let mut curr_inv = 1u64;
        for i in 0..NTT_LANES {
            roots_table[i] = curr;
            inv_roots_table[i] = curr_inv;
            curr = (curr * root) % q;
            curr_inv = (curr_inv * inv_root) % q;
        }

        Self {
            modulus: q,
            root,
            inv_root,
            roots_table,
            inv_roots_table,
        }
    }

    fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
        let mut res = 1u64;
        base %= m;
        while exp > 0 {
            if (exp & 1) == 1 {
                res = (res * base) % m;
            }
            base = (base * base) % m;
            exp >>= 1;
        }
        res
    }

    fn mod_inv(n: u64, m: u64) -> u64 {
        Self::mod_pow(n, m - 2, m)
    }

    fn find_primitive_root(p: u64) -> u64 {
        let phi = p - 1;
        // Factors of 432: 2, 3
        for g in 2..p {
            if Self::mod_pow(g, phi / 2, p) != 1 && Self::mod_pow(g, phi / 3, p) != 1 {
                return g;
            }
        }
        5 // fallback known primitive root for 433
    }

    /// Forward 108-Lane NTT Transform: X_k = ∑_{n=0}^{107} x_n · ω^(n·k) (mod q)
    pub fn forward(&self, input: &[u64; NTT_LANES]) -> [u64; NTT_LANES] {
        let mut output = [0u64; NTT_LANES];
        let q = self.modulus;

        for k in 0..NTT_LANES {
            let mut sum = 0u64;
            for n in 0..NTT_LANES {
                let factor = self.roots_table[(n * k) % NTT_LANES];
                sum = (sum + input[n] * factor) % q;
            }
            output[k] = sum;
        }
        output
    }

    /// Inverse 108-Lane NTT Transform: x_n = (108)^(-1) ∑_{k=0}^{107} X_k · ω^(-n·k) (mod q)
    pub fn inverse(&self, input: &[u64; NTT_LANES]) -> [u64; NTT_LANES] {
        let mut output = [0u64; NTT_LANES];
        let q = self.modulus;
        let inv_n = Self::mod_inv(NTT_LANES as u64, q);

        for n in 0..NTT_LANES {
            let mut sum = 0u64;
            for k in 0..NTT_LANES {
                let factor = self.inv_roots_table[(n * k) % NTT_LANES];
                sum = (sum + input[k] * factor) % q;
            }
            output[n] = (sum * inv_n) % q;
        }
        output
    }

    /// Spectral Frustration Analysis:
    /// Maps a ring of couplings into F_q, computes the spectral power across the 108 lanes,
    /// and detects phase interference nulls indicating optimal topological mediator nodes.
    pub fn analyze_frustration_spectrum(&self, couplings: &[i32]) -> Vec<(usize, f64)> {
        let mut lane_input = [0u64; NTT_LANES];
        let q = self.modulus;

        for i in 0..NTT_LANES {
            let val = if i < couplings.len() { couplings[i] } else { 1 };
            // Map {-1, +1} into F_q: -1 -> q - 1, +1 -> 1
            lane_input[i] = if val < 0 { q - 1 } else { 1 };
        }

        let spectrum = self.forward(&lane_input);
        let mut nodal_scores = Vec::with_capacity(NTT_LANES);

        for (idx, &coeff) in spectrum.iter().enumerate() {
            // Measure spectral distance from zero phase
            let sym = if coeff > q / 2 { (q - coeff) as f64 } else { coeff as f64 };
            nodal_scores.push((idx, sym));
        }

        // Sort by interference nulls (lowest spectral energy indicates destructive nodal point)
        nodal_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        nodal_scores
    }
}
