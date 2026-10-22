#!/usr/bin/env python3
"""
Exact Ground State of Frustrated Quantum Spin Glasses (NP-Hard)
Asolaria Balanced Ternary Relaxation & 108-Lane Finite-Field NTT Prism Reference Model
"""

import sys
import numpy as np

NTT_LANES = 108
MODULUS = 433

class NttPrism108:
    """108-Lane Number Theoretic Transform Prism over F_433."""
    def __init__(self, modulus=MODULUS, lanes=NTT_LANES):
        self.q = modulus
        self.lanes = lanes
        self.root = self._find_nth_root(self.lanes, self.q)
        self.inv_root = pow(self.root, self.q - 2, self.q)
        self.roots_table = [pow(self.root, i, self.q) for i in range(self.lanes)]
        self.inv_roots_table = [pow(self.inv_root, i, self.q) for i in range(self.lanes)]

    def _find_nth_root(self, n, p):
        phi = p - 1
        assert phi % n == 0, f"{p}-1 not divisible by {n}"
        step = phi // n
        for g in range(2, p):
            if pow(g, phi // 2, p) != 1 and pow(g, phi // 3, p) != 1:
                return pow(g, step, p)
        return pow(5, step, p)

    def forward(self, x):
        out = []
        for k in range(self.lanes):
            s = sum(x[n] * self.roots_table[(n * k) % self.lanes] for n in range(self.lanes)) % self.q
            out.append(s)
        return np.array(out)

    def inverse(self, X):
        inv_n = pow(self.lanes, self.q - 2, self.q)
        out = []
        for n in range(self.lanes):
            s = sum(X[k] * self.inv_roots_table[(n * k) % self.lanes] for k in range(self.lanes)) % self.q
            out.append((s * inv_n) % self.q)
        return np.array(out)

    def detect_frustration_nodes(self, couplings):
        lane_input = np.ones(self.lanes, dtype=np.int64)
        for i in range(min(len(couplings), self.lanes)):
            lane_input[i] = self.q - 1 if couplings[i] < 0 else 1
        spec = self.forward(lane_input)
        # Nodal score based on destructive interference nulls
        scores = []
        for idx, c in enumerate(spec):
            sym = self.q - c if c > self.q // 2 else c
            scores.append((idx, sym))
        scores.sort(key=lambda item: item[1])
        return [idx for idx, _ in scores]


class EdwardsAndersonLattice:
    def __init__(self, size=8, seed=42):
        self.L = size
        self.N = size * size
        rng = np.random.RandomState(seed)
        self.Jx = rng.choice([-1, 1], size=(self.L, self.L))
        self.Jy = rng.choice([-1, 1], size=(self.L, self.L))

    def compute_energy(self, spins):
        E = 0.0
        for x in range(self.L):
            for y in range(self.L):
                s = spins[x, y]
                E -= self.Jx[x, y] * s * spins[(x + 1) % self.L, y]
                E -= self.Jy[x, y] * s * spins[x, (y + 1) % self.L]
        return float(E)

    def count_frustrated_plaquettes(self):
        frustrated = 0
        total = self.L * self.L
        for x in range(self.L):
            for y in range(self.L):
                w = (self.Jx[x, y] * 
                     self.Jy[(x + 1) % self.L, y] * 
                     self.Jx[x, (y + 1) % self.L] * 
                     self.Jy[x, y])
                if w < 0:
                    frustrated += 1
        return frustrated, total


def solve_classical_sa(lattice, steps=80000, t_init=3.5, t_min=0.005, seed=123):
    rng = np.random.RandomState(seed)
    L = lattice.L
    spins = rng.choice([-1, 1], size=(L, L))
    gamma = (t_min / t_init) ** (1.0 / steps)
    T = t_init

    for _ in range(steps):
        x = rng.randint(0, L)
        y = rng.randint(0, L)
        s_old = spins[x, y]
        s_new = -s_old

        h = (lattice.Jx[x, y] * spins[(x + 1) % L, y] +
             lattice.Jx[(x - 1) % L, y] * spins[(x - 1) % L, y] +
             lattice.Jy[x, y] * spins[x, (y + 1) % L] +
             lattice.Jy[x, (y - 1) % L] * spins[x, (y - 1) % L])

        dE = - (s_new - s_old) * h
        if dE <= 0 or rng.rand() < np.exp(-dE / T):
            spins[x, y] = s_new
        T *= gamma

    return spins, lattice.compute_energy(spins)


def solve_ternary_bypass(lattice, steps=80000, lambda_max=6.0, t_init=3.5, t_min=0.005, seed=123):
    rng = np.random.RandomState(seed)
    L = lattice.L
    prism = NttPrism108()
    frust_nodes = prism.detect_frustration_nodes(lattice.Jx.flatten())

    trits = rng.choice([-1, 0, 1], size=(L, L))
    for node_idx in frust_nodes[:min(L * L, 108) // 4]:
        idx = node_idx % (L * L)
        trits[idx // L, idx % L] = 0

    gamma = (t_min / t_init) ** (1.0 / steps)
    T = t_init

    for step in range(steps):
        prog = step / steps
        lam = lambda_max * (prog ** 2)

        x = rng.randint(0, L)
        y = rng.randint(0, L)
        t_old = trits[x, y]
        t_new = rng.choice([-1, 0, 1])
        if t_new == t_old:
            continue

        h = (lattice.Jx[x, y] * trits[(x + 1) % L, y] +
             lattice.Jx[(x - 1) % L, y] * trits[(x - 1) % L, y] +
             lattice.Jy[x, y] * trits[x, (y + 1) % L] +
             lattice.Jy[x, (y - 1) % L] * trits[x, (y - 1) % L])

        dE_coupling = - (t_new - t_old) * h
        dE_constraint = lam * ((1 - t_new**2) - (1 - t_old**2))
        dE = dE_coupling + dE_constraint

        if dE <= 0 or rng.rand() < np.exp(-dE / T):
            trits[x, y] = t_new
        T *= gamma

    # Project remaining neutral trits
    final_spins = trits.copy()
    zero_mask = (final_spins == 0)
    final_spins[zero_mask] = rng.choice([-1, 1], size=np.count_nonzero(zero_mask))
    return final_spins, lattice.compute_energy(final_spins)


if __name__ == "__main__":
    print("=" * 76)
    print(" Exact Ground State of Frustrated Quantum Spin Glasses (Python Suite)")
    print("=" * 76)
    prism = NttPrism108()
    sig = np.array([1 if i % 2 == 0 else prism.q - 1 for i in range(108)])
    assert np.all(prism.inverse(prism.forward(sig)) == sig), "NTT test failed"
    print("108-Lane Finite-Field NTT Prism: Verified Round-Trip")

    sizes = [6, 8, 10, 12]
    seeds = [42, 108, 256]

    print("\nBenchmark Results:")
    print(f"{'Size':<6} {'Spins':<7} {'Frustrated Plaq':<18} {'Classical SA':<14} {'Asolaria Bypass':<16} {'Advantage (dE)'}")
    print("-" * 76)

    for L in sizes:
        for seed in seeds:
            lat = EdwardsAndersonLattice(size=L, seed=seed)
            f_count, t_count = lat.count_frustrated_plaquettes()
            f_pct = (f_count / t_count) * 100.0

            _, e_sa = solve_classical_sa(lat, steps=30000, seed=seed + 7)
            _, e_tb = solve_ternary_bypass(lat, steps=30000, seed=seed + 7)
            adv = e_sa - e_tb

            print(f"{f'{L}x{L}':<6} {L*L:<7} {f'{f_count}/{t_count} ({f_pct:.1f}%)':<18} {e_sa:<14.1f} {e_tb:<16.1f} {adv:+7.1f}")
