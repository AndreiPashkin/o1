---
id: 0001
status: accepted
date: `2025-04-18`
parent: 0000
---

# Const constructors

## Context and Problem Statement

Const constructors is one of the ways of enabling compile-time construction of static maps.

Crafting a const constructor presents a number of challenges - mainly due to the current limitations of the Rust
language.

This document is dedicated to describing these challenges and solutions for how to overcome them.

## Decision Drivers

- Language limitations

  Almost 100% of the decisions in this scope are guided by the limitations of the Rust language and finding
  workarounds for them. There are lots of limitations and not many options.
- Type-aware compile-time construction

  Const constructors are an alternative to proc-macro construction which could only hash basic types.
- Single-phase compile-time construction

  Const constructors are also an alternative to multi-stage builds that involve code generation. They allow
  a developer to construct a static map in a single phase just by adding the constructor call to the code.

## Considered Options

1. Overallocation

   In const-context it's only possible to allocate fixed-size arrays. This is a great limitation and the only way
   to circumvent it is to allocate a memory pool or a buffer of a size large enough to perform all operations.

   This buffer could be allocated as a local variable and after all operations are done a new allocation could be made
   at this time - of the optimal size (implying that the optimal size would be known after work on buffer is done). The
   new allocation could be made static instead of local.

2. Operations on const in global context & multi-stage construction

     - Array sizes must be either literals or const constants.
     - Const constants can be passed to functions either as const-generic parameters or as global variables.
     - Operations on const-generic parameters are not allowed.
     - Extracting sizes of slice-parameters as constants is only possible through destructuring via const-generic
       parameters.

   All these constraints imply that it's impossible to operate on const constants fully within the scope of a
   single function (and const constants are mainly useful for specifying array sizes) - it means that instead
   the only path is to do that in the global context - either with global variables or in the global scope (outside
   any function).

   For example to calculate the size of the memory buffer with worst-case size (as described above) it would be
   necessary to:
     - Extract the size of the initial array through destructuring.
     - Save it as a global const-constant.
     - Calculate the buffer size in the global scope and pass down a sa const-generic parameter or
       use it in functions as a global variable and calculate the size inside functions referring directly
       to the global const constant.

3. Usage of Cow-like smart-pointer

   The idea is to use a smart-pointer that can hold onto memory either allocated on the heap in non-const context
   or statically allocated on the stack in const context and in both cases - provide unified access to this memory.

   `Cow` itself can do that, but it doesn't fully fit the requirements since it bounds the underlying type to
   `ToOwned` trait - and it's not desirable to add this constraint to the map implementations.

   The solution is to write a Cow-like smart-pointer tailored for holding onto slices.

4. Usage of macros for open polymorphism of traitless concrete types

   In const-context normal Rust polymorphism is impossible since traits can't declare const methods and trait methods
   can't be implemented as const. The only option is to craft similar concrete types and pass them as parameters
   to macros - thus achieving interchangeability between the concrete types.

## Decision Outcome

There are not a lot of options and all the techniques described above are necessary to use to achieve a parity
with non-const implementation.

## More Information

### Additional problems and solutions

- Generation of pseudo-random numbers at compile-time
- Manipulation with bit-arrays at compile-time

None of these are solvable with existing third-party crates but at the same time there are not fundamental limitations
of the language that prevents from implementing them. I consider these problems to be trivial but still want
to mention.
