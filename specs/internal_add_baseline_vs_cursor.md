# `internal_add`: baseline vs. cursor

Both versions do the same job: insert `start..=end` into the
`BTreeMap<T, T>` that backs `RangeSetBlaze` (keyed by each stored range's
`start`, valued by its `end`), merging with any neighbor that overlaps or
touches. They just walk the map differently. This note is a map of how the
two control-flow shapes correspond to each other, so a change in one can be
checked against the other by eye.

Code: [`src/set.rs`](src/set.rs), `internal_add_baseline` (~line 1145) and
`internal_add_cursor` (~line 1219).

## The core difference: where do you stand relative to `start`?

The baseline uses `BTreeMap::range_mut(..=start).rev()` to look up the
**last stored range whose start is `<= start`** — call it "before". A single
lookup can therefore land *on* a range that begins exactly at `start`:

```
baseline: before = last key <= start

keys:     ... 1     4     9 ...
                     ^
              insert start=4  →  before = (4, ...)   (before.start == start)
```

The cursor version instead does `lower_bound_mut(Included(&start))`, which
parks the cursor *at* the first entry with key `>= start`. That means
`cursor.peek_prev()` only ever sees a key **strictly less than** `start` —
it can never be the entry that starts exactly at `start`.

```
cursor:  lower_bound(start) → cursor sits here
                               |
keys:     ... 1     4     9 ...
                     ^
              insert start=4  →  peek_prev() = (1,...)   peek_next() = (4,...)
```

So the one case the baseline handles in a single branch (`before.start ==
start`) is split into two branches in the cursor version: a `peek_prev`
check for a range that starts *before* `start`, and a separate `peek_next`
check for a range that starts *at* `start`. Everything downstream is a
reshuffling of the same three outcomes baseline already has: **gap /
touch-or-overlap-but-not-contained / fully contained**.

## Side-by-side flow

```
BASELINE                                   CURSOR
--------                                   ------
before = last key <= start                 cursor = lower_bound(start)
                                            prev = cursor.peek_prev()   (key < start)

if before is None:                         if prev interacts with start:
    internal_add2(range)                       if prev.end >= end: return  (contained)
    (insert; delete_extra merges forward)       else: extend prev.end = end
                                                        absorb successors
                                                        return
elif before.end + 1 < start:               if prev doesn't interact (or is None):
    # true gap, no touch                       next = cursor.peek_next()  (key >= start)
    internal_add2(range)                       if next.start == start && next.end >= end:
    (insert; delete_extra merges forward)          return   (contained, exact start match)
                                                 else:
elif before.end < end:                              absorb_successors(from cursor)
    # touches/overlaps, not contained               insert_before(start, pending_end)
    extend before.end = end                         return
    delete_extra(before.start..=end)
    # (merges forward, same as absorb_successors)

else:
    # before.end >= end: fully contained
    do nothing
```

Reading down the right column: the `prev` branch covers baseline's
"before is not None" cases where `before.start < start` (gap / touch /
overlap / contained, decided by comparing `prev.end` to `start` and `end`).
The `next`-based branch covers what's left: no interacting predecessor, so
either nothing needs to change (an exact-start range already covers the
insertion) or we merge forward from the cursor — which is baseline's
`internal_add2` + `delete_extra` combo, just done by removing-and-reinserting
instead of insert-then-merge.

## Merging forward: `delete_extra` vs. `cursor_absorb_successors`

Both scan later entries and eat every one that overlaps or touches the
range being inserted, growing `end`/`pending_end` to the max as they go,
and keeping `len` in sync by subtracting each absorbed range's length.

```
delete_extra(start..=end):                 cursor_absorb_successors(cursor, len, pending_end):
  for (s, e) in map.range_mut(start..)        while let Some((s, e)) = cursor.peek_next():
       .skip(1):                                  if s > pending_end + 1: break   (no touch)
      if s <= end || s <= end+1:                  cursor.remove_next()
          end = max(end, e)                       len -= len(s..=e)
          len -= len(s..=e)                        pending_end = max(pending_end, e)
          mark s for deletion                  if we grew past where we started
  *stored_end = end                                and the predecessor is the one
  remove marked keys                               we extended: write it back, add len
```

