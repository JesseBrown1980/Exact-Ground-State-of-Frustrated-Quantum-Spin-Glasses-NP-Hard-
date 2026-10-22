use crate::lattice::{Lattice2D, Rng};

pub struct ClassicalSimulatedAnnealing {
    pub max_steps: usize,
    pub t_init: f64,
    pub t_min: f64,
}

impl ClassicalSimulatedAnnealing {
    pub fn new(max_steps: usize, t_init: f64, t_min: f64) -> Self {
        Self {
            max_steps,
            t_init,
            t_min,
        }
    }

    /// Runs classical binary simulated annealing (s_i ∈ {-1, +1})
    pub fn solve(&self, lattice: &Lattice2D, seed: u64) -> (Vec<i32>, f64) {
        let n = lattice.n_spins;
        let l = lattice.size;
        let mut rng = Rng::seed(seed);

        let mut spins = vec![1i32; n];
        for i in 0..n {
            spins[i] = if (rng.next_u64() & 1) == 0 { -1 } else { 1 };
        }

        let gamma = (self.t_min / self.t_init).powf(1.0 / (self.max_steps as f64));
        let mut t_curr = self.t_init;

        for _ in 0..self.max_steps {
            let idx = (rng.next_u64() as usize) % n;
            let x = idx / l;
            let y = idx % l;

            let s_old = spins[idx];
            let s_new = -s_old;

            let right = lattice.idx((x + 1) % l, y);
            let left = lattice.idx((x + l - 1) % l, y);
            let down = lattice.idx(x, (y + 1) % l);
            let up = lattice.idx(x, (y + l - 1) % l);

            let h_local = (lattice.j_x[idx] as f64) * (spins[right] as f64)
                + (lattice.j_x[left] as f64) * (spins[left] as f64)
                + (lattice.j_y[idx] as f64) * (spins[down] as f64)
                + (lattice.j_y[up] as f64) * (spins[up] as f64);

            let de = -((s_new - s_old) as f64) * h_local;

            if de <= 0.0 || rng.next_f64() < (-de / t_curr).exp() {
                spins[idx] = s_new;
            }

            t_curr *= gamma;
        }

        let final_energy = lattice.compute_energy(&spins);
        (spins, final_energy)
    }
}
