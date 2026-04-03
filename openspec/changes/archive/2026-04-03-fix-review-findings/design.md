## Context

The review findings cluster around four seams in `clap-tui`:

- effective input state is partly re-derived from environment variables at read time
- the runtime abstraction exposes paste events, but the app loop drops them
- transient feedback timing is only advanced while the UI is idle
- preview argv and clap-backed validation are recomputed during every redraw
- parser-backed execution is available through a helper that is not bound to the clap schema used to render the TUI

These are cross-cutting issues because they touch startup initialization, reducer flow, rendering, validation, and the public app API. The goal is to fix them without reopening the crate’s broader architecture or changing its dependency stack.

## Goals / Non-Goals

**Goals:**

- Make effective form state deterministic after app initialization.
- Preserve supported interactive input events, including paste, across search and text editing flows.
- Ensure toast expiration is based on time rather than on UI idleness.
- Remove full argv derivation and clap validation from the unconditional render path.
- Provide a parser-backed execution path that is provably aligned with the rendered clap schema.
- Keep the work incremental and testable with the existing architecture.

**Non-Goals:**

- Redesigning the full controller/update architecture.
- Replacing clap-backed validation with a custom validation engine.
- Introducing new runtime or state-management dependencies.
- Reworking unrelated UI behaviors outside paste handling and toast lifetime.

## Decisions

### 1. Materialize environment and default-backed input once during state initialization

Effective input reads should become pure projections over stored state plus static command metadata. Environment-backed defaults will therefore be resolved when a command path is initialized, then stored in owned input state the same way ordinary defaults are stored today.

This keeps later reads deterministic and avoids hidden dependence on ambient process state after startup.

Alternatives considered:

- Keep lazy environment lookup in `initial_input_state`.
  Rejected because it makes plain reads impure and allows visible state to change without any state transition.
- Snapshot the process environment separately and keep lazy lookup against that snapshot.
  Rejected because it still spreads initialization logic into query paths with little benefit over materializing values directly.

### 2. Route paste through focus-aware input handling instead of treating it as a no-op

`AppEvent::Paste(String)` should enter the same focus-aware interpretation flow as keyboard text input. Search focus will append pasted text to the query, and text-editing form widgets will insert pasted text through the existing editor/update paths. Non-text contexts may ignore paste explicitly.

This preserves the runtime contract without requiring a separate editing model.

Alternatives considered:

- Convert paste payloads into synthetic key events.
  Rejected because it loses payload boundaries and makes multiline paste awkward.
- Handle paste only in runtime implementations.
  Rejected because focus-aware behavior belongs at the application layer, not in the backend.

### 3. Make toast expiration part of the main event-loop clock, not only the idle branch

Toast expiration should be checked before redraw decisions and after event handling so elapsed time is observed even under continuous interaction. Clearing an expired toast must request a redraw the same way the idle path does today.

Alternatives considered:

- Keep the current idle-only expiration check.
  Rejected because it makes toast lifetime dependent on user activity.
- Move expiration into rendering.
  Rejected because render should consume visible state, not mutate it.

### 4. Cache derived argv and validation on state transitions and reuse them for rendering and Run

The app should maintain a cached derived view model, with explicit invalidation when reducers mutate relevant domain state. Rendering will consume the cached derived state, and Run will reuse the same validation result unless the cache is dirty.

This keeps clap-backed validation as the source of truth while removing repeated parser work from the redraw hot path.

Alternatives considered:

- Continue deriving on every draw.
  Rejected because redraw frequency is tied to interaction frequency, not to semantic state changes.
- Cache only argv and still revalidate in render.
  Rejected because full clap validation is the dominant hot-path cost.

### 5. Add a schema-bound parser execution API and retire the unbound helper as the preferred path

Parser-backed execution should be exposed only through an API that is tied to the clap schema used to construct the TUI. The safest path is a typed constructor or typed wrapper that binds `T: Parser + CommandFactory` at creation time and then exposes parser execution for that same `T`. Existing untyped execution should remain available through `run` and `run_with_matches`.

The current unbound `run_with_parser` helper should be deprecated or constrained so callers are guided toward the bound path rather than parsing the rendered argv with an unrelated parser type.

Alternatives considered:

- Keep the current API and document that callers must pass the matching parser type.
  Rejected because it leaves the footgun intact.
- Perform only a runtime compatibility check between the rendered command and parser type.
  Rejected as the primary fix because it detects mismatch late and keeps the unsafe shape of the API.

## Risks / Trade-offs

- [Derived-state cache invalidation misses a state transition] -> Centralize invalidation in reducer entry points and add tests covering preview/render/Run alignment.
- [Paste integration creates inconsistent behavior across widgets] -> Restrict paste support to search and existing text-editing widgets first, with explicit no-op handling elsewhere.
- [Materializing defaults changes expectations for environment mutation during a session] -> Define startup resolution as the contract and document that runtime environment changes are not reflected until a new app session starts.
- [Typed parser execution introduces API surface churn] -> Keep untyped `run` and `run_with_matches` stable, add the bound path first, and use deprecation messaging before any removal.

## Migration Plan

This change can land in slices:

1. materialize environment/default-backed input during initialization and remove lazy environment reads from effective-state queries
2. add paste handling and centralize toast expiry checks in the event loop
3. introduce cached derived state and switch render and Run to the shared cache
4. add the schema-bound parser execution API and deprecate or constrain the unbound helper

Rollback remains straightforward because each slice is independently testable and can fall back to the current behavior if needed.

## Open Questions

- Should the typed parser-bound API replace `TuiApp::from_factory` directly or be introduced as an additive constructor/wrapper first?
- Which reducer boundary is the narrowest stable place to own derived-state invalidation without spreading cache logic through every helper?
- Should pasted multiline text into repeated-value editors preserve newline boundaries exactly or normalize through existing text-field semantics?
