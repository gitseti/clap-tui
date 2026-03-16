# clap-tui performance assessment

Reviewed against the current workspace state on 2026-03-15.

## Summary

The TUI is probably performant enough for small and medium `clap` command graphs, but it is not yet optimized for scale. The code avoids the worst failure mode for terminal apps, an idle redraw loop, so it should feel responsive in ordinary use. The main issue is not one catastrophic hotspot. It is repeated allocation, sorting, path resolution, and view-model rebuilding across the render path and interaction path.

In practice, that means:

- idle behavior should be good
- simple CLIs should feel fine
- larger command trees and forms will pay unnecessary repeated work on every redraw and many keypresses
- multiline text editing is the clearest place where per-keystroke work is heavier than it needs to be

## What is already good

### 1. Redraw policy is event-driven

`crates/clap-tui/src/app.rs:137-168` redraws only when needed, and `crates/clap-tui/src/app.rs:172-181` only wakes on toast expiry when idle. That is the right baseline. It avoids the common TUI problem where the app repaints every 16 ms regardless of state changes.

### 2. State size is still modest

The app state is not huge, and the crate is still architecturally small enough that a focused optimization pass should have clear payoffs without needing a redesign.

### 3. Tests are fast

`cargo test` currently completes quickly. That does not prove interactive performance, but it does suggest there is no obviously explosive algorithm in the current unit-tested logic.

## Main hotspots

## 1. `normalize_state` repeats selector work after most actions

`crates/clap-tui/src/update.rs:94-105` does three things every time it runs:

- ensures defaults for the current command
- resolves the current command and clones it
- rebuilds visible args and then maps them into another temporary vector

This is not terrible once, but it sits on the post-action path for almost all interactions. It means the app often recomputes the same visible-arg projection even when the action only changed hover state, toast state, or dropdown scroll.

### Why this matters

- visible arg lists are also rebuilt elsewhere
- this cost compounds with the render path doing similar work again
- some actions only need UI cleanup, not domain-derived recomputation

### How I would fix it

Split normalization into narrower passes:

1. `normalize_domain_defaults(state: &mut AppState)`
   - only run when the selected command changes
   - or when state enters a command for the first time

2. `normalize_tab_and_selection(state: &mut AppState, current_command: &CommandSpec)`
   - run only when:
     - selected command changes
     - active tab changes
     - search/filter changes the visible list

3. `normalize_transient_ui(state: &mut AppState)`
   - run for dropdown / hover / selection cleanup only when relevant

Concretely, I would stop calling one broad `normalize_state` after every action and instead let reducers return a small normalization hint, for example:

```rust
enum Normalize {
    None,
    Selection,
    CommandChanged,
}
```

Then `apply_action` can trigger only the necessary follow-up work.

## 2. Form selectors sort and allocate repeatedly

`crates/clap-tui/src/view/form.rs:33-69` rebuilds ordered args each time `visible_args()` is called:

- collect positionals into a `Vec`
- sort them
- collect options into another `Vec`
- sort them
- extend into a new `Vec`
- filter again based on tab

This logic is then called from:

- render path
- update normalization
- keyboard input gating
- navigation

Examples:

- `crates/clap-tui/src/ui/screen.rs:62-70`
- `crates/clap-tui/src/update.rs:96-101`
- `crates/clap-tui/src/controller/keyboard.rs:98-104`
- `crates/clap-tui/src/controller/navigation.rs:20-22`
- `crates/clap-tui/src/controller/navigation.rs:79-89`

### Why this matters

The command spec is immutable for the whole session. Sorting the same args over and over is pure waste.

### How I would fix it

Precompute arg ordering once in the spec layer.

The simplest version:

1. Extend `CommandModel` with precomputed arg index lists:
   - `ordered_arg_indices: Vec<usize>`
   - `option_arg_indices: Vec<usize>`
   - `positional_arg_indices: Vec<usize>`

2. Build those vectors in `CommandModel::from_command(...)`.

3. Change `visible_args()` into a borrowed selector over indices:

```rust
pub(crate) fn visible_args<'a>(
    command: &'a CommandSpec,
    active_tab: ActiveTab,
) -> impl Iterator<Item = OrderedArg<'a>> + 'a
```

If using iterators becomes awkward, return `&[usize]` for the chosen tab and adapt callers.

This removes repeated sorting entirely and reduces many temporary allocations to zero.

## 3. The render path rebuilds a lot of derived view data every frame

`crates/clap-tui/src/ui/screen.rs:47-111` rebuilds:

- preview argv
- selected path clone
- full `ScreenView`
- tree items
- visible args
- layout
- cloned frame snapshot

`ScreenView::build` in `crates/clap-tui/src/ui/screen.rs:26-44` is the center of that work.

### Why this matters

Most of those inputs are stable across many frames. For example:

- tree items do not change unless expansion state, search, or command graph changes
- visible args do not change unless command or tab changes
- preview argv does not change unless form state changes

Right now the redraw path recomputes all of them together.

### How I would fix it

I would separate derived data by invalidation boundary.

