# Map joins: candidate features

<!-- todo0 consider deleting this spec once the work below is implemented and released. -->

Status: planning. Each feature below is a candidate, to be accepted or rejected and designed one at
a time. Nothing here is a commitment to add it.

Baseline: branch `inner-join-universe` (PR #36), which adds `SortedDisjointMap::inner_join`,
`InnerJoinIterMap`, `ValueCarrier` for pairs, `RangeMapBlaze::universe_with`, and
`RangeSetBlaze::universe`.

## Motivating callers

- [range-map-regex](https://github.com/CarlKCarlK/range-map-regex) (`src/dfa.rs`). DFA product
  constructions (`union`, `intersection`, `concat`) inner-join two states' transition maps.
  `subset_transition_map` folds k transition maps into a map of state sets with k − 1 pairwise
  inner joins, rebuilding a `RangeMapBlaze` each time. It hand-writes a `map_values` helper.
- [glrmask](https://github.com/IsaacBreen/glrmask), grammar-constrained LLM decoding. Its `Weight`
  is `RangeMapBlaze<u32, Arc<RangeSetBlaze<u32>>>`, mapping state ranges to token sets. Current
  code (2026-10-03, commit `f927a1406`) keeps it in `crates/glrmask-weight/src/implementation.rs`
  (the published 0.1.1 had it in `src/ds/weight.rs`). It hand-writes a two-way union and a two-way
  combine with a `(Option<&Set>, Option<&Set>)` closure (`union_compact_entries`,
  `combine_compact_entries`), a coalescing builder (`CompactRangeBuilder`), and a k-way union that
  unions overlapping token sets (`union_all_multiway*`). It depends on range-set-blaze 0.3.0 and is
  early-stage, so treat it as a design signal, not demand.

  Local test copy: fork [CarlKCarlK/glrmask](https://github.com/CarlKCarlK/glrmask), branch
  `local-rsb`, which builds against a local `range-set-blaze` checkout. Its RSB-related tests are
  in `glrmask-weight` (needs `--features internal-api`), `glrmask-weighted-automata`,
  `glrmask-dwa-merge`, `glrmask-parser-dwa`, and `glrmask-terminal-dwa`.

## Background: why range-map-regex only needs inner join

A DFA's transition map is total: every symbol leads somewhere, even if only to a dead state.
`range-map-regex` builds each state with `universe_with` and asserts `is_universal()` in
`set_transitions`. The inner join of two universal maps is universal, so an outer join would never
yield a missing side there. Outer joins matter for partial maps such as glrmask's.

## Feature 1: two-way full outer join

Decision: accepted and implemented (2026-10-08) in `src/outer_join_iter_map.rs`. Name:
`outer_join`. Item shape: `(Option<VCL>, Option<VCR>)`, no new enum. No separate left/right
methods for now.

Naming note: `inner_join` is the standard term. The standard name for this operation is "full
(outer) join"; "outer join" alone names the family (left, right, full). `outer_join` is
unambiguous while it is the only outer variant. If `left_join`/`right_join` are ever added, add
`full_join` and deprecate `outer_join` (renaming a released item is breaking).

Yields every stretch covered by at least one input:
`(RangeInclusive<T>, (Option<VCL>, Option<VCR>))`, never `(None, None)`.

- Left and right outer joins are filters (`left.is_some()`), so they probably don't need their
  own methods.
- No new `ValueCarrier` impl is needed: `(Option<VCL>, Option<VCR>)` is a pair of `Option`
  carriers, both of which already exist.
- Implementation: extend `InnerJoinIterMap`'s merge loop to emit the one-sided prefix and
  remainder instead of discarding them. One pass, no allocation.

Already expressible today, less efficiently: `fill_gaps` makes each side total with `Option`
values, then `inner_join` and drop `(None, None)`:

```rust,no_run
# use range_set_blaze::prelude::*;
# let a = RangeMapBlaze::from_iter([(1..=5, "a")]);
# let b = RangeMapBlaze::from_iter([(4..=8, "b")]);
let outer = a
    .range_values()
    .fill_gaps()
    .inner_join(b.range_values().fill_gaps())
    .filter(|(_, (left, right))| left.is_some() || right.is_some());
```

This walks the whole key domain, including gaps covered by neither side, and the filter drops out
of `SortedDisjointMap`. A native join avoids both. The composed form compiles and gives the
expected answer (checked 2026-10-08: `(1..=3, (Some, None))`, `(4..=5, (Some, Some))`,
`(6..=8, (None, Some))`), so it makes a good test oracle for the native join.

Documentation should state that inner and outer agree when both inputs are universal, and that
`(None, None)` never occurs.

Real-world check (2026-10-08): on the glrmask fork, `union_compact_entries` (about 95 lines of
hand-written merge) and `combine_compact_entries` (a sort-and-sweep over all boundaries) were
rewritten as one `outer_join` pass feeding glrmask's existing builder. The hand-written versions
are kept as test-only references, and new exhaustive tests over all 4,096 pairs of small weights
show identical results for union, intersection, and difference. All 487 RSB-related glrmask tests
pass (commit `7891b1f85` on `local-rsb`). The closure shape needed no adaptation.

Second check: glrmask's hand-written `Weight::is_disjoint` (an inner-join loop) and
`Weight::is_subset` (an outer-join loop) became `inner_join(...).all(...)` and
`outer_join(...).all(...)`, with the originals kept as test-only references and an exhaustive
differential test (commit `d00731cee`). All RSB-related glrmask tests pass.

## Feature 1b: materialized joins on `RangeMapBlaze`

Decision: accepted and implemented (2026-10-08). `RangeMapBlaze::inner_join(&self, &other)` and
`RangeMapBlaze::outer_join(&self, &other)` return `RangeMapBlaze<T, (V, V2)>` and
`RangeMapBlaze<T, (Option<V>, Option<V2>)>`, cloning values, following the `fill_gaps` pattern
(lazy on iterators, materialized on the struct). No `RangeSetBlaze` forms and no owned
`into_*` forms for now.

## Feature 2: `transform_values`

Naming (2026-10-08): renamed from `map_values` before release, because in a crate whose main
type is a map, "map" reads as the noun (the data structure) rather than the verb (transform each
value).

Decision: struct form accepted and implemented (2026-10-08); iterator form pending.

`RangeMapBlaze::transform_values(&self, f: FnMut(&V) -> W) -> RangeMapBlaze<T, W>` returns a map with
the same keys and mapped values, merging touching ranges whose new values are equal. `f` is
called once per range in ascending key order, so stateful closures (such as assigning new state
IDs) are deterministic. It is built in one pass over the B-tree and keeps `len` without recounting.
Tests: doctests, edge cases (empty, gaps, merging up to the maximum key), call order, and a
quickcheck comparison against rebuilding with `from_iter`.

Real-world check (2026-10-08): `range-map-regex` commit `9890513` replaces every
`RangeMapBlaze::from_iter(a.range_values().inner_join(b.range_values()).map(...))` and its local
`map_values` helper with `a.inner_join(&b).transform_values(...)` (union, intersection, concat, star,
minimize, `subset_transition_map`). All 30 tests pass, and every example prints byte-identical
output (including state counts), so state numbering is unchanged. One cost: the materialized
`inner_join` clones values; in `concat` that is one extra `StateIdSet` clone per range.

Open questions for the iterator form:

- Output carrier: owned `W` has no `ValueCarrier`, and a blanket impl would conflict with the
  existing ones. Options: the closure returns a carrier (`W: ValueCarrier`), or the adapter wraps
  results in `Rc<W>` like `into_range_values`.
- Whether it is needed at all: both real callers so far only needed the struct form.

### Candidate 2b: filtering transform

Decision: considered and deferred (2026-10-08).

Like `transform_values`, but `f` returns `Option<W>` and ranges mapping to `None` are removed.
The only evidence is glrmask's `Weight::clip_tokens` (`#[doc(hidden)]`, behind its `internal-api`
feature, two call sites), which intersects each token set with `0..=max_token` and drops ranges
whose result is empty. Deferred because the evidence is thin, it adds public API, and it can be
added later without breaking anything. Revisit if a second real caller appears.

Workaround: with the streaming `transform_values` and the existing `SortedDisjointMap` impl for
`core::iter::Filter`, this is one lazy pass with no intermediate maps (checked 2026-10-08):

```rust,no_run
# use range_set_blaze::prelude::*;
# let map = RangeMapBlaze::from_iter([(1..=3u8, 10u8), (4..=6, 3), (7..=9, 12), (10..=12, 14)]);
let kept: RangeMapBlaze<u8, u8> = map
    .range_values()
    .transform_values(|value| (*value >= 10).then_some(value / 10))
    .filter(|(_, Owned(value))| value.is_some())
    .transform_values(|Owned(value)| value.expect("filtered to Some"))
    .into_range_map_blaze();
// (1..=3, 1), (7..=12, 1)
```

Filtering removes whole ranges, which leaves gaps, so the stream stays sorted, disjoint, and
merged. This leaves even less reason for a dedicated method; the remaining cost is the `expect`.

### Iterator form of `transform_values`

Decision: first deferred, then accepted and implemented (2026-10-08), so that later value-producing
stream operations (multiway joins) can be iterator-first with trivial struct wrappers.

`SortedDisjointMap::transform_values(f: FnMut(VC) -> W)` returns `TransformValuesIterMap`, a
`SortedDisjointMap<T, Owned<W>>` that merges touching ranges whose new values are equal and calls
`f` once per input range in key order.

New carrier `Owned<V>(pub V)`: a `ValueCarrier` that holds its value inline (`Value = V`). It
solves the owned-value carrier problem: closures create values that have no owner for `&W` to
borrow from, `Rc<W>` allocates per range, and a blanket `impl ValueCarrier for V` would conflict
with the existing carrier impls. Cloning an `Owned<V>` (when a downstream operation splits a range)
clones the `V`; for large values a closure can return an `Rc<V>` instead. Collecting a stream of
`Owned<V>` into a `RangeMapBlaze` yields `RangeMapBlaze<T, V>`. `Owned` is exported at the crate
root, not in the prelude.

Name decided (2026-10-08): `Owned`, matching std's `Cow::Borrowed`/`Cow::Owned` vocabulary for the
contrast with `&V`. Considered: `ByValue` (precise but clunky), `Inline` (wrong contrast for `&V`),
`Value` (collides with `ValueCarrier::Value`). Users cannot make their own value types carriers for
std types (orphan rule), and a blanket `impl ValueCarrier for V` would overlap the existing impls,
which is why a wrapper is needed. A `ValueCarrier` impl for `Cow<'a, V>` (mixed borrowed/owned
values) is possible and additive; deferred until a real use appears.

Values should be cheap to clone (they are cloned whenever a range splits); this is now stated in
the `RangeMapBlaze` docs. `Owned<V>` is cheap exactly when `V` is; large values use
`Owned<Rc<X>>`.

`Owned` is in the prelude (decided 2026-10-08).

### Layering principle

Decided (2026-10-08): every operation is iterator-first, with the `RangeMapBlaze` form as a thin
wrapper (`self.range_values().op(...).into_range_map_blaze()`). `inner_join`, `outer_join`, and
`transform_values` all follow this; the struct `transform_values` was converted from its own
B-tree build to the wrapper. Multiway joins should follow the same pattern.

## Feature 3: multiway joins (inner and outer)

Decision: accepted and implemented (2026-10-08) in `src/multiway_join_iter_map.rs`.

- Names: `inner_join` / `outer_join`, the same as the two-way joins (receivers differ).
- Iterator-first on `MultiwaySortedDisjointMap`; thin wrappers on `MultiwayRangeMapBlazeRef`
  (`[&a, &b, &c].inner_join(f)`). No owned-maps (`MultiwayRangeMapBlaze`) form for now.
- Closure: inner `FnMut(&[VC]) -> W`, outer `FnMut(&[Option<VC>]) -> W` (one slot per input, in
  input order; never all `None`). Through the struct wrappers, `VC = &V`. Results are carried by
  `Owned<W>`; touching ranges with equal results merge. No `V` is cloned: outer slots are moved in
  from the streams and borrowed in place; the inner join's dense buffer copies carriers (pointers,
  for `&V`) only for inputs that changed.
- Zero inputs: outer is empty; inner is the universal range with `f(&[])`, matching
  `RangeSetBlaze`'s zero-input intersection.
- Performance, from the start, matching the existing multiway union: one sweep using
  `KMergeMap` (heap over starts, tagged with input position) plus a min-heap of active ends.
  O(log k) per input range; the slot slice is updated in place, never rebuilt.
- Tests: brute-force per-key oracle under quickcheck (checked with 20,000 cases), merging,
  call order, zero/one inputs, maximum key.
- Real-world check: `range-map-regex` commit `8cd9a97` replaces `subset_transition_map`'s k − 1
  pairwise joins with one multiway `inner_join`; the zero-input rule replaces its explicit
  empty-set case. All tests pass and example output is byte-identical.

The design notes below record how this was reached.

k-way versions of features 1 and the existing inner join, alongside the existing
`MultiwayRangeMapBlaze` / `MultiwaySortedDisjointMap` traits. The existing multiway map `union` is
a priority union (one input's value wins); these would carry or combine all overlapping values.

Design decisions:

- **Item shape.** Each stretch has a variable number of values. A standard `Iterator` cannot lend
  a borrowed slice, so either allocate per item (`Vec` or `SmallVec`) or offer only a fold.
- **Fold form.** For example `union_with(|values| ...)`, yielding one value per stretch with no
  per-item list. Both motivating callers only fold, so this may be the version to build first, or
  the only one.
- **Positions.** Outer: `Option` per input position (which input contributed what) versus only the
  present values. A product construction may need positions; a fold usually doesn't.
- **Value types.** All inputs share one value type, unlike the two-way join's mixed pair.

### Draft for discussion (not decided)

Evidence from the callers:

- glrmask's `union_all_multiway_impl_with_token_cache` (`crates/glrmask-weight`) does not fold
  pairwise. At each stretch it passes the whole slice of active token sets to
  `union_active_token_sets(&[SharedTokenSet])`, because unioning many sets at once is cheaper
  (`shared_token_union_many`) and it caches results keyed on the slice's pointers. It also
  special-cases fully disjoint inputs (no overlaps: emit entries directly).
- `range-map-regex`'s `subset_transition_map` folds k universal maps into a set of next states, one
  materialized `inner_join` per extra input.

Proposed shape: one combining call per stretch that receives every value present there as a
slice, and returns the output value or `None` (drop the stretch). The iterator calls the closure
inside `next()` with a slice borrowed from a reused internal buffer, so it yields one owned value
per stretch: no lending iterator and no per-item allocation. Inner-join semantics are the special
case "return `None` unless all k inputs are present".

Sketch (names are placeholders):

```rust,ignore
// Struct form, on collections of maps (alongside MultiwayRangeMapBlaze / ...Ref):
let merged: RangeMapBlaze<T, W> =
    [&a, &b, &c].outer_join_with(|values: &[&V]| -> Option<W> { ... });

// Iterator form, on collections of SortedDisjointMap iterators:
// same closure, yields (RangeInclusive<T>, W) -- needs an answer to the owned-carrier question
// from feature 2.
```

Questions for the human:

1. **Present values only, or positions?** `&[&V]` (only inputs present at the stretch) fits both
   callers. `&[Option<&V>]` (length k, one slot per input) also tells which input contributed
   what, at the cost of filtering when unneeded. A product construction over k inputs would want
   positions.
2. **Names.** For example `outer_join_with` / `inner_join_with`, or `union_with` /
   `intersection_with` to sit beside the existing priority `union` / `intersection`.
3. **Struct form only first?** As with `transform_values`, both callers would be served by the struct
   form; the iterator form inherits feature 2's owned-carrier question.

Implementation outline: a k-way sweep. Each input is sorted and disjoint, so at most one range per
input covers any key. Keep each input's current range; the next boundary is the smallest of the
active ranges' ends + 1 and the upcoming starts. At each stretch, fill the buffer with the active
values and call the closure. That is O(k) per output stretch; a heap keyed on boundaries makes it
O(log k) for large k. The existing `KMergeMap` (merge by start, ties by input index) may be
reusable for the start ordering. Merge touching outputs with equal values, as `transform_values` does.
Test oracle: fold pairwise with `outer_join` + `transform_values`.

## Feature 4: materialized joins on `RangeMapBlaze`

Moved up and implemented as feature 1b.

## Not planned

- **Set joins.** The six optimized set operators already cover two-input set combinations.
- **New map-with-set joins.** `map_and_set_intersection` and `map_and_set_difference` already
  cover masking a map by a set.

## Suggested order

1. Done: feature 1 (outer join), feature 1b (materialized joins), feature 2 struct form
   (`transform_values`), and the `range-map-regex` port.
2. Done: deferred candidate 2b (filtering transform); added the iterator form of
   `transform_values` with the `Owned` carrier.
3. Done: feature 3 (multiway joins). Possible check: rewrite glrmask's `union_all_multiway*`
   on multiway `outer_join` in the fork.
