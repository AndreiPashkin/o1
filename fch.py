# fch_faithful.py — FCH minimal perfect hashing (papers-faithful naming + rich docstrings)
# -----------------------------------------------------------------------------
# This implementation follows Fox–Chen–Heath (’92) and the SIGIR’21 restatement
# of FCH’s “hash & displace” MOS framework. Symbols are aligned with the papers:
#   - Mapping (MAP): h10, h11, h12 ; uneven split via p1≈0.6n, p2≈0.3m
#   - Ordering (ORD): sort buckets by non-increasing size
#   - Searching (SRCH): per-bucket pilot ⟨d_i,b_i⟩ with h20(·, s2+b_i) + d_i (mod n)
#
# Why buckets and pilots?
#   FCH turns MPHF construction into a search over small “per-bucket” choices
#   instead of over all keys at once. The pilot for bucket B_i stores enough
#   information to deterministically place the keys of B_i into free slots; all
#   pilots together define the MPHF (one probe to read ⟨d_i,b_i⟩). :contentReference[oaicite:0]{index=0}
#
# Key references inside docstrings:
#   - MOS framework and goals: FCH’92 §4, figs. 4–5; SIGIR’21 §3.
#   - Search step (pattern, rotation, auxiliary index): FCH’92 §4.3–4.3.1. :contentReference[oaicite:2]{index=2}
#   - p1/p2 skew and pilot packing for single read: SIGIR’21 §3 (Form. (1), discussion). :contentReference[oaicite:3]{index=3}
# -----------------------------------------------------------------------------

from math import ceil, log2
import random
import unittest
from typing import Iterable, List, Tuple, Set

MASK64 = (1 << 64) - 1


# -----------------------------
# 64-bit hashing primitives
# -----------------------------

def mix64(x: int, seed: int) -> int:
    """
    Pseudo-random 64→64 mixer (non-crypto) used to instantiate the PRFs h(·,seed).

    Rationale (papers level):
    FCH assumes a pseudo-random hash family h(·, s) to (i) map keys into the
    mapping/address spaces and (ii) realize the search phase. We use a small,
    fast avalanche mixer (two 64-bit odd multipliers and two xorshift folds)
    to diffuse bits before multiply-high range reduction. FCH itself is agnostic
    to the exact mixer as long as outputs are sufficiently uniform/independent. :contentReference[oaicite:4]{index=4}

    Steps (all mod 2^64):
      1) XOR with seed → inject seed-dependent randomness
      2) * 0x9E3779B185EBCA87 → spread bits across lanes
      3) ^ (>>32) → fold high entropy downward
      4) * 0xC2B2AE3D27D4EB4F → additional avalanche
      5) ^ (>>29) → final diffusion

    (This is not cryptographic; swap in a stronger PRF if your threat model requires it.)
    """
    x = (x ^ seed) & MASK64
    x = (x * 0x9E3779B185EBCA87) & MASK64
    x ^= (x >> 32)
    x = (x * 0xC2B2AE3D27D4EB4F) & MASK64
    x ^= (x >> 29)
    return x & MASK64


def fastrange(x: int, m: int) -> int:
    """
    Multiply-high range reduction: map a “random-looking” 64-bit integer x to [0..m).

    Why:
      FCH and the SIGIR’21 restatement rely on uniform-ish hashing to map to
      [n] or [m]. Multiply-high reduction (taking the high half of x*m) is a
      fast, bias-resistant way to get a number in [0..m) without division. (General
      technique; not specific to FCH.)
    """
    return ((x & MASK64) * (m & MASK64) >> 64) % m


# -----------------------------
# FCH minimal perfect hash
# -----------------------------

