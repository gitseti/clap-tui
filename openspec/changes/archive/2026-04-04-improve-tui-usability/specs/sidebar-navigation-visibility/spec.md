## ADDED Requirements

### Requirement: Sidebar keeps the active command visible
The TUI SHALL maintain a visible sidebar window that keeps the selected command row on-screen during keyboard-driven navigation, search result changes, and expand or collapse operations.

#### Scenario: Keyboard navigation reaches rows below the current window
- **WHEN** the user moves the sidebar selection to a command below the currently visible rows
- **THEN** the sidebar window scrolls to keep the newly selected command visible
- **AND** the selected command remains highlighted in the rendered sidebar

#### Scenario: Search narrows the visible command list
- **WHEN** a search query changes the available command rows
- **THEN** the sidebar window and selection are clamped to the filtered result set
- **AND** any remaining selected command is rendered within the visible sidebar window

### Requirement: Sidebar scrolling follows sidebar-directed pointer input
The TUI SHALL apply pointer scrolling over the sidebar to the sidebar command list instead of the main form.

#### Scenario: Pointer wheel moves over the sidebar
- **WHEN** the user scrolls the mouse wheel while the pointer is over the sidebar list
- **THEN** the sidebar window scroll offset changes
- **AND** the form scroll offset remains unchanged

#### Scenario: Pointer wheel moves over a scrolled sidebar with the selected command off-screen
- **WHEN** sidebar scrolling would move the selected command out of view
- **THEN** the rendered sidebar still keeps the active selection visible or repositions the window around it on the next selection update

#### Scenario: User clicks a row in a scrolled sidebar
- **WHEN** the sidebar has scrolled and the user clicks a visible sidebar row or expand/collapse affordance
- **THEN** hit testing resolves against the scrolled sidebar window
- **AND** the clicked row or expand/collapse target receives the same interaction it would have received before scrolling
