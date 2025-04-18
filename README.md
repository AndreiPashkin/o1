# O1 Project

The O1 (as in _O(1)_) project focuses on implementing hashing schemes for perfect and
general-purpose hash tables.

## Roadmap to 1.0.0

- [ ] Basic universal hashing function families.
  - [x] The Dietzfelbinger multiply-shift family.
  - [x] The polynomial family.
  - [ ] Optimizations by introducing SIMD instructions.
  - [ ] Other optimizations.
- [ ] An alternative fast hashing algorithm.
- [x] The FKS perfect hashing scheme.
  - [ ] Compile-time construction.
  - [x] Run-time construction.
  - [ ] Hybrid construction (with multi-stage build).
- [ ] Benchmarking setup.
  - [ ] Against the `HashMap` from the standard library.
  - [ ] Against the `HashMap` with an alternative hasher.
  - [ ] Against `phf`.
  - [ ] Benchmarking of the hash functions.
- [ ] Implement the brute-force perfect hashing scheme that guarantees minimal lookup and construction times at
      the expense of increased memory usage.
- [ ] `no_std` support.
- [ ] `derive`-macro for auto-generation of library's hashers.

## Development

The following standards are followed to maintain the project:
- https://www.conventionalcommits.org/en/v1.0.0/
  - "dev" commit type to designate internal, non-user-facing features.
- https://semver.org/
- https://keepachangelog.com/en/1.1.0/
- https://adr.github.io/madr/

## Useful resources

- PHF - compile-time static maps based on perfect hashing for Rust:
  https://github.com/rust-phf/rust-phf
- Perfect hashing:
  - The original paper describing the FKS scheme: https://dl.acm.org/doi/10.1145/828.1884
  - PtrHash - a new scheme focused on lookup speed: https://curiouscoding.nl/posts/ptrhash-paper/
- Benchmarks:
  - Benchmark of minimal perfect hashing schemes: https://github.com/roberto-trani/mphf_benchmark
  - A very detailed benchmark that includes linear-probing, robin-hood and Cuckoo schemes:
    https://dl.acm.org/doi/10.14778/2850583.2850585
  - Another interesting comparison that includes Cuckoo, Hopscotch and linear probing schemes:
    http://jakubiuk.net/stuff/hash_tables_cache_performance.pdf
