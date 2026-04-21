## Crate Setup

Act as a senior Rust library reviewer.

Evaluate this crate setup (Cargo.toml, workspace, macros split, publish strategy).

Focus only on:
- correctness for crates.io publishing
- macro usability (re-exports, UX)
- dependency/version pitfalls
- potential breaking or irreversible decisions

List only concrete issues or risks. No praise, no explanations unless necessary.


## Live VAlidation

Act as a senior Rust reviewer.

Evaluate the validation system of this clap-based TUI (live validation using clap errors).

Focus ONLY on:
- correctness of validation (edge cases, partial input, consistency with clap)
- limitations of using clap for interactive validation
- UX issues in validation behavior (timing, noise, clarity)
- problems for users implementing custom argument types

List only concrete issues or risks. No praise. No general advice.

What you have now
User types in --replicas

Error appears at the bottom:

invalid value 'abc' for '--replicas'

This works, but:

user must map error → field
slower, more cognitive load
✅ What you should do instead
Attach error directly to the input

Visually:

--replicas
[ abc ]
  ❌ Must be a number

or inline:

--replicas   [ abc ❌ ]