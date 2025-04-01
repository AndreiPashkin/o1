# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- README.md with the roadmap.
- .gitignore file.
- pre-commit config.
- Multiply-shift hashing algorithm.
- Polynomial hashing algorithm.
- Statistical testing of strong universality property.
- `Hasher` trait - an abstraction over the hashing algorithms distinct from `core::hash::Hash` and `core::hash::Hasher`
  traits from the standard library.
- `MSPHasher` - a hasher implementation based on the multiply-shift and polynomial hashing algorithms.

### Changed

### Removed

[unreleased]: TODO: create the repository