The one thing `cursor_absorb_successors` has that `delete_extra` doesn't is
the `pending_is_stored` flag: when we got here by *extending* the
predecessor in place (rather than inserting a fresh entry), the merged
range's `end` already lives in the map as `*stored_end_mut`. After
absorbing, if a successor reached farther than the predecessor's first
extension, that same stored slot has to be updated again. Baseline doesn't
need this distinction because `delete_extra` always writes back to
`*end_after` at the end regardless of how it got there.

## Worked examples

Stored: `[1..=3]`, `[7..=9]`. Insert `4..=6` (touches both neighbors,
merges everything into one range).

```
before:   [1,3]        [7,9]
                4..6
insert:   [1,3]  [4,6]  [7,9]
touches --^        ^-- touches

baseline:
  before = (1,3)  (last key <= 4)
  before.end(3)+1 == 4 == start → not a true gap → falls to "before.end < end" (3 < 6)
  extend before.end = 6        → map: [1,6] [7,9]
  delete_extra(1..=6):
     next entry (7,9): 7 <= 6+1  → absorbs it, end_new = 9
     *end_after = 9              → map: [1,9]

cursor:
  cursor = lower_bound(4)   →  positioned at (7,9)
  prev = peek_prev() = (1,3);  interacts: 3+1==4 → true
    3 < end(6) → extend: *stored_end_mut = 6      → map: [1,6] [7,9]
    cursor_absorb_successors(pending_end=6, pending_is_stored=true):
       peek_next = (7,9): 7 <= 6+1 → interacts, remove it, pending_end = 9
       pending_end(9) > initial(6) → write back predecessor's end = 9
                                    → map: [1,9]
  same end state, same len delta (added 4,5,6 then 7,8,9 either way)
```

Stored: `[2..=8]`. Insert `4..=6` (fully contained — no-op case).

```
baseline:
  before = (2,8); before.end(8) >= end(6) → "completely contained, do nothing"

cursor:
  cursor = lower_bound(4) → positioned just past (2,8) (next key, if any)
  prev = peek_prev() = (2,8); interacts (8 >= 4)
    prev.end(8) >= pending_end(6) → return immediately, no mutation
```

Stored: `[1..=2]`, `[7..=8]`. Insert `4..=5` (clean gap, touches nothing).

```
baseline:
  before = (1,2); before.end(2)+1 = 3 < start(4) → true gap
  internal_add2(4..=5): insert (4,5) fresh
  delete_extra(4..=5): next entry is (7,8); 7 <= 5+1? no (7 > 6) → stop, nothing absorbed
  map: [1,2] [4,5] [7,8]

cursor:
  cursor = lower_bound(4) → positioned at (7,8)
  prev = peek_prev() = (1,2); interacts? 2 >= 4? no. 2+1==4? no → doesn't interact
  peek_next() = (7,8); start(7) != 4 → no exact-match shortcut
  cursor_absorb_successors(pending_end=5, pending_is_stored=false):
     peek_next = (7,8): 7 <= 5+1(=6)? no → stop, nothing absorbed
  insert_before(4, 5)
  map: [1,2] [4,5] [7,8]
```

## Why they should always agree

Both algorithms are answering the same three questions about the insertion
`start..=end` relative to whatever is already stored — *is there anything
touching or overlapping on the left? on the right, going forward? does
something already fully cover it?* — using the same touch/overlap test
(`stored_end >= start` or `stored_end + 1 == start`, applied symmetrically
on both sides) and the same overflow-safe arithmetic (`checked_add_one`,
`safe_len` over half-open-shifted ranges to dodge `T::MAX` overflow). The
cursor version just answers "is there a range starting exactly at `start`"
as its own branch instead of folding it into the left-neighbor lookup,
because `lower_bound_mut` can't hand back that entry as a "previous" one.

This is exactly what the differential tests in
[`src/tests_set.rs`](src/tests_set.rs) (and the `_map` equivalents) check
directly: run both functions from the same starting `RangeSetBlaze` and
assert equal resulting map, `len()`, and `len_slow()` — exhaustively over
small domains, randomly over 20k incremental insertions, and against a
list of hand-picked edge cases (touch-both-sides, exact containment, `u8`
and `char` boundary values, the UTF-16 surrogate gap, and the full domain).

