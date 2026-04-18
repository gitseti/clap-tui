## MODIFIED Requirements

### Requirement: Dense forms establish a scannable information hierarchy
The TUI SHALL visually distinguish section framing, field labels, editable values, metadata badges, help text, and validation messaging so dense forms can be scanned without reading each line in order.

#### Scenario: Field renders label, value, metadata, and help
- **WHEN** a form field shows a label, current value, inherited or default metadata, and descriptive help
- **THEN** the editable value is visually more prominent than the label
- **AND** the label is visually more prominent than descriptive help text
- **AND** metadata badges remain compact and visually secondary to the editable value

#### Scenario: Long form renders multiple sections
- **WHEN** the form shows multiple argument groups or long vertical runs of fields
- **THEN** section heading, control, metadata, help text, and spacing follow a consistent vertical rhythm
- **AND** adjacent sections remain distinguishable through lightweight framing rather than extra heavy panel chrome

### Requirement: Theme colors map to stable semantic roles
The TUI SHALL assign stable semantic roles to shell surfaces, active workflow surfaces, focus, selection, passive metadata, inherited or implicit state, success, warning-like state, and error feedback so different meanings do not rely on the same accent treatment.

#### Scenario: Focus and success states are visible together
- **WHEN** the UI shows a focused control and a success-oriented state in the same screen
- **THEN** the two states use distinct visual treatments
- **AND** a user can distinguish focus from success without reading explanatory copy

#### Scenario: Inherited or implicit metadata is rendered
- **WHEN** the UI displays inherited, default, environment-sourced, or implicit-value metadata
- **THEN** that metadata uses a semantic treatment distinct from both passive neutral text and error feedback
- **AND** the treatment remains visually secondary to the current editable value

#### Scenario: Adjacent surfaces are rendered in any supported theme
- **WHEN** the app renders the outer shell, sidebar, content panel, input surface, and preview surface in any supported theme preset
- **THEN** neighboring layers remain visually separable without relying on border color alone
- **AND** the active workflow surfaces remain more visually prominent than passive shell chrome

#### Scenario: Control chrome and command output appear together
- **WHEN** the UI renders interactive controls together with breadcrumb or generated invocation content
- **THEN** control chrome emphasis and command-or-result emphasis use distinct semantic treatments
- **AND** a user can distinguish editable interface chrome from invocation output without relying on labels alone

### Requirement: Surface chrome follows a shared visual language
The TUI SHALL use a consistent chrome vocabulary for shell surfaces, sidebar, workspace, preview, overlays, and compact action areas so the screen feels like one deliberate design system even when the surfaces have different priorities.

#### Scenario: Multiple major surfaces are visible together
- **WHEN** the sidebar, main workspace, preview, and footer are rendered in the same frame
- **THEN** their borders, titles, layering, background treatments, or spacing read as belonging to one design system
- **AND** differences in importance are expressed intentionally through stronger or quieter treatments rather than by widget-local accident

#### Scenario: Overlay surface is opened above the workspace
- **WHEN** a dropdown or help overlay appears on top of the main screen
- **THEN** the overlay uses chrome and layering that clearly identifies it as transient and foregrounded
- **AND** the overlay still feels visually related to the surrounding UI