class FCH:
    """
    Minimal Perfect Hash Function (MPHF) via FCH (MOS framework).

    Overview (faithful to papers):
      - INPUT: key set S, |S| = n. We target a minimal table T with |T| = n (MPHF). :contentReference[oaicite:5]{index=5}
      - PARAMETERS:
          m = ⌈ c · n / (log2 n + 1) ⌉ buckets (b in FCH’92) to store pilots;
              choosing c (≈ 2.6–4+) trades build time for space ≈ c·n bits. :contentReference[oaicite:6]{index=6}
          p1 ≈ 0.6n, p2 ≈ 0.3m for uneven mapping split (heavy/light). :contentReference[oaicite:7]{index=7}
          s1: mapping seed for h10/h11/h12 ; s2: search seed for h20 (reseeds on failure). :contentReference[oaicite:8]{index=8}
      - MOS:
          (i) MAP: map keys into non-uniform buckets with the p1/p2 split;
          (ii) ORDER: sort buckets by non-increasing size; :contentReference[oaicite:9]{index=9}
          (iii) SEARCH: per bucket B_i find pilot ⟨d_i,b_i⟩ so that positions
                ( h20(k, s2+b_i) + d_i ) mod n are free. Pack pilot as v_i = (d_i<<1)|b_i
                to enable one-read lookup. :contentReference[oaicite:10]{index=10}

    Lookup (single pilot read):
      i = bucket(k); v = P[i]; b = v & 1; d = v >> 1;
      return ( h20(k, s2 + b) + d ) mod n.  (SIGIR’21 §3, discussion/psuedocode) :contentReference[oaicite:11]{index=11}
    """

    # ---------- Public API ----------

    def __init__(
            self,
            keys: Iterable[int],
            c: float = 3.0,
            seed: int = 123456789,
            p1_frac: float = 0.60,  # default split: ~60% keys in heavy group (S1)
            p2_frac: float = 0.30,  # default split: ~30% buckets assigned to heavy
    ):
        """
        Build the MPHF for the given set S of keys.

        Args:
          keys: Iterable of integers (hash non-integers to 64-bit upstream).
          c:    Controls m (bucket count) via m = ⌈ c·n / (log2 n + 1) ⌉;
                larger c ⇒ smaller buckets ⇒ faster SEARCH but ≈ c·n bits in pilots. :contentReference[oaicite:12]{index=12}
          seed: Base seed to derive s1 (mapping) and s2 (search).
          p1_frac, p2_frac: the (paper’s) heuristic split p1≈0.6n, p2≈0.3m for uneven buckets. :contentReference[oaicite:13]{index=13}

        Notes:
          - We keep only lookup essentials: n, m, p1, p2, s1, s2, pilots.
          - Keys are NOT stored; the build is “single-shot” per S. :contentReference[oaicite:14]{index=14}
        """
        # mapping seed (h10/h11/h12)
        self.s1 = (mix64(seed, 0xA5A5A5A5A5A5A5A5) ^ seed) & MASK64
        # search seed (h20), may be reseeded during SEARCH
        s2_init = (mix64(seed, 0xC3C3C3C3C3C3C3C3) ^ (seed << 1)) & MASK64

        n, m, p1, p2, s2_final, pilots = self._build(
            keys=[int(k) & MASK64 for k in keys],
            c=max(c, 2.6),
            s1=self.s1,
            s2=s2_init,
            p1_frac=p1_frac,
            p2_frac=p2_frac,
        )

        self.n = n
        self.m = m
        self.p1 = p1
        self.p2 = p2
        self.s2 = s2_final
        self.pilots: List[int] = pilots

    def index(self, key: int) -> int:
        """
        Evaluate the MPHF for a key k ∈ S:

          i = bucket(k)
          v = pilots[i]; b = v & 1; d = v >> 1
          return ( h20(k, s2 + b) + d ) mod n

        This is the papers’ one-read lookup using the interleaved (d_i, b_i) encoding
        suggested in the SIGIR’21 restatement. :contentReference[oaicite:15]{index=15}
        """
        k = int(key) & MASK64
        i = self.bucket(k, self.n, self.m, self.p1, self.p2, self.s1)
        v = self.pilots[i]
        b = v & 1
        d = v >> 1
        return (self.h20_to_n(k, self.n, self.s2, b) + d) % self.n

    def permuted_positions(self, keys: Iterable[int]) -> List[int]:
        """
        Convenience helper for tests/validation: map a batch of keys to positions.
        Expected to be a permutation of [0..n-1] for the construction set S.
        """
        return [self.index(k) for k in keys]

    # ---------- Paper-faithful hash helpers ----------

    @staticmethod
    def h10_to_n(k: int, n: int, s1: int) -> int:
        """
        h10 : S → [0..n-1] — coarse mapping used to split S at threshold p1.

        Paper context:
          The mapping stage first hashes keys into [n] (h10). This enables the
          uneven split (≈60/40) and then remaps to buckets [m] (h11/h12). :contentReference[oaicite:16]{index=16}
        """
        return fastrange(mix64(k, s1), n)

    @staticmethod
    def h11_map(t: int, p2: int, s1: int) -> int:
        """
        h11 : [0..p1-1] → [0..p2-1] — heavy-side buckets.

        Paper context:
          If h10(k) < p1, the key is sent into the first p2 buckets (“heavy”).
          This concentrates ≈60% of keys into ≈30% of buckets. :contentReference[oaicite:17]{index=17}
        """
        return fastrange(mix64(t ^ 0x9E3779B97F4A7C15, s1), p2)

    @staticmethod
    def h12_map(t: int, m_minus_p2: int, s1: int) -> int:
        """
        h12 : [p1..n-1] → [0..m-p2-1] — light-side buckets (offset added by caller).

        Paper context:
          Keys in the complement (≈40%) are spread across the remaining ≈70%
          of buckets to keep average sizes small and simplify SEARCH. :contentReference[oaicite:18]{index=18}
        """
        return fastrange(mix64(t ^ 0xC2B2AE3D27D4EB4F, s1), m_minus_p2)

    @classmethod
    def bucket(cls, k: int, n: int, m: int, p1: int, p2: int, s1: int) -> int:
        """
        Compute the bucket index for k:

            t = h10(k)
            if t < p1: return h11(t)
            else:      return p2 + h12(t)

        Paper context:
          This is the unequal mapping (≈60%→≈30% buckets) before ORDER/SEARCH. :contentReference[oaicite:19]{index=19}
        """
        t = cls.h10_to_n(k, n, s1)
        if t < p1:
            return cls.h11_map(t, p2, s1)
        return p2 + cls.h12_map(t, m - p2, s1)

    @staticmethod
    def h20_to_n(k: int, n: int, s2: int, bbit: int) -> int:
        """
        h20 : S × {0,1} → [0..n-1] — search-stage hash with designated bit.

        Paper context:
          The designated bit b_i toggles the seed between s2 and s2+1 to
          enforce in-bucket injectivity. If a bucket collides for both b=0 and
          b=1, reseed s2 and retry — preserving the ORDER of buckets. :contentReference[oaicite:20]{index=20}
        """
        return fastrange(mix64(k, (s2 + (bbit & 1)) & MASK64), n)

    # ---------- Build (Map -> Order -> Search) ----------

    def _build(
            self,
            keys: List[int],
            c: float,
            s1: int,
            s2: int,
            p1_frac: float,
            p2_frac: float,
    ) -> Tuple[int, int, int, int, int, List[int]]:
        """
        Run the MOS pipeline using locals; return lookup parameters.

        Returns:
          (n, m, p1, p2, s2_final, pilots[]) where pilots[i] = (d_i<<1)|b_i.

        Paper context:
          - m ≈ c·n/(log2 n + 1): pilots store ⌈log2 n⌉ + 1 bits each, so total pilot
            bits ≈ m(⌈log2 n⌉+1) ≈ c·n. (Space ≈ c bits/key.) :contentReference[oaicite:21]{index=21}
          - ORDER by non-increasing size to improve SEARCH success rates. :contentReference[oaicite:22]{index=22}
          - SEARCH uses per-bucket injectivity (via b_i) and rotation by d_i. :contentReference[oaicite:23]{index=23}
        """
        n = len(keys)
        if n == 0:
            raise ValueError("FCH requires at least 1 key")

        # Choose m so pilot bits ≈ c·n (each pilot ≈ ⌈log2 n⌉+1 bits).
        m = max(1, int(ceil(c * n / (log2(n) + 1.0))))

        # Split thresholds p1≈0.6n, p2≈0.3m (rounded/clamped)
        p1 = max(1, min(n - 1, int(p1_frac * n)))
        p2 = max(1, min(m - 1, int(p2_frac * m)))

        # MAP: assign keys to buckets with the skew
        buckets = self._map_buckets(keys, n, m, p1, p2, s1)

        # ORDER: largest → smallest
        order = self._order_buckets(buckets)

        # SEARCH: compute pilots ⟨d_i,b_i⟩ (reseed s2 if needed)
        s2_final, pilots = self._search_pilots(buckets, order, n, m, s1, s2)

        return n, m, p1, p2, s2_final, pilots

    def _map_buckets(self, keys: List[int], n: int, m: int, p1: int, p2: int, s1: int) -> List[List[int]]:
        """
        MAP stage: assign each key to a bucket using the uneven split.

        Paper context:
          The mapping compresses the search space from n choices to m bucket
          choices and gives SEARCH fewer (per-bucket) degrees of freedom. The
          uneven split improves practical performance by creating a handful of
          larger buckets and many small ones (handled later quickly). :contentReference[oaicite:24]{index=24}
        """
        buckets: List[List[int]] = [[] for _ in range(m)]
        for k in keys:
            i = self.bucket(k, n, m, p1, p2, s1)
            buckets[i].append(k)
        return buckets

    @staticmethod
    def _order_buckets(buckets: List[List[int]]) -> List[int]:
        """
        ORDER stage: sort buckets by non-increasing size (ties by id).

        Paper context:
          Process the largest buckets first; they’re the most constrained and
          benefit most from an early, emptier table (higher success chance). :contentReference[oaicite:25]{index=25}
        """
        order = list(range(len(buckets)))
        order.sort(key=lambda i: (-len(buckets[i]), i))
        return order

    def _search_pilots(self, buckets: List[List[int]], order: List[int],
                       n: int, m: int, s1: int, s2: int) -> Tuple[int, List[int]]:
        """
        SEARCH stage: for each B_{o,i} (in ordered sequence), find a pilot ⟨d_i,b_i⟩.

        Mechanics and data structures (paper-accurate):
          - Ensure in-bucket injectivity with a designated bit b_i ∈ {0,1}:
            try b=0; if collision(s), try b=1; if still colliding, reseed s2 and restart. :contentReference[oaicite:26]{index=26}
          - Maintain an auxiliary free-slot index (`randomTable`, `mapTable`, `taken`):
            choose a random free slot x, align pattern element u∈P_i to x via
            d=(x−u) mod n, then check/commit rotation. This accelerates search. :contentReference[oaicite:27]{index=27}

        Reseeding policy:
          If injectivity fails for some bucket under both b∈{0,1}, or a bucket
          cannot be placed with any d, we reseed s2 and restart SEARCH; ORDER
          and mapping stay unchanged. (FCH heuristic.) :contentReference[oaicite:28]{index=28}
        """
        max_reseeds = 64
        reseeds = 0

        while True:
            random_table, map_table = self._make_random_permutation(n, s1, s2)
            taken = [False] * n
            filled = 0
            pilots = [0] * m

            failed_bucket = None
            for i in order:
                ks = buckets[i]
                if not ks:
                    pilots[i] = 0
                    continue

                placed_bucket = False
                for bbit in (0, 1):
                    if self._has_intra_collision(ks, n, s2, bbit):
                        continue

                    pattern = [self.h20_to_n(k, n, s2, bbit) for k in ks]

                    trial_random = list(random_table)
                    trial_map = list(map_table)
                    trial_taken = list(taken)

                    placed, d, trial_random, trial_map, trial_taken, trial_filled = self._place_bucket(
                        pattern, trial_random, trial_map, trial_taken, filled, n
                    )
                    if not placed:
                        continue

                    random_table = trial_random
                    map_table = trial_map
                    taken = trial_taken
                    filled = trial_filled
                    pilots[i] = (d << 1) | (bbit & 1)
                    placed_bucket = True
                    break

                if not placed_bucket:
                    failed_bucket = i
                    break

            if failed_bucket is None:
                return s2, pilots

            reseeds += 1
            if reseeds > max_reseeds:
                raise RuntimeError("Too many reseeds while placing; increase c or change seed")
            s2 = mix64(s2, 0x94D049BB133111EB)

    @staticmethod
    def _make_random_permutation(n: int, s1: int, s2: int) -> Tuple[List[int], List[int]]:
        """
        Build-time auxiliary: make a random permutation of [0..n-1] and its inverse.

        Paper context:
          FCH recommends an auxiliary index to quickly find/align free slots:
          a random permutation (`randomTable`) and its reverse (`mapTable`).
          Align u∈P_i to some free x by d=(x−u) mod n; test rotation; on success,
          mark positions taken and move them to the front region [0..filled). :contentReference[oaicite:29]{index=29}
        """
        rng = random.Random(s1 ^ (n << 1) ^ s2)
        random_table = list(range(n))
        rng.shuffle(random_table)
        map_table = [0] * n
        for idx, pos in enumerate(random_table):
            map_table[pos] = idx
        return random_table, map_table

    def _has_intra_collision(self, ks: List[int], n: int, s2: int, b: int) -> bool:
        """
        Return True if ∃ k1 ≠ k2 in the bucket with
          h20_to_n(k1, n, s2, b) == h20_to_n(k2, n, s2, b).

        Paper context:
          “If two keys in the bucket already collide under h20(·, s2+b_i), no
          displacement can separate them — they would move together.” (paraphr.)
          Thus injectivity is a prerequisite before searching for d_i. :contentReference[oaicite:31]{index=31}
        """
        seen: Set[int] = set()
        for k in ks:
            u = self.h20_to_n(k, n, s2, b)
            if u in seen:
                return True
            seen.add(u)
        return False

    @staticmethod
    def _place_bucket(pattern: List[int],
                      random_table: List[int],
                      map_table: List[int],
                      taken: List[bool],
                      filled: int,
                      n: int) -> Tuple[bool, int, List[int], List[int], List[bool], int]:
        """
        Try to place a bucket by *rotating its pattern* onto free slots.

        Mechanism (paper-accurate, FCH’92 §4.3–4.3.1):
          Let pattern P_i = {u_j}. Choose a free slot x from random_table[filled..].
          Set d = (x − u0) mod n (align a representative u0 ∈ P_i to x). Then test
          all (u_j + d) mod n are free. If so, *commit*: mark each used position as
          taken and swap it into the front of random_table (update map_table). :contentReference[oaicite:32]{index=32}

        Returns:
          (placed?, d, random_table, map_table, taken, filled)
        """
        u0 = pattern[0]
        for idx in range(filled, n):
            x = random_table[idx]
            d = (x - u0) % n
            rotated = [(u + d) % n for u in pattern]
            if any(taken[pos] for pos in rotated):
                continue

            # Commit rotation: mark and move to the front region [0..filled)
            for pos in rotated:
                taken[pos] = True
                j = map_table[pos]
                b = random_table[filled]
                random_table[j], random_table[filled] = b, pos
                map_table[b] = j
                map_table[pos] = filled
                filled += 1

            return True, d, random_table, map_table, taken, filled

        return False, 0, random_table, map_table, taken, filled


