# Library Quality Follow-Up

Even if the architecture plan is implemented successfully, the crate should still be reviewed as a library product rather than only as an internal app architecture.

## Follow-up TODOs

- review the public API surface for clarity, naming, and long-term stability
- add focused tests around update/action handling, argv generation, command selection, and geometry-dependent interaction behavior
- improve crate-level documentation and examples for embedding, configuration, and extension
- ensure crate-local abstractions do not leak widget implementation details into the public API unnecessarily
- document compatibility expectations, feature-flag behavior, and semver-sensitive extension points
