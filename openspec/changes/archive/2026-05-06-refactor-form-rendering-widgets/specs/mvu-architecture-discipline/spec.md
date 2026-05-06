## ADDED Requirements

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
