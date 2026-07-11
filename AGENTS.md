# Coding Notes for Agents

This file contains shared workspace rules for this repository. `range-set-blaze` is a
published crate (crates.io) with real downstream users, so stability, semver discipline, and
public-API quality matter more here than in an experimental project.

## Stability and API Policy

- This crate is published and versioned under semver. Treat any change to a `pub` signature,
  trait bound, or re-export as a breaking-change candidate; call it out explicitly rather than
  making it silently.
- **July 2026 review note:** the experimental float features are being reviewed as their first
  version. During this review, intentionally revise their public APIs and internal code
  organization when that improves clarity or long-term design; this note may no longer be true
  after this review period.
- Do not remove or rename public items. If an item is genuinely obsolete, deprecate it with
  `#[deprecated(note = "...")]` and leave removal to a deliberate major-version bump the human
  approves.
- New capabilities that are not yet API-stable belong behind a `_experimental` (or
  `_nightly_experimental`) feature flag, matching the existing `rog_experimental`,
  `float_experimental`, and `float_nightly_experimental` pattern in `Cargo.toml`.
  Do not fold experimental behavior into the default feature set.
- Keep the crate `no_std` (see `#![no_std]` in `src/lib.rs`) and `alloc`-based (not
  allocation-free) unless the user explicitly changes that goal. The `std` feature only adds
  `std`-specific trait impls/conveniences on top, it is not required for core functionality.
- Respect the documented MSRV (`rust-version` in `Cargo.toml`, currently 1.87) and `edition`.
  Don't use newer syntax/stdlib features than the MSRV allows.

## Unsafe Code

- Avoid introducing new `unsafe` blocks. The crate has a small number of existing `unsafe`
  transmutes (`src/float/total.rs`, `src/float/finite.rs`) for zero-cost layout reinterpretation
  between newtype wrappers and their primitive representation — each is narrowly scoped and
  documented at the call site. Follow that bar: if a change truly requires `unsafe`, call it out
  explicitly, keep it as narrow as possible, and explain the safety invariant in a `// SAFETY:`
  comment so the user can review it carefully.
- Do not "fix" warnings or errors by suppressing lints (`#[allow(...)]`, crate-level allow
  attributes, or similar) unless the human explicitly requests that suppression.
- If warnings are caused by obsolete code, delete or refactor the obsolete code instead of
  hiding the warning.

## Type Invariants

- Some types document a public invariant narrower than their raw representation allows — e.g.
  `Finite<T>` (`src/float/finite.rs`) claims "only finite values, with zero canonicalized to
  `+0.0`, are legal" even though its representation is a full `f64`/`f32`. For these types, any
  **safe** method that could otherwise construct or return a value violating that invariant must
  guard the precondition with an unconditional `assert!` (checked in release builds too), or must
  itself be `unsafe fn` with a `# Safety` doc explaining the precondition the caller must uphold.
  `debug_assert!`-only checking is not acceptable for these methods, because it lets 100% safe
  code break the invariant in release builds — silently contradicting doc comments like
  `new_unchecked`'s that promise safe code can never do that.
- This does *not* apply to types whose "invariant" is just being a valid instance of their own
  representation, with no narrower legal subset — e.g. plain integers (`i32`, `u32`, ...) or
  `Total<T>` (whose legal domain equals its entire `Ordered` bit-width, including infinities). For
  those, wrapping arithmetic that overflows always produces some other legal value of the same
  type; there's nothing to break. `debug_assert!`-only preconditions are fine there and match
  Rust's own `+`/`-` operators (panic in debug, wrap in release) — see `Integer::add_one`,
  `Integer::inclusive_end_from_start` in `src/integer.rs`, and `FiniteFloat::inclusive_end_from_start`
  in `src/float/finite_float.rs` (which operates on the raw primitive float, not the narrower
  `Finite<T>` wrapper).
- When adding or reviewing a method on an invariant-bearing type, ask: "if the caller passes an
  out-of-precondition argument, can the *type-level* invariant (not just the numeric answer) end
  up wrong?" If yes, it needs a hard `assert!` or `unsafe fn`, not `debug_assert!`.

## Error Handling

