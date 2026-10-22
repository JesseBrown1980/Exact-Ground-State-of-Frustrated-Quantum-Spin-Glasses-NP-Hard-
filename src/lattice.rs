/// Edwards-Anderson Spin Glass Lattice model.
///
/// Models 2D square and 3D cubic lattices with random nearest-neighbor
/// anti-ferromagnetic couplings J_ij ∈ {-1, +1}.

pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn seed(seed: u64) -> Self {
        let mut rng = Self {
            s: [0x123456789abcdef0, 0x0fedcba987654321, 0x13579bdf2468ace0, 0x2468ace013579bdf],
        };
        // SplitMix64 initialization
        let mut z = seed.wrapping_add(0x9E3779B97F4A7C15);
        for i in 0..4 {
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            rng.s[i] = z ^ (z >> 31);
            z = z.wrapping_add(0x9E3779B97F4A7C15);
        }
        rng
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let result = (self.s[0].wrapping_add(self.s[3]))
            .rotate_left(23)
            .wrapping_add(self.s[0]);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    #[inline]
    pub fn choose_coupling(&mut self) -> i32 {
        if (self.next_u64() & 1) == 0 { -1 } else { 1 }
    }
}

#[derive(Clone)]
pub struct Lattice2D {
    pub size: usize,
    pub n_spins: usize,
    pub j_x: Vec<i32>,
    pub j_y: Vec<i32>,
}

impl Lattice2D {
    pub fn new(size: usize, seed: u64) -> Self {
        let n_spins = size * size;
        let mut rng = Rng::seed(seed);
        let mut j_x = Vec::with_capacity(n_spins);
        let mut j_y = Vec::with_capacity(n_spins);

        for _ in 0..n_spins {
            j_x.push(rng.choose_coupling());
            j_y.push(rng.choose_coupling());
        }

        Self {
            size,
            n_spins,
            j_x,
            j_y,
        }
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        x * self.size + y
    }

    /// Evaluates the Hamiltonian for a configuration of spins/trits:
    /// H = - ∑_<i,j> J_ij * s_i * s_j
    pub fn compute_energy(&self, spins: &[i32]) -> f64 {
        let mut energy = 0.0;
        let l = self.size;

        for x in 0..l {
            let next_x = (x + 1) % l;
            for y in 0..l {
                let next_y = (y + 1) % l;
                let i = self.idx(x, y);
                let s_i = spins[i] as f64;

                let right = self.idx(next_x, y);
                let down = self.idx(x, next_y);

                energy -= (self.j_x[i] as f64) * s_i * (spins[right] as f64);
                energy -= (self.j_y[i] as f64) * s_i * (spins[down] as f64);
            }
        }
        energy
    }

    /// Counts frustrated plaquettes via Wilson loop product:
    /// Φ(P) = ∏_{(i,j) ∈ ∂P} sgn(J_ij)
    /// A plaquette is frustrated if Φ(P) = -1.
    pub fn count_frustrated_plaquettes(&self) -> (usize, usize) {
        let l = self.size;
        let total_plaquettes = l * l;
        let mut frustrated = 0;

        for x in 0..l {
            let next_x = (x + 1) % l;
            for y in 0..l {
                let next_y = (y + 1) % l;
                let top = self.j_x[self.idx(x, y)];
                let right = self.j_y[self.idx(next_x, y)];
                let bottom = self.j_x[self.idx(x, next_y)];
                let left = self.j_y[self.idx(x, y)];

                let loop_product = top * right * bottom * left;
                if loop_product < 0 {
                    frustrated += 1;
                }
            }
        }
        (frustrated, total_plaquettes)
    }
}