---

## The map side: everything above, plus a value

`RangeMapBlaze::internal_add(range, value)` ([`src/map.rs`](src/map.rs),
`internal_add_baseline` ~line 1523, `internal_add_cursor` ~line 1833) has
the same left/right structure as the set version — a predecessor question
and a forward-merge scan — but a stored range can now only merge with a
neighbor **if the values also match**. If they don't match, overlapping
regions have to be *split* instead of merged: the old range gets trimmed,
possibly leaving a residual chunk behind it.

That one extra dimension (`same_value`) is why `internal_add_baseline` is
~200 lines of nested `if`s covering combinations of `before_contains_new`
× `same_value` × `same_start` × `same_end` × "is there an interesting
before-before" — and why the cursor rewrite pulls the decision logic out
into two small, table-like classifier functions that don't touch the
`BTreeMap` at all:

```rust
fn classify_predecessor(stored_end, pending_start, pending_end, same_value) -> PredecessorInsertAction
fn classify_forward(stored_start, stored_end, pending_end, same_value) -> Option<ForwardInsertAction>
```

`internal_add_cursor` then just does what each classification says. The
mechanical B-tree work (removing an entry, writing back an `end`,
inserting a residual) is identical regardless of *why* — that separation
is what makes this version checkable at a glance instead of branch by
branch.

### Predecessor: three outcomes instead of two

The set version's predecessor check only asks "does it interact?" (gap vs.
touch/overlap). The map version adds a middle case: it can interact *and*
still need to be cut in two, because the value differs.

```
classify_predecessor(stored_end, pending_start, pending_end, same_value):

  no overlap, no touch  ──────────────────────▶  Unaffected
  (stored_end + 1 < pending_start)                 do nothing

  overlaps or touches,
  same_value             ─────────────────────▶  MergeSameValue
                                                    stored_end >= pending_end? → return (contained)
                                                    else: extend stored_end = pending_end,
                                                          pending_start = stored_start (adopt it)

  overlaps or touches,
  different value         ────────────────────▶  KeepLeftResidual { left_end, right_start }
                                                    trim stored range to left_end = pending_start - 1
                                                    if stored_end > pending_end:
                                                        remember a right_start residual
                                                        (the tail of the old range, beyond pending_end)
```

Picture: predecessor `[BBBBBBB]` (different value), inserting `aaa` that
starts partway through it and ends before it does:

```
keys:    ...  B  B  B  B  B  B  B  ...
                    a  a  a
                    ^start      ^end

classify_predecessor → KeepLeftResidual { left_end, right_start: Some(..) }

after:   ...  B  B |gone| a  a  a |gone| B  ...
              (trimmed to left_end)      (right residual, reinserted later)
```

### Forward scan: the same three shapes, mirrored

`cursor_scan_forward` walks `peek_next()` the way `cursor_absorb_successors`
does for sets, but each candidate gets the same three-way classification —
just from the other side:

```
classify_forward(stored_start, stored_end, pending_end, same_value):

  no overlap, no touch   ─────────────────────▶  None (stop scanning)

  overlaps or touches,
  same_value              ────────────────────▶  MergeSameValue
                                                    pending_end = max(pending_end, stored_end)
                                                    (removed; loop continues)

  overlaps or touches,
  different value,
  stored_end <= pending_end ──────────────────▶  DeleteOverwritten
                                                    (fully covered; removed; loop continues)

  overlaps or touches,
  different value,
  stored_end > pending_end  ──────────────────▶  KeepRightResidual { right_start }
                                                    (removed, but its tail is remembered
                                                     and reinserted; loop stops here —
                                                     nothing past it can touch)
```

### The key simplification: no separate "before-before" case

