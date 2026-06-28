## 1. Public API and Implementation

- [x] 1.1 Add and root-export the documented `TuiSubmission<T>` public type
- [x] 1.2 Implement `Tui::run_with_argv()` and delegate `Tui::run()` to the shared typed path

## 2. Verification Coverage

- [x] 2.1 Add tests for exact argv, derived defaults, hidden entrypoints, cancellation, and errors

## 3. Documentation and Validation

- [x] 3.1 Document the richer typed API in rustdoc and README with an argv-inspection example
- [x] 3.2 Run formatting, clippy, tests, rustdoc, and package verification
- [x] 3.3 Add a compile-checked hello-world example for `run_with_argv()`
