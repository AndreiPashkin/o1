---
id: 0002
status: accepted
date: `2025-05-08`
parent:
---
- Core crate - to avoid dependency cycle.
- Unified versions, release cycle and changelog - because it's simpler and easier.
  - Also, Serde does it.

# Split the project into multiple crates

## Context and Problem Statement

At this point the project became moderately large (~5k LoC) and the most important the testing submodule now
contains a large auto-generated file `data.rs` that is of 10k+ LoC, and it slows down compilation even if no changes
are made to it.

## Decision Drivers

1. Slow compilation times

## Considered Options

1. Split the project into multiple crates
   1. Keep crate versions independent and release them separately.
   2. Keep a unified version and release cycle for all the crates.
2. Keep the data in `data.rs` as a binary file and include it via `include_bytes!` macro.

## Decision Outcome

1. Accepted:
    - After realizing that there are no good alternatives and splitting into crates is a [recommended practice][1].
    - A lot of big projects do that:
      - [Serde][2].
      - [Tokio][3].
      - [Clap][4].

   1. Rejected - it would make sense if the individual crates were big and complex enough (like in Tokio),
      so that making a change in one of them would really serve some narrow purpose within this crate only. But
      this is not our case - for now the project is simple enough and change in any of the crates would likely
      serve some project-wise goal. Serde, for example, uses unified versioning approach.
   2. Accepted. As a side not - interesting that Cargo allows to inherit versions of crates from the workspace's
      `Cargo.toml` - it allows to avoid repetition an errors and avoid going to each crate's `Cargo.toml` and bumping
      the version manually every time - but Serde project doesn't use it for some reason.
2. Rejected, because it could still be slow and I also sometimes need to refer to the data during the development and
   bugfixing (for example to create a small test for an isolated test for only one key-value pair).

[1]: https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html
[2]: https://github.com/serde-rs/serde
[3]: https://github.com/tokio-rs/tokio
[4]: https://github.com/clap-rs/clap