#### Step 1: introduce a small derived-view cache in state

Something like:

```rust
struct DerivedViewState {
    tree_items: Vec<TreeItem>,
    active_args: Vec<OrderedArgOwned>,
    preview_argv: Vec<String>,
    dirty_tree: bool,
    dirty_args: bool,
    dirty_preview: bool,
}
```

I would not cache layout rectangles here. Only semantic derived data.

#### Step 2: update cache only on specific transitions

- command change:
  - mark tree, args, and preview dirty
- search query change:
  - mark tree dirty
- tab change:
  - mark args dirty
- form value change:
  - mark preview dirty

#### Step 3: make render borrow cached semantic data

`ui::screen::render(...)` should primarily:

- borrow current semantic view data
- compute frame geometry
- paint widgets

That keeps per-frame work closer to actual drawing.

### Important constraint

I would not start with a broad generic cache layer. A few explicit dirty flags are enough here and easier to reason about.

## 4. Tree building allocates aggressively

`crates/clap-tui/src/view/command_tree.rs:14-97` rebuilds the whole visible tree by:

- lowercasing strings during matching
- cloning command names into `path`
- joining path parts into keys
- cloning `path` for every child
- formatting labels into new `String`s
- materializing a full `Vec<TreeItem>`

This happens both in render and in navigation helpers such as:

- `crates/clap-tui/src/controller/navigation.rs:50-68`
- `crates/clap-tui/src/controller/navigation.rs:93-105`
- `crates/clap-tui/src/controller/navigation.rs:108-120`

### Why this matters

The same visible tree is recomputed multiple times for the same user-visible state.

### How I would fix it

There are two levels of fix.

#### Low-risk fix

Cache visible tree items by:

- expanded set version
- search query

and reuse the same `Vec<TreeItem>` for both render and sidebar navigation.

#### Better structural fix

Change `expanded` from `HashSet<String>` to a path-based key that does not require `join("::")` on every traversal.

For example:

- use `CommandPath` directly if it can be made hashable without constant cloning
- or store stable command ids in the spec model and use those ids everywhere

Then tree traversal can avoid building joined string keys repeatedly.

#### Search optimization

Precompute lowercase search fields per command in the spec model:

- lowercase command name
- lowercase `about`

That removes repeated `to_lowercase()` calls in `build_tree_items_inner`.

## 5. Controllers recompute the same derived data instead of sharing it

The keyboard and navigation layers repeatedly resolve the current command and rebuild visible args:

- `crates/clap-tui/src/controller/keyboard.rs:98-104`
- `crates/clap-tui/src/controller/navigation.rs:20-22`
- `crates/clap-tui/src/controller/navigation.rs:79-89`

### Why this matters

This is not just raw CPU cost. It also makes correctness harder because multiple places must agree on the same projection logic.

### How I would fix it

Introduce a selector module with a small set of reused borrowed queries:

- `selected_command(state) -> &CommandSpec`
- `visible_arg_slice(state) -> &[usize]`
- `selected_visible_arg(state) -> Option<&ArgSpec>`
- `sidebar_items(state) -> &[TreeItem]`

Then either:

- compute those selectors from cached derived state
- or compute them once per event dispatch and thread them through

I would favor the cached-derived-state approach because the render path needs the same data.

## 6. Text editing reconstructs `TextArea` on every keypress

`crates/clap-tui/src/editor_state.rs:103-137` converts the internal editor state into a fresh `TextArea`, applies one input, and then copies state back out:

- clone lines into `TextArea`
- mutate `TextArea`
- copy lines back into `Vec<String>`

`EditorState::ensure_editor` also compares against `editor.text()`, which joins all lines into a new `String` before deciding whether to replace the editor (`crates/clap-tui/src/editor_state.rs:39-54`, `crates/clap-tui/src/editor_state.rs:71-72`).

### Why this matters

This is the most direct per-keystroke allocation hotspot in the codebase.

### How I would fix it

I would keep the current widget-agnostic editor model and make it the single source of truth, but stop round-tripping through `TextArea` for ordinary editing operations.

#### Preferred fix

Implement local editing commands directly on `TextEditor`:

- insert char
- backspace
- delete
- newline
- cursor movement
- selection extension

This is more code than the current adapter, but it removes the most expensive repeated conversion in the input path.

#### Lower-risk interim fix

If full local editing is too much for one pass:

1. store a lightweight displayed-text fingerprint
2. avoid `editor.text()` string join in `ensure_editor`
3. keep `TextArea` round-trip only for multiline fields
4. use direct mutation for the common single-line cases

That would already reduce the typical cost materially.

## 7. `build_argv` clones current form state unnecessarily

`crates/clap-tui/src/view/argv.rs:4-20` clones the current form using:

```rust
state.domain.current_form().cloned().unwrap_or_default()
```

This happens for:

- preview rendering
- run action
- copy-preview action
- required-field checking

### Why this matters

The serializer is read-only. It should not require ownership of the whole form state.

### How I would fix it

Change the serializer entry points to accept borrowed form state:

