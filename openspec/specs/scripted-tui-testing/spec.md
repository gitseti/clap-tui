## ADDED Requirements

### Requirement: App-level scripted TUI flows are testable deterministically
The crate's internal test suite SHALL support deterministic scripted app-flow tests that drive the real TUI application loop with scripted input events and a non-terminal backend. These tests MUST be able to exercise redraw, input dispatch, state updates, effects, and final run outcomes without requiring a PTY or live terminal session.

#### Scenario: Scripted flow drives a successful run
- **WHEN** a test provides a command definition, a scripted sequence of input events, and a deterministic runtime/backend
- **THEN** the app loop processes those events through the same logic used by interactive sessions
- **THEN** the test can assert on the final argv returned by Run

#### Scenario: Scripted flow observes cancellation
- **WHEN** a test drives the app with a scripted cancellation sequence
- **THEN** the app loop exits through the same cancellation path used by interactive sessions
- **THEN** the test can assert on that cancellation outcome without using a real terminal

### Requirement: Scripted tests can inspect rendered frames and layout
The crate-internal scripted testing harness SHALL expose the latest rendered frame in a form that allows tests to assert on visible text and locate meaningful UI targets from layout metadata. This observation support MUST remain crate-internal or test-only rather than establishing a new stable public extension surface. Tests MUST be able to react to intermediate frames rather than only to the final result.

#### Scenario: Test waits for a rendered target before interacting
- **WHEN** a scripted test needs to interact with a dropdown, footer button, or other rendered control
- **THEN** it can inspect the latest rendered frame and associated layout metadata before emitting the next input event
- **THEN** it can derive interaction coordinates from that layout instead of relying on hard-coded screen positions

#### Scenario: Test asserts validation state from an intermediate frame
- **WHEN** a scripted flow produces an invalid command state before the final action
- **THEN** the test can inspect the rendered frame text for the visible validation message
- **THEN** the test can continue driving the flow after that assertion

### Requirement: Representative scenario coverage protects interactive regressions
The crate SHALL include scenario-focused scripted tests for representative interactive user journeys rather than relying only on reducer and pure render tests. These scenarios MUST cover both successful interaction and at least one invalid or blocked path.

#### Scenario: Happy path covers realistic mixed interaction
- **WHEN** the crate defines a representative scripted scenario with navigation, value editing, and Run
- **THEN** the scenario verifies that the rendered UI, event handling, and final argv stay aligned through the full flow

#### Scenario: Invalid path blocks Run with visible feedback
- **WHEN** a scripted scenario attempts to run a command that is still invalid
- **THEN** Run is blocked rather than returning argv
- **THEN** the rendered UI exposes the validation feedback expected for that invalid state

#### Scenario: Event-loop behavior is covered beyond pure reducers
- **WHEN** the app loop handles behavior that depends on redraw timing or loop control flow, such as toast expiry or resize-triggered redraw
- **THEN** at least one scripted app-flow test exercises that behavior through the real loop
- **THEN** the regression is covered at a level higher than reducer-only or render-only tests

### Requirement: The project documents the role of scripted TUI tests
Contributor-facing documentation SHALL explain where scripted TUI tests fit in the project’s testing strategy relative to reducer tests, render tests, and any optional PTY smoke coverage.

#### Scenario: Contributor chooses the right test layer
- **WHEN** a contributor adds or changes interactive behavior
- **THEN** the project documentation explains when scripted TUI flow tests are appropriate
- **THEN** the contributor can distinguish them from lower-level reducer or render assertions
