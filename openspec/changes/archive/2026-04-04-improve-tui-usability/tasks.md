## 1. Sidebar Navigation Visibility

- [x] 1.1 Add sidebar scroll state and visibility helpers in the UI/navigation state layer so the selected command can be kept within the visible sidebar window.
- [x] 1.2 Update sidebar rendering, frame snapshot hit testing, and pointer-wheel handling to use the windowed sidebar rows instead of always rendering from the top.
- [x] 1.3 Add tests covering keyboard navigation, filtered search results, expand/collapse transitions, sidebar-directed scrolling, and click or caret interactions after the sidebar has scrolled.

## 2. Adaptive Terminal Layout

- [x] 2.1 Introduce a shared compact-layout mode that activates below 20 rows or 80 columns so header, preview, footer, and sidebar chrome yield space before the form becomes unusable, with header collapse and a one-line minimal preview treatment.
- [x] 2.2 Update footer/status rendering to prioritize validation feedback, then primary and secondary actions, before low-priority hints in constrained widths.
- [x] 2.3 Tune spacing and content density so sparse forms do not feel visually overpowered by footer or sidebar chrome.
- [x] 2.4 Clamp dropdowns, help overlays, and toasts so transient surfaces remain fully visible in compact layouts.
- [x] 2.5 Add layout and snapshot-style tests for representative roomy and constrained terminal sizes, including invalid-command states and compact-overlay geometry.

## 3. Command Context Orientation

- [x] 3.1 Update the workspace title and header rendering to show nested command-path context without reserving empty header rows when descriptive content is absent.
- [x] 3.2 Strengthen sidebar tree hierarchy cues so nested branches and expanded structure are easier to scan independently of the active selection.
- [x] 3.3 Revise the preview surface so it clearly identifies itself as command preview output, preserves click-to-copy, and advertises both pointer and `Ctrl+Y` keyboard copy paths in regular and compact layouts.
- [x] 3.4 Add tests covering nested command context rendering, sidebar depth cues, header collapse when `about` is missing, preview click-to-copy, `Ctrl+Y` copy behavior, and visible preview copy affordances.

## 4. Interaction and Feedback Clarity

- [x] 4.1 Expand focus traversal to include search in a fixed Sidebar -> Search -> Form order with reverse traversal support, replace toggle-only focus logic with explicit next or previous focus helpers, and update footer/help hints so advertised focus behavior matches the actual control flow.
- [x] 4.2 Improve choice-widget and counter affordances by documenting multi-select controls inline, replacing dropdown-like counter affordances with stepper-oriented ones, using stronger required-choice empty states, and auditing existing dropdown outside-click retargeting before making targeted fixes.
- [x] 4.3 Clarify inherited-value behavior and required-field empty states with field-level copy that explains what the user can do next and that editing inherited values creates local overrides.
- [x] 4.4 Strengthen feedback styling so validation summaries, primary actions, inherited badges, selected default dropdown rows, success toasts, and error toasts each use clearly differentiated visual treatments that remain legible without relying on color alone.
- [x] 4.5 Add tests covering focus traversal, reverse traversal, multi-select interaction hints, counter rendering, required empty states across widget types, inherited-field guidance, dropdown retargeted clicks to form, sidebar, and search targets, validation-summary emphasis, readable selected defaults, and success or error toast styling.
