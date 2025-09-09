# O1 Project — Guide for LLM Agents

This document orients LLM-based agents (Claude Code, Codex, etc) working on **O1**, a Rust project focused on
hashing, perfect hashing and static maps. It includes project's overview, important concepts, knowledge, constraints,
layout, and hands‑on usage that are helpful to make edits.

---

## 1 Project's goals

- In general - explore and implement different hashing algorithms, hashing schemes.
- Implement static perfect hash-maps, that could be constructed at run-time, compile-time using const-expressions and
  macros, compile-time using code generation.

## 1.2 Workspace crate layout

* `o1` — main library: static maps (current: FKS), hashing families (MSP, XXH3 feature), utilities.
* `o1_core` — core traits and shared types (e.g., `Hasher<T>`, map traits, errors).
* `o1_test` — testing helpers: statistical checks, equivalence tests (run‑time vs const), and data generation.

## 1.3 Core ideas at a glance

### 1.3.1 Perfect hashing

Perfect hashing builds a collision-free hash function for a fixed (static) key set, so lookups are guaranteed O(1) in
the worst case. A common construction is a two-level scheme: first hash keys into buckets, then give each bucket its own
second-level table of size roughly (bucket size)² with a bucket-specific hash so that no collisions occur; the "minimal"
variant packs exactly n keys into n slots. This is great for read-only dictionaries you build once and query many times,
but it’s not suited to dynamic inserts—adding even one new key typically breaks the perfect mapping.

#### 1.3.1.1 FKS perfect hashing scheme

FKS (Fredman–Komlós–Szemerédi) builds a two‑level table: level 1 hashes keys into buckets using a 2‑universal family;
each non‑trivial bucket gets its own level‑2 table sized ≈`k²` with a per‑bucket hash chosen until it is perfect
(collision‑free). This yields expected O(n) total space and O(1) worst‑case queries with at most \~2 memory reads.

- Fredman, Komlós, Szemerédi, *Storing a Sparse Table with O(1) Worst Case Access Time*, **JACM** 31(3), 1984. DOI: 10.1145/828.1884.
- Earlier version: *FOCS 1982*. DOI: 10.1109/SFCS.1982.39.

### 1.3.2 Hybrid run‑time/compile‑time hasher interface

The central entity in the project is Hasher. It uses a hybrid run-time/compile-time interface to make compile-time
construction of static maps with const-expressions possible.

The idea is that a static map's const-constructor uses a compile-time version of the hasher's interface to build
the map at compile-time. And then at run-time it uses the run-time version of the same hasher to perform lookups.

Run-time interface is represented by `o1_core::core::Hasher` trait. Compile-time interface is based on a convention,
rather than enforced by language. This is because Rust traits do not support const-functions as part of them yet. So
instead of relying on traits - compile time methods are implemented as part of impls for each hasher type with a suffix
`_const`, they are supposed to be equivalent to the run-time counterparts.

This approach is also described in
`adr/0000-design-to-support-for-compile-time-and-run-time-hash-table-construction.md`.

Examples of implementations of the hybrid interface could be found in `o1/src/hashing/hashers/msp/` and
`o1/src/hashing/hashers/xxh3/`.

---

## 2 Repository Layout

```
adr/                           Architectural decision records (MADR)
o1_core/                       Core interfaces and errors
  src/core.rs                  `Hasher<T>`, map traits, error types

o1/                            Main library
  src/fks/                     FKS map implementation
  src/hashing/                 Hasher families
    hashers/msp/               MSP hasher-family implementation
    hashers/xxh3/              (feature `xxh3`) XXH3‑based hasher-family implementation
    multiply_shift.rs          Dietzfelbinger multiply‑shift hashing algorithm
    polynomial.rs              Polynomial hashing algorithm

o1_test/                       Test helpers and data
  src/{stat,generate,...}.rs   Statistical tests, generators, equivalence checks
  src/data.rs.tpl              Gomplate template for fixtures
  scripts/gen_map_test_data.py Data generator

README.md                      Project overview
Taskfile.yml                   Tasks (tests, lint, data gen)
.pre-commit-config.yaml        fmt, check, clippy, commitizen
Cargo.toml                     Workspace-wise Cargo configuration - shared deps, features, lint levels
```

---

# 3 Tests

## 3.1 Running the tests

```bash
# Runs all tests from all crates
cargo test

# Runs a test from a specific crate and module
cargo test -p o1 fks::ctors::new::tests::test_build_get_map_u32

# Slow tests are enabled
cargo test -p o1 --features _slow-tests
```


## 4 Taskfile commands

```bash
# Full suite with slow tests
task test

# Lints (fmt, check, clippy)
task lint

# Code generation
task generate
```

---

## 5 Standards and Conventions

## 5.1 Code quality & style

- Generally prefer to follow [Rust Style Guide](https://doc.rust-lang.org/stable/style-guide/).
- Use [the Rust Book](https://doc.rust-lang.org/stable/book/index.html) for a reference on idiomatic Rust code.
- Aim to write idiomatic Rust code
- Follow general principles of writing maintainable code:
  - DRY (Don't Repeat Yourself).
  - KISS (Keep It Simple, Stupid).
  - SPR (Single Responsibility Principle).
  - YAGNI (You Aren't Gonna Need It).
  - Separation of Concerns.
  - Write Self-Documenting Code.
  - Prefer avoiding overly large functions and modules.
  - Prefer not shortened names for variables, functions, other entities. Instead of `svc` prefer `service`, instead of
    `cfg` - `config`, instead of `val` - `value`, etc. For closure parameters use one letter or short names.
  - Etc.
- Follow general OOP principles.
- Follow the existing code style in each existing file.
- Add comments for complex code, sophisticated algorithms, etc.