# -----------------------------
# Unit tests (sanity checks)
# -----------------------------

class TestFCH(unittest.TestCase):
    """Basic sanity checks for the FCH MPHF (not exhaustive)."""

    def test_permutation_property_medium(self):
        """
        For a moderate n, the produced positions from the build set S must be
        a permutation of [0..n-1] (by definition of MPHF). :contentReference[oaicite:33]{index=33}
        """
        rng = random.Random(7)
        n = 2000
        keys = {rng.getrandbits(64) for _ in range(n)}
        fch = FCH(keys, c=3.5, seed=12345)
        pos = fch.permuted_positions(keys)
        self.assertEqual(len(set(pos)), n, "positions are not unique")
        self.assertEqual(min(pos), 0)
        self.assertEqual(max(pos), n - 1)

    def test_determinism_same_seed(self):
        """
        With identical keys and seed/params, construction is deterministic
        (same pilots, same mapping), since all pseudorandomness is seeded.
        """
        rng = random.Random(13)
        n = 800
        keys = [rng.getrandbits(64) for _ in range(n)]
        fch1 = FCH(keys, c=3.2, seed=999)
        fch2 = FCH(keys, c=3.2, seed=999)
        self.assertEqual(fch1.pilots, fch2.pilots)
        self.assertEqual([fch1.index(k) for k in keys],
                         [fch2.index(k) for k in keys])

    def test_small_n_edges(self):
        """
        Edge sanity for tiny sets; the mapping must still be a permutation
        (n=1 trivial; small n ensures no corner-case regressions).
        """
        # n = 1
        keys = [0x1234]
        fch = FCH(keys, c=3.0, seed=1)
        pos = fch.permuted_positions(keys)
        self.assertEqual(pos, [0])

        # n = 16
        keys = list(range(16))
        fch = FCH(keys, c=3.0, seed=2)
        pos = fch.permuted_positions(keys)
        self.assertEqual(len(set(pos)), 16)
        self.assertEqual(min(pos), 0)
        self.assertEqual(max(pos), 15)

    def test_p_split_alignment(self):
        """
        The computed p1,p2 reflect the requested split fractions (clamped).
        SIGIR’21 suggests p1≈0.6n, p2≈0.3m. :contentReference[oaicite:34]{index=34}
        """
        rng = random.Random(23)
        n = 500
        keys = [rng.getrandbits(64) for _ in range(n)]
        p1_frac, p2_frac = 0.6, 0.3
        fch = FCH(keys, c=3.0, seed=12, p1_frac=p1_frac, p2_frac=p2_frac)

        expected_p1 = max(1, min(n - 1, int(p1_frac * n)))
        m = fch.m
        expected_p2 = max(1, min(m - 1, int(p2_frac * m)))

        self.assertEqual(fch.p1, expected_p1)
        self.assertEqual(fch.p2, expected_p2)


# -----------------------------
# Test runner
# -----------------------------

if __name__ == "__main__":
    unittest.main(verbosity=2)