Baseline's hardest cases are the ones where the range starting *exactly*
at `start` has a *different* value than the insertion, but something
further left — the "before-before" — has the *same* value and touches at
`start`. Baseline has to explicitly look one step further back
(`before_iter.next()`) and special-case merging into it (see the
`interesting_before_before` blocks at
[map.rs:1582](src/map.rs#L1582) and [map.rs:1681](src/map.rs#L1681)).

The cursor version never needs that special case, for the same structural
reason as the set version: `lower_bound_mut(Included(&start))` means
`peek_prev()` **only ever sees a key strictly less than `start`** — an
entry that starts exactly at `start` is never the "predecessor", it's the
first thing the forward scan sees. So:

* If the true predecessor (baseline's "before-before") shares the new
  value, `classify_predecessor` merges into *it* directly
  (`pending_is_stored = true`, `pending_start` snaps back to that
  predecessor's start) — exactly the outcome baseline reaches by manually
  hunting for an interesting before-before.
* The entry that used to sit at exactly `start` is then just the first
  candidate the forward scan sees, classified like any other successor
  (its value differs from the new value, so it gets trimmed to a residual
  or deleted like anything else in its way).

One cursor lookup (`peek_prev`) plus one generic scan (`peek_next` in a
loop) does the job of baseline's four separate same-start branches
([map.rs:1579-1616](src/map.rs#L1579-L1616) and
[map.rs:1679-1738](src/map.rs#L1679-L1738)).

### Worked example: a value-split sandwich

This is `map_repro1` in [`src/tests_map.rs`](src/tests_map.rs) — stored
`(20..=21, "a")`, `(24..=29, "b")`; insert `25..=25` with `"c"`.

```
keys:     20 21        24 25 26 27 28 29
values:    a  a         b  .  .  .  .  b
insert:                    c
                            ^25..=25, "c"

expected: (20..=21,"a") (24..=24,"b") (25..=25,"c") (26..=29,"b")
```

```
baseline:
  before = (24, end=29, "b")     # last key <= 25
  same_start? 24 != 25 → no
  before_contains_new: 29 >= 25 → yes
  same_value: "b" == "c"? → no
  same_end: 25 == 29? → no
  → "before contains new, different value, not same start/end" branch:
       end_value_before.end = 24           # trim: (24..=24, "b")
       insert (25, {end:25, value:"c"})     # the new range
       insert (26, {end:29, value:"b"})     # right residual (old tail)

cursor:
  cursor = lower_bound(25)              # positioned past key 24 (no key >= 25)
  peek_prev() = (24, end=29, "b")
  classify_predecessor(stored_end=29, pending_start=25, pending_end=25, same_value=false):
       overlaps: 29 >= 25 → true
       different value, stored_end(29) > pending_end(25) → right_start = Some(26)
       ⇒ KeepLeftResidual { left_end: 24, right_start: Some(26) }
  apply:
       end_value(24 entry).end = 24         # trim: (24..=24, "b")
       right_residual = Some((26, {end:29, value:"b"}))
  right_residual is Some → skip the forward scan
       (nothing past key 24 can still be touching key 25..=29's old
        territory — it was all inside the range just trimmed)
  pending_is_stored == false → insert (25, {end:25, value:"c"})
  insert right_residual: (26, {end:29, value:"b"})

both:     (20..=21,"a") (24..=24,"b") (25..=25,"c") (26..=29,"b")   ✓
```

Note both versions skip re-scanning forward here, for the same reason:
whatever used to be at `26..=29` was already fully accounted for as part
of the *old* `24..=29` range, and by `RangeMapBlaze`'s own canonical-form
invariant (no two touching stored ranges ever share a value), whatever
comes after `29` can't have the same value as the `"b"` residual either —
so there's nothing left to merge. Baseline encodes this by simply not
calling `delete_extra` in that branch; the cursor version encodes it by
only running `cursor_scan_forward` `if right_residual.is_none()`.

### Confidence

Same story as the set side: [`src/tests_map.rs`](src/tests_map.rs) has the
matching trio — `map_cursor_insert_targeted_differential` (hand-picked
edge cases: exact-range, contained-same-value, contained-different-value,
left/right-edge trims, multi-range sweeps, u128 boundaries),
`map_cursor_insert_exhaustive_small_domain`, and
`map_cursor_insert_randomized_differential` — each building a
`RangeMapBlaze` two ways (`internal_add_baseline` vs. `internal_add_cursor`)
from the same inputs and asserting the resulting map, `len()`, and
`len_slow()` all agree. All pass, including with `debug_assert!(self.len
== self.len_slow())` active on every call.
