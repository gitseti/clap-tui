## 1. Characterization And Audit

- [x] 1.1 Audit repeated projection logic and classify current `FrameSnapshot` usage into layout-only versus domain-coupled call sites.
- [x] 1.2 Add characterization tests for representative keyboard, mouse, dropdown, and invalid-field-focus flows before changing the architecture seams.
- [x] 1.3 Add regression tests proving preview argv, visible validation, and Run gating remain aligned across redraw and interaction flows.

## 2. Shared Selectors And Snapshot Boundaries

- [x] 2.1 Introduce shared selectors for visible sidebar items, visible form items, active form field context, and invalid-field targeting behind behavior-preserving APIs.
- [x] 2.2 Refactor controller, reducer, and render call sites to use the shared selectors instead of recomputing equivalent projections.
- [x] 2.3 Refactor domain-coupled `FrameSnapshot` call sites to read business context from app state or selectors while keeping snapshot helpers layout- and hit-testing-focused.
- [x] 2.4 Add direct selector tests covering active form field resolution, visible sidebar projection, and invalid-field targeting.

## 3. Dispatch Tightening Without Broad Elm Expansion

- [x] 3.1 Preserve and document the current single `Effect` model for this change; do not introduce a broader `Cmd` or subscription layer.
- [x] 3.2 Record the post-selector reassessment of flat action-surface coupling and decide whether scoped message families belong in this change or in a follow-up change.
- [x] 3.3 Reassessment outcome: scoped message families stay out of this change, so the existing top-level dispatch path remains unchanged.
- [x] 3.4 Add reducer, dispatch, and scripted app-flow tests proving representative keyboard and mouse interactions, clipboard copy, dropdown behavior, and preview/validation/run alignment preserve current behavior through the refactor.
