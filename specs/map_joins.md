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
- [glrmask](https://github.com/IsaacBreen/glrmask) (`src/ds/weight.rs`), grammar-constrained LLM
  decoding. Its `Weight` is `RangeMapBlaze<u32, Arc<RangeSetBlaze<u32>>>`, mapping state ranges to
  token sets. It hand-writes a two-way outer join with a combining closure
  (`combine_compact_entries`), a coalescing builder (`CompactRangeBuilder`), and a k-way union
  that unions overlapping token sets (`union_all_multiway`). It is version 0.1.1 on
  range-set-blaze 0.3.0, so treat it as a design signal, not demand.

## Background: why range-map-regex only needs inner join

A DFA's transition map is total: every symbol leads somewhere, even if only to a dead state.
`range-map-regex` builds each state with `universe_with` and asserts `is_universal()` in
`set_transitions`. The inner join of two universal maps is universal, so an outer join would never
yield a missing side there. Outer joins matter for partial maps such as glrmask's.

## Feature 1: two-way full outer join

Decision: accepted and implemented (2026-10-08) in `src/outer_join_iter_map.rs`. Name: `outer_join`. Item shape: `(Option<VCL>, Option<VCR>)`, no
new enum. No separate left/right methods for now.

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

## Feature 2: `map_values`

Decision: pending.

Apply a function to each value, merging adjacent ranges whose new values are equal, so the result
is still a valid `SortedDisjointMap`. Both motivating callers hand-write it.

- Iterator adapter on `SortedDisjointMap`, plus a `RangeMapBlaze` method returning a
  `RangeMapBlaze`.
- Without merging, `join(...).map(...)` leaves the `SortedDisjointMap` world, which is why every
  call site in `range-map-regex` goes through `RangeMapBlaze::from_iter`.

Open questions:

- Closure input: `&V` (logical value) or the carrier `VC`.
- Output carrier type: owned `W` values need a carrier; check what `ValueCarrier` impls exist for
  owned values, or whether the adapter yields `Rc<W>` or similar.
- Name: `map_values` versus something matching existing naming.

## Feature 3: multiway joins (inner and outer)

Decision: pending. Needs the most design.

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

## Feature 4: materialized joins on `RangeMapBlaze`

Decision: pending; low priority.

`RangeMapBlaze::inner_join(&other) -> RangeMapBlaze<T, (V, V2)>`, and an outer form. Decide after
seeing how `range-map-regex` reads with features 1 and 2; `collect` may be enough.
`src/inner_join_iter_map.rs` carries a related `todo000 consider adding to RMS`.

## Not planned

- **Set joins.** The six optimized set operators already cover two-input set combinations.
- **New map-with-set joins.** `map_and_set_intersection` and `map_and_set_difference` already
  cover masking a map by a set.

## Suggested order

1. Feature 1 (outer join) and feature 2 (`map_values`): small, immediately usable.
2. Port `range-map-regex`'s `union`, `intersection`, and `subset_transition_map` to them as a
   real-world check.
3. Design feature 3, fold form first, informed by that port.
4. Revisit feature 4.
