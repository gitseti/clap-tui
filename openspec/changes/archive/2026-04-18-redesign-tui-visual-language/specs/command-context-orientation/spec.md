## MODIFIED Requirements

### Requirement: Workspace surfaces the selected command hierarchy
The TUI SHALL show enough command-path context in the main workspace for users to identify the currently selected nested command without relying on the sidebar alone, and SHALL present that context as a deliberate header region instead of incidental panel chrome.

#### Scenario: Nested command is selected
- **WHEN** the user selects a subcommand below the root command
- **THEN** the main workspace renders the selected command path or breadcrumb context
- **AND** the rendered context distinguishes the selected nested command from its ancestors

#### Scenario: Sidebar is not the focused panel
- **WHEN** focus is in the form or another non-sidebar surface
- **THEN** the main workspace still renders the current command context
- **AND** the user can identify which command the form edits

#### Scenario: Workspace header includes command description
- **WHEN** the selected command provides descriptive `about` text
- **THEN** the workspace renders that description as part of the main context header
- **AND** the description remains visually secondary to the selected command path while still clearly associated with it

### Requirement: Sidebar hierarchy remains easy to scan
The TUI SHALL use visual hierarchy in the sidebar tree that makes group labels, actionable commands, child depth, branch state, and the active row easy to distinguish at a glance.

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

#### Scenario: Sidebar exposes branch and row affordances
- **WHEN** the sidebar renders expandable groups, leaf commands, and the currently active row together
- **THEN** expandable rows expose a visible branch-state affordance beyond indentation alone
- **AND** the active row exposes an additional affordance such as an edge marker, trailing indicator, or equivalent emphasis beyond background fill alone
- **AND** these affordances remain legible even when the sidebar is not the focused surface

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

#### Scenario: Preview reads as a horizontal result band
- **WHEN** the preview is rendered in a roomy layout
- **THEN** it reads as a strong horizontal result surface distinct from the editing form above it
- **AND** its title or header treatment anchors the left side while the copy affordance remains discoverable on the right
