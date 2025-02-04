---
status: "accepted"
date: `2025-02-01`
id: 0000
---

# Design to support for compile-time and run-time hash table construction

## Context and Problem Statement

One of the main ways of using perfect hash tables is to pre-compute the hash table at compile time and include it
into the program's binary. At the same time obviously hash tables should operate at run-time.

This is not easy to achieve and there are multiple ways of doing that.

It is also desirable to have a way to construct (not just operate) hash tables at run-time.
And on top of that a user should be able to choose a hash function for the hash table - which is also hard
to achieve because trait-based polymorphism doesn't work in const-expressions.

The problem is how to achieve both compile-time and run-time construction and how to design the type hierarchy to
support all these use-cases and ensure maximum code re-use, avoid hindering performance with having many abstractions
and layers of indirection and provide a nice user-facing API that would be not overly imposing in terms of mental
overhead.

## Decision Drivers

1. Performance.

   The design should not hinder performance. In the sense that there should not be layers of indirection
   that introduce unnecessary overhead.
2. Functional completeness.

   - Supporting both compile-time and run-time construction is very desirable for the project.
   - A reasonable level of modularity and ability to customize the hash tables.
   - Support for a simple case of static hash tables that are constructed once and remain immutable.
3. Language constraints.

   Design should take into account the limitations of the Rust language:
   - Rust's traits do not support declaring const methods.
   - And methods declared in traits can't be implemented as const.
4. Out-of-scope features for now:

   - Mutable (dynamic) hash tables.
   - Support for special hash tables that use own special hash-functions.
   - Support for sophisticated hash functions that need additional information to be constructed -
     like the whole set of keys and so on.
   - Adapters for the standard library `Hash`/`Hasher` traits.

## Considered Options

1. Compile-time construction
   1. Procedural macros
      - Advantages:
        - Simplicity and flexibility of macros.
        - Single-phase build process.
        - Procedural derive-macros can access type information.
      - Disadvantages:
        - "Normal" procedural macros lack of type information at macro-evaluation time.
        - And even if the type has been processed by a derive-macro - a normal proc-macro won't be able
          to understand what user-defined type the parsed value belongs to and therefore won't be able
          to access any metadata left by the derive-macro.
   2. Const-expressions
      - Advantages:
        - No complexity of two-stage builds for the user.
        - Enables using type system and generics.
      - Disadvantages:
        - Const-expressions have limitations, not everything can be done with const-expressions.
          [`xxhash-rust`][1] crate maintains separate const and non-const implementations because of that -
          const-functions do not allow to implement certain optimizations.
   3. Two-stage builds with serialization/deserialization
      - Advantages:
        - Allows to use full power of Rust to do whatever for construction of hash tables.
      - Disadvantages:
        - General inconvenience of having to set up a two-stage build.
          Might be an overkill for a small project.
        - Deserialization at compile time might be impossible to implement due to const-expressions' limitations.
   4. Two-stage builds with code-generation
      - Advantages:
        - Same as for 1.3.
        - No limitations of const-expressions.
      - Disadvantages:
        - User would have to implement code-generation for custom types. That could be alleviated with derive-macros.
