## 1. Serializer Policy

- [x] 1.1 Update command-local argv synthesis to preserve clap-safe semantic assignment when positionals and variable-arity options coexist.
- [x] 1.2 Remove or narrow any preview/parse serialization split that can hide parse-affecting materialized defaults or token ordering differences.

## 2. Regression Coverage

- [x] 2.1 Add serializer or pipeline tests covering a positional that must appear before a greedy option to preserve clap parsing.
- [x] 2.2 Add coverage that preview, validation, and run share the same parse-relevant argv when defaults are materialized.

## 3. Validation

- [x] 3.1 Verify the `kitchen-sink serve` scenario reparses to the intended `document_root` and `feature` assignments.
- [x] 3.2 Run the relevant test suite for clap argv fidelity and update any affected expectations.
