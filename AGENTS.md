# Coding Notes for Agents

## General Policies

- Avoid introducing `unsafe` blocks. If a change truly requires `unsafe`, call it out explicitly and explain why.
- Avoid silent clamping; prefer asserts or typed ranges so out-of-range inputs fail fast.
- Prefer `no_run` doctests; use `ignore` only when absolutely necessary (and explain why).
- Always use `rust,no_run` in doctest fences, not just `no_run`.
- Hide boilerplate in doctests using the `#` prefix for non-essential setup lines.
- When adding docs for modules or public items, keep one primary compilable example and have related docs point back to it instead of duplicating snippets.
- Prefer `const` values defined in local context (inside the function/example) when only used there.
- Do not add redundant `just` recipes that only mirror an existing `cargo` alias/command.

## Module Structure Convention

Do not create `mod.rs` files.

Correct pattern:

- `src/foo.rs` (main module file)
- `src/foo/bar.rs` (submodule)
- `src/foo/baz.rs` (another submodule)

Incorrect pattern:

- `src/foo/mod.rs`

## Variable Naming Conventions

Use standard Rust naming:

- snake_case for locals, fields, and functions
- UpperCamelCase for types
- SCREAMING_SNAKE_CASE for constants

Variables should generally match their type names converted to snake_case.

Avoid single-character variables in normal logic; prefer descriptive names.

When creating references for closures or captured values, append `_ref`.

## Comment Conventions

Use `TODO0`/`TODO00` prefix for TODO items (`TODO` + priority):

```rust
// TODO00 high priority task
// TODO0 lower priority consideration
// TODO lowest standard todo for general items
```

- For stable workarounds with a better nightly path, add:
  `// TODO_NIGHTLY When nightly feature <feature_name> becomes stable, change this code by <specific change>.`
- Preserving comments: do not remove TODO comments when changing code. Move them if needed. If a TODO may be stale, append `(may no longer apply)` instead of deleting it.
- Debug code policy: do not remove debug/test comparison code until the bug is proven fixed and confirmed.
- Commit messages: suggest a concise 1-2 line commit message when completing work, in a fenced code block.

## Documentation Conventions

- Keep examples focused and avoid duplicate near-identical snippets across docs.
- When referring to examples, use explicit names (for example, "`RangeMapBlaze` example") rather than vague labels like "struct-level example."
- Spelling: use American English.

Markdown formatting:

- Add blank lines before and after headings, lists, and fenced code blocks.
- Keep list marker style consistent within a file.

## API Design Patterns

- Avoid redundant API paths; prefer one clear way to do a thing unless there is a strong compatibility reason.
- Do not expose equivalent APIs in multiple forms by default.
- Avoid builder-pattern-heavy APIs when direct constructors are clear.
- Prefer taking slices over requiring users to construct collections.

## Rust Style

- If an item comes from `crate`, `core`, `std`, or `alloc`, import it with `use` rather than long fully-qualified paths in code.
- Follow Rust getter/setter naming:
  - getters: no `get_` prefix
  - setters: `set_` prefix

### Parsing into a Stronger Type

Prefer shadowing when converting from weaker to stronger types:

```rust
let width = width.parse::<u32>()?;
```

Guidelines:

- Shadow at the smallest reasonable scope.
- Use checked conversions before shadowing where truncation/overflow is possible.
- Avoid long-range shadowing that hurts readability.