2. Object oriented design
   1. Type structure
      1. Rely on `Hash` and `Hasher` traits from Rust's `core`.
         - Advantages:
           - Compatibility with the standard library and third-party crates.
         - Disadvantages:
           - They are obviously not usable at compile-time. But for compile-time there it would be necessary
             to have a separate implementation anyway due to Rust's limitations.
           - Memory usage overhead. `Hasher` is designed to hash all primitive types which might require
             for an implementation to rely on multiple hash families and to have a storage for parameters
             for instances of the hash families even if they are not needed for the particular instance of a
             hash table. For example if a hash table is constructed for a type that requires only hashing
             `u32` and `u64` values, parameters for hash functions that are able to hash all other types
             would still have to be stored.
      2. Custom `Hasher` trait with the hashed type as a generic parameter.
         - Advantages:
           - Would allow for leaner type-specific implementations.
         - Disadvantages:
           - No compatibility with the stdlib's traits.
           - Less modular design compared to the stdlib's one - no separation between how to hash and
             what hash-function(s) to use.
      3. Straightforward map design with three generic params - for key, value and hasher types.
      4. Split `Hasher` and `HasherBuilder` traits.
         - Advantages:
           - Would allow for custom construction schemes for more sophisticated hashers.
         - Disadvantages:
           - More complex design.
      5. Support for `Borrow`-able types.
         - Advantages:
           - More idiomatic and usable API.
         - Disadvantages:
           - More complex design.
      6. Derivable `Hasher` trait implementation.
         - Advantages:
           - Would allow for easy implementation of custom hashers.
         - Disadvantages:
           - More complex design.
   2. Code-generation
      1. Reliance on `ToTokens` trait.
         - Advantages:
           - It is a part of the standard library and is already implemented for many types.
         - Disadvantages:
           - I don't see any.
      2. Two-phase builds with code-generation.
         - Advantages:
           - It would allow to fully use all features of Rust language.
         - Disadvantages:
           - It would require a complex setup on the part of the user.
   3. Const-expressions based compile-time construction
      1. Use macros generic types, traits with associated types for polymorphism without traits for compile-time
         construction code.
         - Advantages:
           - It seems like it is the only real option.
         - Disadvantages:
           - Macros won't enforce type-safety. It's a hack to bypass limitations of traits.
      2. Reliance on [compose-idents](https://crates.io/crates/compose-idents) crate to avoid clashes of global
         variables and functions defined by macros used for compile-time construction of hash tables.
         - Advantages:
           - The only option found after research.
         - Disadvantages:
           - Additional layer complexity.
           - RustRover doesn't like it and stops highlighting the syntax.
      3. Duplicate implementations for compile-time and run-time construction.
         - Advantages:
           - There is no choice - look at 1.2. - const-expressions are limited, it wouldn't be possible to craft
             const-expression code that is as efficient as non-const code. So it's simply a constraint that has to
             be satisfied.
         - Disadvantages:
           - Code duplication is a disadvantage in itself.
      4. Const-expressions code only for compile-time construction and equivalent non-const code for run-time operation.
         In the sense that even compile-time constructed hash tables would be operated by run-time code at run-time.
         - Advantages:
           - It would allow to keep the compile-time construction part small and separate, dedicated only and specifically
             to compile-time operation and normal object-oriented code for everything else.
         - Disadvantages:
           - It would require some duplication - but it is inevitable anyway.
           - Another problem is that both implementations have to be equivalent.


## Decision Outcome

- 1.1. - Tentatively rejected, because procedural macros could only operate on syntactic level and therefore limited
         accessing type information. That hinders support for user-defined types (like enums). And even with
         derive-macro tricks it seems to be impossible to properly evaluate values of user-defined types and hash them
         in proc-macros.
- 1.2. - Tentatively approved, because there is no alternative for compile time construction in single-phase builds.
         At the same time it introduces some complications on its own.
- 1.3 - Tentatively rejected:
        - On one hand it seems like it is possible to deserialize the basic types at compile time.
          - The main problem is deserialization of strings. It requires serializing total size of strings
            and then reading it in a procedural-macro and rendering as a constant expression that could be
            used as a size of a static array - used as a memory pool for static strings. It could be done,
            but it would require writing a little subproject while benefits over code-generation are not clear.
        - I might return to it in the future.
- 1.4 - Approved, because it is one of few real options (along with 1.2).
- 2.1.1. - Rejected, because of memory usage and performance overhead due to an additional level of indirection.
- 2.1.2. - Approved, because it would allow for leaner type-specific implementations:
    - Self-contained struct which contains all the necessary state for hashing values such as seed values or the address
      space size.
    - Implementations specific to the type that is hashed. That would allow to tailor each implementation to the
      specific needs of the type and enable optimizations that are type-specific.
    - Every hasher should have alternative constructors for making a new instance from a seed and a number of buckets
      and for making a hasher from the type-specific state object.
    - Design of const-hashers should obviously rely on generic types, traits with associated types and macros.
    - Design of const-hashers requires a separate attention though, I'll discuss it in a separate ADR.
- 2.1.3. - Approved - seems like it fits most use cases of static maps considered for now.
- 2.1.4. - Tentatively rejected, it's not needed for now, but it's a good pattern and might be useful in the future.
- 2.1.5. - Tentatively approved, it's a good and a standard way of doing things in Rust. But implementing it is not a
           priority.
- 2.1.6. - Tentatively approved, very desirable to have, but not a priority for now.
- 2.2.1. - Approved.
- 2.2.2. - Approved.
- 2.3.1. - Approved, no other alternative.
- 2.3.2. - Approved, no other alternative to avoid clashes of global names.
- 2.3.3. - Approved, no other alternative.
- 2.3.4. - Approved, it's a very good idea, but at a cost of additional effort. And the constraint of equivalence
           might hurt the performance of the run-time version.


Overall I see code-generation is the option that has the most prospect of yielding good results
while not being too hard to implement.
Compile-time construction is tricky to implement, it requires writing a lot of macro-involved code. I think it would be
worth to implement it for simple perfect hashing schemes that are used for small data-sets.
Compile-time deserialization requires the most effort to implement while achieving the same as code-generation.

### Debt

1. Detailed design for const-hashers and the compile-time construction API.

[1]: https://github.com/DoumanAsh/xxhash-rust/
