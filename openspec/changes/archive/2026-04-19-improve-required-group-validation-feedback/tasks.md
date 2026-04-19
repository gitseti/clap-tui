## 1. Validation Adaptation

- [x] 1.1 Extend missing-required validation adaptation to recognize required-group failures reported through composite clap references.
- [x] 1.2 Populate `ValidationState.summary` for missing required groups with stable corrective wording instead of allowing an empty or generic fallback.
- [x] 1.3 Populate `ValidationState.field_errors` for the visible member fields of a missing required group.

## 2. UI Behavior Alignment

- [x] 2.1 Ensure required-group validation summaries and inline field errors stay consistent when rendered in the form and footer.
- [x] 2.2 Ensure correction navigation focuses the first visible member field for a missing required group.

## 3. Regression Coverage

- [x] 3.1 Add pipeline-level tests covering missing required groups, including composite references such as `<--fast|--safe>`.
- [x] 3.2 Add rendered UI or interaction tests covering inline errors, footer summary feedback, and correction focus for missing required groups.