- This is a data structure crate in the spirit of `BTreeSet`/`BTreeMap`/`HashSet`: the public API
  should not return `Result`. Invalid input (e.g. a malformed range, NaN where a total order is
  required) should fail via `assert!`/`panic!` at the call site, the same way `BTreeSet` panics
  rather than returning `Result` for programmer-error conditions. Don't introduce `Result` return
  types, including for fallible construction — prefer `Option` (single failure mode) or a panic,
  and don't implement `TryFrom`/`TryInto` just to get a `Result`-shaped constructor.
- Avoid silent clamping or best-effort fallback behavior on out-of-range or invalid input; prefer
  asserts so misuse fails fast and visibly, especially in `const fn` and hot paths where a silent
  wrong answer would be worse than a panic.
- Never use `let _ = …` to suppress a genuine `Result`. For a non-`Result` value that is
  intentionally unused, call the function as a plain statement instead of binding it to `_`.

## Testing and Local CI

- `just check-all` is the local CI gate — it mirrors what GitHub CI runs (clippy, stable tests,
  nightly tests, formatting, doc links, etc.). Run it before pushing.
- `just clippy` matches CI's exact clippy invocation (`-D clippy::all`); do not add narrower
  clippy configs that could pass locally but fail CI, and do not weaken CI's clippy flags.
- When adding a feature flag, wire it into the relevant `just`/xtask commands so CI actually
  exercises it — an experimental feature that only compiles when a human remembers to pass
  `--features` by hand is effectively untested.
- Prefer `no_run` doctests over `ignore`; use `ignore` only when truly necessary and say why.
  Always write `rust,no_run` (not bare `no_run`) in the fence. Hide setup boilerplate with a `#`
  prefix when it's required for compilation but not relevant to the reader.
- For public methods, prefer a doctest on the method itself. Where one shared example covers a
  family of methods, have each method's doc comment link to it explicitly rather than leaving
  the connection implicit.
- Do not remove debug/test code, commented-out comparison blocks, or in-progress test scaffolding
  until the underlying issue is proven fixed and the human has accepted the cleanup.

## Import Style

- If an item comes from `crate`, `core`, `std`, or `alloc`, import it with `use` instead of using
  a fully-qualified path in code. Fully-qualified paths are fine in docs or comments.

## Module Structure Convention

Do not create `mod.rs` files. This repo already follows the `src/foo.rs` + `src/foo/bar.rs`
pattern throughout (e.g. `src/float.rs` + `src/float/finite.rs`); keep new modules consistent
with it.

## Code Organization

- In binaries and examples, `main` should be the first function in the file, with helper
  functions defined below it. This isn't Pascal — readers want the entry point first, not
  buried after its helpers.

## Comment and TODO Conventions

- Plain `TODO` means non-blocking/future work.
- Never delete `TODO`/`TODO0`/`TODO000` (or similarly numbered `todo`) comments from the
  codebase, even when refactoring the surrounding code or implementing the work the comment
  describes. These are intentional reminders the user placed deliberately. Only remove one when
  the user explicitly asks you to close it out. If code moves, move the comment with it; if you
  believe it's stale, append `(may no longer apply)` rather than deleting it.
- Document non-obvious invariants (e.g. why an `unsafe` transmute is sound, why a bound is
  needed for coherence) at the point of use, not just in a commit message.

## Documentation Conventions

- Use American spelling.
- When linking to module or type documentation, name the item in the link text.
- If an item comes from `crate`, `core`, or `alloc`, import it with `use` instead of a
  fully-qualified path in code; fully-qualified paths are fine in docs/comments.
- Rust getters should not use a `get_` prefix (`len()`, not `get_len()`).
- Markdown formatting: blank lines before/after lists, fenced code blocks, and headings; keep
  list marker style consistent within a file.
- Spell out identifiers (`percent`, not `pct`) rather than abbreviating; abbreviations save
  little typing and cost a reader a small decode step every time.

## Specs

Put implementation specs (`*_SPEC.md` and similar planning documents) in the `specs/` directory,
not the repo root. Every spec must include a `todo0` comment near the top reminding readers to
consider deleting the spec once the work it describes is complete, for example:

```markdown
<!-- todo0 consider deleting this spec once the work below is implemented and released. -->
```

## Release Discipline

- Do not run the real `cargo publish` (or `just publish-dry-all`'s non-dry variant). Prepare
  release notes, version bumps, and the publish command, but the actual publish step is run by
  the human.
- Always suggest a concise 1-2 line commit message when completing work, in a fenced code block.
- Treat `Cargo.lock` and dependency version bumps as deliberate, reviewable changes, not
  incidental side effects of an unrelated task.