```rust
pub(crate) fn build_argv(command: &CommandModel, state: Option<&CommandFormState>) -> Vec<String>
```

or:

```rust
pub(crate) fn build_argv(command: &CommandModel, state: &CommandFormState) -> Vec<String>
```

with a shared empty default stored once.

That removes one more repeated clone from both render and command execution paths.

## 8. `current_command()` hides extra work and a path clone

`crates/clap-tui/src/input.rs:112-118` resolves the current command by calling `resolved(...)`, and `crates/clap-tui/src/spec.rs:233-240` clones the `CommandPath` to build a `ResolvedCommand`, even though most callers only need `&CommandSpec`.

### Why this matters

This is a smaller hotspot than the others, but it is called often enough that it should be simplified.

### How I would fix it

Replace:

```rust
fn domain_resolved_command(&self) -> ResolvedCommand<'_>
```

with two separate APIs:

- `current_command(&self) -> &CommandSpec`
- `resolved_command(&self) -> (&CommandPath, &CommandSpec)` only where both are actually needed

Most call sites only need the borrowed command. They should not pay for path cloning.

## 9. Layout production still clones more than necessary

`crates/clap-tui/src/ui/screen.rs:71-72` clones the produced snapshot:

- build screen layout
- clone snapshot out of it
- then use both layout areas and the cloned snapshot

`crates/clap-tui/src/ui/form.rs:101-107` also clones arg ids into layout maps and vectors during each layout pass. Those clones are probably acceptable because interaction geometry is frame-local, but the `screen_layout.snapshot.clone()` step is more suspicious.

### How I would fix it

Refactor `layout::build_screen_layout(...)` so the caller can move the snapshot out without cloning. For example:

- return `(ScreenAreas, FrameSnapshot)`
- or keep a layout struct but destructure it by value

This is not the highest-value optimization, but it is easy and local.

## Prioritized fix order

I would implement performance work in this order.

### Phase 1: cheap wins with low design risk

1. Stop cloning form state in `view/argv.rs`.
2. Remove the `ResolvedCommand` path clone from ordinary `current_command()` lookups.
3. Stop cloning the screen layout snapshot in `ui/screen.rs`.
4. Add a selector helper so keyboard/navigation stop rebuilding visible args independently.

Expected result:

- smaller per-event overhead
- no architecture churn
- easy to verify with tests

### Phase 2: remove repeated projection work

1. Precompute ordered arg indices in `CommandSpec`.
2. Replace `visible_args()` sorting with borrowed index-based selection.
3. Cache sidebar tree items and visible args with explicit dirty flags.
4. Narrow `normalize_state` so it only runs the work each action actually needs.

Expected result:

- much lower redraw cost
- much lower navigation cost
- clearer ownership of semantic derived data

### Phase 3: attack the real typing hotspot

1. Rework `TextEditor` to handle common edit operations directly.
2. Keep `TextArea` as a render adapter rather than an input engine.

Expected result:

- lower latency for text entry
- less allocation churn
- cleaner separation between interaction state and widget implementation

## How I would validate the fixes

The repo currently has good correctness tests but no performance harness. I would add lightweight measurement before and during the optimization pass.

### 1. Add micro-benchmarks for pure selectors

Candidate benchmark targets:

- `view::form::visible_args`
- `view::command_tree::tree_items`
- `view::argv::build_argv`

Use representative synthetic command graphs:

- 10 args / 5 subcommands
- 50 args / 20 subcommands
- 100+ args / deep subcommand nesting

### 2. Add reducer-path measurements

Measure:

- tab switch
- sidebar move
- form move
- typing into a multiline field

This can be done with focused benches around reducers and editor state, without running a real terminal backend.

### 3. Add render-only profiling hooks behind a feature flag

Under a crate-local `tracing` or `perf` feature:

- time tree building
- time visible-arg selection
- time preview argv construction
- time layout pass

That will show whether the derived-data changes actually move the needle.

## Concrete implementation sketch

If I were implementing this next, I would use the following sequence:

1. change argv serialization to borrow form state
2. simplify `current_command()` so it resolves directly without `ResolvedCommand`
3. refactor `ui::screen::render` to stop cloning the snapshot
4. add precomputed arg index slices to `CommandSpec`
5. replace `visible_args()` with index-based borrowed access
6. add `DerivedViewState` with explicit dirty flags
7. route keyboard, navigation, and render through shared derived selectors
8. narrow normalization based on action outcome
9. optimize `TextEditor`

That order keeps the early changes local and testable, and delays the more invasive editor work until the rest of the hot path is already cheaper.

## Bottom line

The current TUI should be fine for ordinary usage, but it is paying too much repeated derivation cost across redraws and input handling. The most valuable improvements are:

- stop rebuilding sorted arg lists
- stop rebuilding the command tree multiple times for the same UI state
- stop cloning form state and path data in read-only paths
- stop round-tripping text input through `TextArea` on every keypress

This is a good candidate for a focused optimization pass because the hotspots are understandable, localized, and mostly separable from user-facing behavior.
