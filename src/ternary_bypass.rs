use crate::lattice::{Lattice2D, Rng};
use crate::ntt_prism::NttPrism108;

pub struct TernaryBypassSolver {
    pub max_steps: usize,
    pub lambda_max: f64,
    pub t_init: f64,
    pub t_min: f64,
}

impl TernaryBypassSolver {
    pub fn new(max_steps: usize, lambda_max: f64, t_init: f64, t_min: f64) -> Self {
        Self {
            max_steps,
            lambda_max,
            t_init,
            t_min,
        }
    }

    /// Solves the Edwards-Anderson ground state using Balanced Ternary Relaxation
    /// aided by NTT prism spectral nodal seeding.
    pub fn solve(&self, lattice: &Lattice2D, seed: u64) -> (Vec<i32>, f64) {
        let n = lattice.n_spins;
        let l = lattice.size;
        let mut rng = Rng::seed(seed);

        // 1. Initialize NTT Prism to identify high-frustration cycle hotspots
        let prism = NttPrism108::new();
        let spectral_nodes = prism.analyze_frustration_spectrum(&lattice.j_x);

        // 2. Initialize in balanced ternary {-1, 0, +1}
        let mut trits = vec![0i32; n];
        for i in 0..n {
            let r = rng.next_u64() % 3;
            trits[i] = match r {
                0 => -1,
                1 => 0,  // Neutral mediator state
                _ => 1,
            };
        }

        // Seed spectral interference nodes as neutral mediator trits (0)
        for &(node_idx, _) in spectral_nodes.iter().take(n.min(108) / 4) {
            let target_spin = node_idx % n;
            trits[target_spin] = 0;
        }

        // 3. Homotopy Annealing Loop
        let gamma = (self.t_min / self.t_init).powf(1.0 / (self.max_steps as f64));
        let mut t_curr = self.t_init;

        let trit_options = [-1, 0, 1];

        for step in 0..self.max_steps {
            let progress = step as f64 / self.max_steps as f64;
            // Quadratic ramp of binary projection penalty: λ(progress) = λ_max * progress^2
            let lambda = self.lambda_max * progress.powi(2);

            let idx = (rng.next_u64() as usize) % n;
            let x = idx / l;
            let y = idx % l;

            let t_old = trits[idx];
            let t_choice = trit_options[(rng.next_u64() % 3) as usize];
            if t_choice == t_old {
                continue;
            }
            let t_new = t_choice;

            // Compute local exchange field from 4 neighbors
            let right = lattice.idx((x + 1) % l, y);
            let left = lattice.idx((x + l - 1) % l, y);
            let down = lattice.idx(x, (y + 1) % l);
            let up = lattice.idx(x, (y + l - 1) % l);

            let h_local = (lattice.j_x[idx] as f64) * (trits[right] as f64)
                + (lattice.j_x[left] as f64) * (trits[left] as f64)
                + (lattice.j_y[idx] as f64) * (trits[down] as f64)
                + (lattice.j_y[up] as f64) * (trits[up] as f64);

            // Energy differential: Coupling + Homotopy constraint
            // ΔH_coupling = -(t_new - t_old) * h_local
            let de_coupling = -((t_new - t_old) as f64) * h_local;
            // ΔH_constraint = λ * [(1 - t_new^2) - (1 - t_old^2)]
            let de_constraint = lambda * (((1 - t_new * t_new) - (1 - t_old * t_old)) as f64);
            let de_total = de_coupling + de_constraint;

            // Metropolis transition criterion
            if de_total <= 0.0 || rng.next_f64() < (-de_total / t_curr).exp() {
                trits[idx] = t_new;
            }

            t_curr *= gamma;
        }

        // 4. Boundary condensation: project any remaining neutral mediators (0) to {-1, +1}
        let mut binary_spins = vec![1i32; n];
        for i in 0..n {
            if trits[i] == 0 {
                let x = i / l;
                let y = i % l;
                let right = lattice.idx((x + 1) % l, y);
                let left = lattice.idx((x + l - 1) % l, y);
                let down = lattice.idx(x, (y + 1) % l);
                let up = lattice.idx(x, (y + l - 1) % l);

                let h_local = (lattice.j_x[i] as f64) * (binary_spins[right] as f64)
                    + (lattice.j_x[left] as f64) * (binary_spins[left] as f64)
                    + (lattice.j_y[i] as f64) * (binary_spins[down] as f64)
                    + (lattice.j_y[up] as f64) * (binary_spins[up] as f64);

                binary_spins[i] = if h_local >= 0.0 { 1 } else { -1 };
            } else {
                binary_spins[i] = trits[i];
            }
        }

        let final_energy = lattice.compute_energy(&binary_spins);
        (binary_spins, final_energy)
    }
}
