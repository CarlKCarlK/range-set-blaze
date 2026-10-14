//! # Joins
//!
//! A join lines up two or more range maps by key and combines their values wherever they
//! overlap. This guide covers the joins and their relatives: the two-way joins (`inner_join`,
//! `left_join`, `full_join`), the multiway joins, `transform_values`, and the low-level
//! `sweep` that they are built on.
//!
//! # Table of Contents
//! * [Joins versus set operations](#joins-versus-set-operations)
//! * [The operations at a glance](#the-operations-at-a-glance)
//! * [Choosing a join](#choosing-a-join)
//! * [What every join guarantees](#what-every-join-guarantees)
//! * [Maps versus streams](#maps-versus-streams)
//! * [Multiway joins](#multiway-joins)
//! * [Universal inputs](#universal-inputs)
//! * [`transform_values`](#transform_values)
//! * [`sweep`: the low-level primitive](#sweep-the-low-level-primitive)
//! * [Performance](#performance)
//!
//! ## Joins versus set operations
//!
//! On maps, a set operation such as [`intersection`][crate::SortedDisjointMap::intersection]
//! (`a & b`) decides which keys to keep and keeps *one* input's value for each: where inputs
//! overlap, the right-most input's value wins (for difference, `a - b`, the values come from
//! `a`). A join keeps or combines the values of *every* input:
//!
//! ```
//! use range_set_blaze::prelude::*;
//!
//! let a = RangeMapBlaze::from_iter([(1..=5, "a")]);
//! let b = RangeMapBlaze::from_iter([(4..=8, "b")]);
//!
//! // Intersection: the keys in both, with one map's value.
//! assert_eq!((&a & &b).to_string(), r#"(4..=5, "b")"#);
//!
//! // Inner join: the keys in both, with a value computed from both maps' values.
//! let joined = a.inner_join(&b, |l, r| format!("{l}{r}"));
//! assert_eq!(joined.to_string(), r#"(4..=5, "ab")"#);
//! ```
//!
//! ## The operations at a glance
//!
//! In this table, `a`, `b`, and `c` are [`RangeMapBlaze`][crate::RangeMapBlaze]s, and `s1`,
//! `s2`, and `s3` are [`SortedDisjointMap`][crate::SortedDisjointMap] streams, such as
//! `a.range_values()`.
//!
//! | Operation | Keys | Map form | Stream form |
//! |---|---|---|---|
//! | [inner join][inner] | in both | `a.inner_join(&b, \|l, r\| ..)` | `s1.inner_join(s2)` yields `(range, (l, r))` |
//! | [left join][left] | in the left | `a.left_join(&b, \|l, r\| ..)` | `s1.left_join(s2)` yields `(range, (l, Option<r>))` |
//! | [full join][full] | in either | `a.full_join(&b, \|l, r\| ..)` | `s1.full_join(s2)` yields `(range, (Option<l>, Option<r>))` |
//! | [multiway inner join][m_inner] | in all | `[&a, &b, &c].inner_join(\|values\| ..)` | `[s1, s2, s3].inner_join(\|values\| ..)` |
//! | [multiway full join][m_full] | in any | `[&a, &b, &c].full_join(\|values\| ..)` | `[s1, s2, s3].full_join(\|values\| ..)` |
//! | [transform values][transform] | same | `a.transform_values(\|v\| ..)` | `s1.transform_values(\|v\| ..)` |
//! | [sweep][sweep] | n/a | n/a | `[s1, s2, s3].sweep()` yields [`SweepEvent`][crate::SweepEvent]s |
//!
//! The map forms take a function and return a new `RangeMapBlaze`. The two-way stream joins
//! take no function: they yield pairs of borrowed values, which you can consume directly or pass
//! to [`transform_values`][transform]. The multiway stream joins take a function, like the map
//! forms.
//!
//! [inner]: crate::RangeMapBlaze::inner_join
//! [left]: crate::RangeMapBlaze::left_join
//! [full]: crate::RangeMapBlaze::full_join
//! [m_inner]: crate::MultiwayRangeMapBlazeRef::inner_join
//! [m_full]: crate::MultiwayRangeMapBlazeRef::full_join
//! [transform]: crate::SortedDisjointMap::transform_values
//! [sweep]: crate::MultiwaySortedDisjointMap::sweep
//!
//! ## Choosing a join
//!
//! * **Inner join**: the keys covered by *both* inputs (multiway: by *all* inputs).
//! * **Left join**: the keys covered by the left input, with the right input's value or `None`.
//!   It stops as soon as the left input ends, without reading the rest of the right input, so it
//!   suits questions about the left input, such as "is every left key covered by the right?".
//!   For a right join, swap the inputs.
//! * **Full join**: the keys covered by *either* input (multiway: by *any* input), with each
//!   input's value or `None`. The function never sees all values `None`.
//!
//! ```
//! use range_set_blaze::prelude::*;
//!
//! let a = RangeMapBlaze::from_iter([(1..=5, "a")]);
//! let b = RangeMapBlaze::from_iter([(4..=8, "b")]);
//!
//! let inner = a.inner_join(&b, |l, r| (*l, *r));
//! assert_eq!(inner.to_string(), r#"(4..=5, ("a", "b"))"#);
//!
//! let left = a.left_join(&b, |l, r| (*l, r.copied()));
//! assert_eq!(left.to_string(), r#"(1..=3, ("a", None)), (4..=5, ("a", Some("b")))"#);
//!
//! let full = a.full_join(&b, |l, r| (l.copied(), r.copied()));
//! assert_eq!(
//!     full.to_string(),
//!     r#"(1..=3, (Some("a"), None)), (4..=5, (Some("a"), Some("b"))), (6..=8, (None, Some("b")))"#
//! );
//! ```
//!
//! ## What every join guarantees
//!
//! * **One call per constant range, in key order.** The function is called once for each
//!   maximal range over which the inputs' values do not change, from the lowest keys up. It may
//!   be `FnMut` and keep state, for example to number new values as they first appear.
//! * **Canonical output.** Touching ranges with equal results are merged, so every result is a
//!   valid `RangeMapBlaze` or [`SortedDisjointMap`][crate::SortedDisjointMap], and two ranges
//!   whose input values differ but whose results are equal become one range.
//! * **Values should be cheap to clone.** Results are cloned whenever a range is split, as everywhere
//!   in this crate. Wrap large values in `Rc` or `Arc`.
//!
//! ```
//! use range_set_blaze::prelude::*;
//!
//! let a = RangeMapBlaze::from_iter([(0..=9, 1), (10..=19, 2)]);
//! let b = RangeMapBlaze::from_iter([(5..=14, 10)]);
//!
//! let mut calls = 0;
//! let present = a.full_join(&b, |l, r| {
//!     calls += 1;
//!     usize::from(l.is_some()) + usize::from(r.is_some())
//! });
//! // Four constant ranges: 0..=4, 5..=9, 10..=14, 15..=19.
//! assert_eq!(calls, 4);
//! // 5..=9 and 10..=14 have different values but the same result, so they merge.
//! assert_eq!(present.to_string(), "(0..=4, 1), (5..=14, 2), (15..=19, 1)");
//! ```
//!
//! ## Maps versus streams
//!
//! The map forms (on `RangeMapBlaze`) are the convenient choice. They clone only the function's
//! results into the new map, never the input values. Internally, each is a stream join followed
//! by `transform_values`.
//!
//! The stream forms (on [`SortedDisjointMap`][crate::SortedDisjointMap] streams, such as
//! [`RangeMapBlaze::range_values`][crate::RangeMapBlaze::range_values]) are lazy. They borrow
//! the values instead of cloning them, can stop early, and chain with other stream operations.
//! Use them when you don't need a new map, for example to answer a yes-or-no question:
//!
//! ```
//! use range_set_blaze::prelude::*;
//!
//! let a = RangeMapBlaze::from_iter([(1..=5, "a")]);
//! let b = RangeMapBlaze::from_iter([(4..=8, "b")]);
//!
//! // Is every key of `a` also in `b`? Stops at the first uncovered range.
//! let covered = a.range_values().left_join(b.range_values()).all(|(_, (_, r))| r.is_some());
//! assert!(!covered);
//!
//! // The map join equals the stream join followed by `transform_values`.
//! let streamed = a
//!     .range_values()
//!     .inner_join(b.range_values())
//!     .transform_values(|(l, r)| format!("{l}{r}"))
//!     .into_range_map_blaze();
//! assert_eq!(streamed, a.inner_join(&b, |l, r| format!("{l}{r}")));
//! ```
//!
//! Collecting a two-way stream join directly gives a map of owned pairs, cloning both values.
//! Usually it is better to compute what you need first, with the map form or `transform_values`,
//! so that equal results merge and only the results are cloned:
//!
//! ```
//! use range_set_blaze::prelude::*;
//!
//! let a = RangeMapBlaze::from_iter([(1..=5, "a")]);
//! let b = RangeMapBlaze::from_iter([(4..=8, "b")]);
//!
//! let pairs: RangeMapBlaze<i32, (&str, &str)> =
//!     a.range_values().inner_join(b.range_values()).into_range_map_blaze();
//! assert_eq!(pairs.to_string(), r#"(4..=5, ("a", "b"))"#);
//! ```
//!
//! ## Multiway joins
//!
//! The multiway joins take any number of inputs, all with the same value type. The function
//! receives a slice with one value per input, in input order: `&[&V]` for the inner join, and
//! `&[Option<&V>]` for the full join. The slice is updated in place between calls, not rebuilt.
//!
//! ```
//! use range_set_blaze::prelude::*;
//!
//! let a = RangeMapBlaze::from_iter([(0..=9, 1)]);
//! let b = RangeMapBlaze::from_iter([(5..=14, 10)]);
//! let c = RangeMapBlaze::from_iter([(8..=20, 100)]);
//!
//! let sums = [&a, &b, &c].full_join(|values| values.iter().flatten().copied().sum::<i32>());
//! assert_eq!(
//!     sums.to_string(),
//!     "(0..=4, 1), (5..=7, 11), (8..=9, 111), (10..=14, 110), (15..=20, 100)"
//! );
//! ```
//!
//! There is no multiway left join: use the full join and check `values[0]`. With zero inputs,
//! the full join is empty and the inner join covers every key with the value `f(&[])`.
//!
//! For inputs with different value types, chain two-way joins, or first give them a common type
//! with `transform_values`.
//!
//! The multiway joins take references to maps (`[&a, &b]`), not owned maps: a join builds a new
//! map and only reads its inputs, so owning them would gain nothing.
//!
//! ## Universal inputs
//!
//! A map is universal when it covers every key (see
//! [`RangeMapBlaze::is_universal`][crate::RangeMapBlaze::is_universal]). When both inputs are
//! universal, the inner and full joins give the same ranges, with every value present. To make a
//! map universal, fill its gaps with
//! [`RangeMapBlaze::fill_gaps`][crate::RangeMapBlaze::fill_gaps] (values become `Option`s), or
//! start from [`RangeMapBlaze::universe_with`][crate::RangeMapBlaze::universe_with]. See the
//! [Ranges and gaps guide][crate::gaps] for more on gaps.
//!
//! ```
//! use range_set_blaze::prelude::*;
//!
//! let a = RangeMapBlaze::from_iter([(1u8..=5, "a")]);
//! let b = RangeMapBlaze::from_iter([(4u8..=8, "b")]);
//!
//! let joined = a.fill_gaps().inner_join(&b.fill_gaps(), |l, r| (*l, *r));
//! assert!(joined.is_universal());
//! assert_eq!(
//!     joined.to_string(),
//!     r#"(0..=0, (None, None)), (1..=3, (Some("a"), None)), (4..=5, (Some("a"), Some("b"))), (6..=8, (None, Some("b"))), (9..=255, (None, None))"#
//! );
//! ```
//!
//! ## `transform_values`
//!
//! [`transform_values`][crate::RangeMapBlaze::transform_values] keeps the keys and replaces each
//! value with `f` applied to it, merging touching ranges whose new values are equal. It follows
//! the same rules as the joins (one call per range, in key order). On a stream it yields
//! [`Owned`][crate::Owned] values, which hold the new values directly; collecting into a
//! `RangeMapBlaze` removes the wrapper.
//!
//! ```
//! use range_set_blaze::prelude::*;
//!
//! let map = RangeMapBlaze::from_iter([(1..=3, 10), (4..=6, 11), (8..=9, 20)]);
//! assert_eq!(map.transform_values(|v| v / 10).to_string(), "(1..=6, 1), (8..=9, 2)");
//!
//! let mut stream = map.range_values().transform_values(|v| v / 10);
//! assert_eq!(stream.next(), Some((1..=6, Owned(1))));
//! ```
//!
//! ## `sweep`: the low-level primitive
//!
//! [`MultiwaySortedDisjointMap::sweep`][sweep] yields every input range's start and end as
//! [`SweepEvent`][crate::SweepEvent]s, in key order. The multiway joins are built on it. Use it
//! for computations the joins don't cover, such as overlap depth, running aggregates, or custom
//! precedence among inputs.
//!
//! A range ending at key `p - 1` ends before a range starting at `p` starts, so between
//! consecutive events the set of active ranges is constant:
//!
//! ```
//! use range_set_blaze::{SweepEvent, prelude::*};
//!
//! let a = RangeMapBlaze::from_iter([(0..=4, "a")]);
//! let b = RangeMapBlaze::from_iter([(5..=9, "b")]);
//!
//! let events: Vec<_> = [a.range_values(), b.range_values()].sweep().collect();
//! assert_eq!(
//!     events,
//!     vec![
//!         SweepEvent::Start { range: 0..=4, input: 0, value: &"a" },
//!         SweepEvent::End { at: 4, input: 0 },
//!         SweepEvent::Start { range: 5..=9, input: 1, value: &"b" },
//!         SweepEvent::End { at: 9, input: 1 },
//!     ]
//! );
//! ```
//!
//! ## Performance
//!
//! Every join, transform, and sweep makes one pass through its inputs. The two-way joins cost
//! O(1) per input range. The multiway joins and `sweep` cost O(log k) per input range for k
//! inputs. The map forms then build the result `RangeMapBlaze` from sorted, disjoint ranges.
