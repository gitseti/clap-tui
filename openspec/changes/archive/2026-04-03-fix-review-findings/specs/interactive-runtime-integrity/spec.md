## ADDED Requirements

### Requirement: Paste events reach interactive text flows
The TUI SHALL interpret `Paste` events according to the active focus target instead of discarding them. Search input and text-editing form widgets MUST accept pasted text through the same logical editing flow as typed text.

#### Scenario: Search accepts pasted text
- **WHEN** search is focused and the runtime emits a paste event
- **THEN** the pasted text is appended to the active search query
- **THEN** the sidebar view updates using the new query text

#### Scenario: Text field accepts pasted text
- **WHEN** a text-editing form widget is focused and the runtime emits a paste event
- **THEN** the pasted text is inserted into the active editor state
- **THEN** preview argv and validation reflect the resulting field value

### Requirement: Toast expiration is time-based during active interaction
Transient toast visibility MUST be driven by elapsed time rather than by idle polling alone. Expired toasts SHALL clear during sustained input as well as during idle periods.

#### Scenario: Toast clears while the user keeps interacting
- **WHEN** a toast expires while the user continues to produce keyboard or mouse input
- **THEN** the expired toast is removed without waiting for an idle poll cycle
- **THEN** the next redraw omits the expired toast

