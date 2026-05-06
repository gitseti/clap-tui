# mvu-architecture-discipline Specification

## Purpose
TBD - created by archiving change tighten-mvu-architecture. Update Purpose after archive.
## Requirements
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

### Requirement: Form field geometry has explicit layout ownership
The TUI SHALL keep responsive form field geometry behind the crate-level `layout::form` boundary. Form layout code MUST produce reusable field projections for labels, inputs, descriptions, field bounds, mode-specific offsets, and repeated-field input geometry without depending on widget rendering code. Query/selector code MUST remain responsible for deciding which fields are visible and in what semantic order, not for owning long-term label/input projection math.

#### Scenario: Layout projects field geometry before rendering
- **WHEN** the form needs field rectangles for rendering, scroll bounds, hit testing, or snapshot population
- **THEN** crate-level `layout::form` produces the field projection
- **THEN** widgets consume projected rectangles instead of calculating responsive label/input placement themselves

#### Scenario: Query helpers do not own responsive layout math
- **WHEN** form query helpers determine visible arguments, field ordering, section headings, or semantic field facts
- **THEN** they may pass those facts into the layout boundary
- **THEN** they do not remain the primary owner of responsive label, input, description, and field-bounds projection logic

#### Scenario: Layout remains independent of widgets
- **WHEN** form layout projection is computed
- **THEN** it depends on model/query data and geometry inputs
- **THEN** it does not depend on `ratatui::Frame` or widget rendering modules
- **THEN** widget modules draw inside the resulting layout rectangles

### Requirement: Form widgets consume shared render models
The TUI SHALL derive common per-field rendering context once before dispatching to widget-specific form renderers. Widget-specific renderers MUST consume that shared render model for selection, validation, effective-value, default/source, required, editability, and widget-kind context instead of independently recomputing equivalent domain or derived-state projections. Text-like model values MUST borrow existing data or use `Cow<'a, str>` when possible, allocating only for values that must be constructed for display.

#### Scenario: Field render model drives widget dispatch
- **WHEN** the form renderer prepares to render a visible field
- **THEN** it builds a render model from the screen view, UI state, ordered argument, frame layout, and derived field state
- **THEN** the selected widget renderer receives that model together with drawing-specific geometry and configuration inputs

#### Scenario: Widget renderer does not duplicate domain projection
- **WHEN** a text, repeated-value, optional-value, toggle, choice, or counter renderer needs common field state
- **THEN** it reads that state from the shared render model
- **THEN** it does not independently resolve validation, effective values, required badges, editability, source badges, or default-backed styling from the broader application state

#### Scenario: Render model avoids unnecessary owned strings
- **WHEN** a field render model represents text that already exists in command, input, validation, or derived state
- **THEN** it borrows that text directly or stores it as `Cow::Borrowed`
- **THEN** it allocates owned text only for derived display strings that cannot be borrowed directly

#### Scenario: Render model preserves existing ownership boundaries
- **WHEN** form rendering consumes geometry and domain state
- **THEN** geometry continues to come from the frame snapshot and form layout projection
- **THEN** command, validation, effective-value, and selection semantics continue to come from the screen view, UI state, and derived state
- **THEN** the render model remains a render-facing projection rather than a second domain model
