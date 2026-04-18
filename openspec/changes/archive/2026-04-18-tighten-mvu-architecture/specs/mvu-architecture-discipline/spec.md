## ADDED Requirements

### Requirement: Architecture tightening preserves representative interaction behavior
The TUI SHALL preserve the current behavior of representative keyboard and pointer interaction flows while internal selectors, reducer boundaries, and snapshot usage are tightened. The representative baseline for those flows MUST be established by the characterization coverage added in this change.

#### Scenario: Keyboard flow preserves selection and run alignment
- **WHEN** a representative keyboard interaction flow navigates the sidebar, edits a form field, and invokes Run
- **THEN** selected command state, active field state, preview argv, visible validation, and Run gating remain aligned with the pre-refactor behavior
- **THEN** redraw-only events do not change those outcomes

#### Scenario: Pointer flow preserves hit testing and state updates
- **WHEN** a representative pointer interaction flow clicks sidebar items, form inputs, dropdown options, and preview or footer controls
- **THEN** hit testing resolves the same rendered targets as before the refactor
- **THEN** the resulting selection, dropdown, copy, and focus behavior remains aligned with the pre-refactor behavior

### Requirement: Shared selectors drive common interaction context
The TUI SHALL provide shared selector helpers for common interaction context such as visible sidebar items, visible form items, the active form field, and invalid-field targeting. Controllers, reducers, and render view-model assembly MUST use those shared selectors instead of recomputing equivalent projections independently.

#### Scenario: Active form field stays aligned across classification and update
- **WHEN** keyboard classification and reducer logic both need the currently selected form field
- **THEN** they resolve it through the same shared selector contract
- **THEN** the field mutated by the reducer matches the field rendered as active to the user

#### Scenario: Sidebar projection stays aligned across navigation and rendering
- **WHEN** search filtering changes the visible sidebar items
- **THEN** navigation logic and rendering both resolve sidebar items through the same shared selector contract
- **THEN** selection clamping and rendered sidebar rows stay aligned

### Requirement: Side effects remain explicit through the current app-loop effect model
The TUI SHALL keep run completion, clipboard writes, and other non-stateful work in the current explicit `Effect` model interpreted at the app-loop boundary rather than executing them during input classification or rendering.

#### Scenario: Copy preview emits an explicit effect
- **WHEN** a reducer handles a copy-preview interaction
- **THEN** it returns an explicit `Effect` describing the clipboard operation
- **THEN** the app loop interprets that effect outside the reducer

#### Scenario: Run gating uses explicit effects and shared derived state
- **WHEN** a reducer handles a run interaction
- **THEN** it returns an explicit `Effect` for run execution
- **THEN** the app loop uses the shared derived validation state to decide whether execution proceeds

### Requirement: Frame snapshots remain layout-derived rather than domain-derived
The TUI SHALL use `FrameSnapshot` only for geometry, hit-testing, and viewport bookkeeping derived from the last render. Interaction logic that depends on command state, selection semantics, or validation semantics MUST read those from `AppState` or shared selectors instead of treating `FrameSnapshot` as a second domain model.

#### Scenario: Pointer hit testing uses layout without owning domain truth
- **WHEN** a pointer interaction needs to locate a rendered control
- **THEN** it may use the latest frame snapshot for coordinates and hit testing
- **THEN** the underlying command and selection semantics still come from app state and shared selectors

#### Scenario: Interaction logic reads business context from app state
- **WHEN** an interaction depends on the active argument, selected command path, or validation state
- **THEN** that logic reads business context from app state or shared selectors
- **THEN** frame snapshot data remains limited to view-derived layout metadata
