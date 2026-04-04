## MODIFIED Requirements

### Requirement: Sidebar hierarchy remains easy to scan
The TUI SHALL use visual hierarchy in the sidebar tree that makes group labels, actionable commands, child depth, branch state, and the active row easy to distinguish.

#### Scenario: Nested branch is expanded
- **WHEN** the sidebar renders commands at multiple nesting levels
- **THEN** parent and child rows use clear depth cues beyond text labels alone
- **AND** expanded structure is easy to scan without relying solely on the current selection highlight

#### Scenario: Sidebar shows groups and leaf commands together
- **WHEN** the tree includes section-like group labels together with actionable command rows
- **THEN** group labels use a more subdued treatment than actionable command rows
- **AND** actionable leaf commands remain visually identifiable as selectable items

#### Scenario: Active row is visible in the sidebar
- **WHEN** a command row is the currently active selection
- **THEN** the active state uses a full-row or equivalently obvious highlight treatment
- **AND** that active treatment remains easy to find at a glance even in a dense tree

### Requirement: Preview advertises its purpose and copy action
The TUI SHALL label the command preview so users can recognize it as the generated invocation, understand its importance as the result of current edits, and discover how to copy it.

#### Scenario: Preview is rendered
- **WHEN** the preview surface is visible
- **THEN** the UI renders text that identifies the surface as command preview output
- **AND** the preview uses a title, border, emphasis, or equivalent treatment strong enough to distinguish it from passive chrome
- **AND** the UI exposes the available copy interaction through a visible hint, label, or title
- **AND** activating the preview with the pointer still copies the generated command line

#### Scenario: Preview includes command, flags, and args
- **WHEN** the preview shows a generated invocation
- **THEN** the rendered command name, flags, and argument values use visual emphasis that helps users parse the invocation structure at a glance
- **AND** that emphasis does not make the preview harder to read in the active theme

#### Scenario: Preview is shown in compact layout
- **WHEN** the preview is rendered in a reduced or compact layout mode
- **THEN** the UI still preserves a recognizable preview label or copy affordance
- **AND** the compact treatment still reads as an important result surface rather than a passive footer hint

#### Scenario: User copies preview without the pointer
- **WHEN** the preview is available and the user invokes the preview-copy keyboard action
- **THEN** the generated command line is copied to the clipboard
- **AND** the UI surfaces that keyboard path in its visible affordances
