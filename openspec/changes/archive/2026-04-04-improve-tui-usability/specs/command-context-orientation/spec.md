## ADDED Requirements

### Requirement: Workspace surfaces the selected command hierarchy
The TUI SHALL show enough command-path context in the main workspace for users to identify the currently selected nested command without relying on the sidebar alone.

#### Scenario: Nested command is selected
- **WHEN** the user selects a subcommand below the root command
- **THEN** the main workspace renders the selected command path or breadcrumb context
- **AND** the rendered context distinguishes the selected nested command from its ancestors

#### Scenario: Sidebar is not the focused panel
- **WHEN** focus is in the form or another non-sidebar surface
- **THEN** the main workspace still renders the current command context
- **AND** the user can identify which command the form edits

### Requirement: Sidebar hierarchy remains easy to scan
The TUI SHALL use visual hierarchy in the sidebar tree that makes expanded branches, child depth, and the active row easy to distinguish.

#### Scenario: Nested branch is expanded
- **WHEN** the sidebar renders commands at multiple nesting levels
- **THEN** parent and child rows use clear depth cues beyond text labels alone
- **AND** expanded structure is easy to scan without relying solely on the current selection highlight

### Requirement: Preview advertises its purpose and copy action
The TUI SHALL label the command preview so users can recognize it as the generated invocation and discover how to copy it.

#### Scenario: Preview is rendered
- **WHEN** the preview surface is visible
- **THEN** the UI renders text that identifies the surface as command preview output
- **AND** the UI exposes the available copy interaction through a visible hint, label, or title
- **AND** activating the preview with the pointer still copies the generated command line

#### Scenario: Preview is shown in compact layout
- **WHEN** the preview is rendered in a reduced or compact layout mode
- **THEN** the UI still preserves a recognizable preview label or copy affordance

#### Scenario: User copies preview without the pointer
- **WHEN** the preview is available and the user invokes the preview-copy keyboard action
- **THEN** the generated command line is copied to the clipboard
- **AND** the UI surfaces that keyboard path in its visible affordances
