// https://docs.rs/range_bounds_map/latest/range_bounds_map/range_bounds_set/struct.RangeBoundsSet.html
// Here are some relevant crates I found whilst searching around the topic area:

// https://crates.io/crates/sorted-iter
//    cmk0 Look at sorted-iter's note about exporting.
//    cmk0 Look at sorted-iter's note about their testing tool.
// https://docs.rs/rangemap Very similar to this crate but can only use Ranges and RangeInclusives as keys in it's map and set structs (separately).
// https://docs.rs/btree-range-map
// https://docs.rs/ranges Cool library for fully-generic ranges (unlike std::ops ranges), along with a Ranges data structure for storing them (Vec-based unfortunately)
// https://docs.rs/intervaltree Allows overlapping intervals but is immutable unfortunately
// https://docs.rs/nonoverlapping_interval_tree Very similar to rangemap except without a gaps() function and only for Ranges and not RangeInclusives. And also no fancy coalescing functions.
// https://docs.rs/unbounded-interval-tree A data structure based off of a 2007 published paper! It supports any RangeBounds as keys too, except it is implemented with a non-balancing Box<Node> based tree, however it also supports overlapping RangeBounds which my library does not.
// https://docs.rs/rangetree I'm not entirely sure what this library is or isn't, but it looks like a custom red-black tree/BTree implementation used specifically for a Range Tree. Interesting but also quite old (5 years) and uses unsafe.
// https://docs.rs/btree-range-map/latest/btree_range_map/
// Related: https://lib.rs/crates/iset
// https://lib.rs/crates/interval_tree
// https://lib.rs/crates/range-set
// https://lib.rs/crates/rangemap
// https://lib.rs/crates/ranges
// https://lib.rs/crates/nonoverlapping_interval_tree

// !!!cmk0 how could you write your own subtraction that subtracted many sets from one set via iterators?
// !!!cmk0 sort by start then by larger stop

mod merger;
mod safe_subtract;
mod tests;

use gen_ops::gen_ops_ex;
use itertools::Itertools;
use itertools::KMergeBy;
use itertools::MergeBy;
use merger::Merger;
use num_traits::Zero;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use sorted_iter::sorted_pair_iterator::SortedByKey;
use std::cmp::max;
use std::collections::btree_map;
use std::collections::BTreeMap;
use std::convert::From;
use std::fmt;
use std::ops::{BitOrAssign, Sub};
use std::str::FromStr;
use trait_set::trait_set;

// cmk rule: Support Send and Sync (what about Clone (Copy?) and ExactSizeIterator?)
// cmk rule: Use trait_set

trait_set! {
    pub trait Integer =
    num_integer::Integer
    + FromStr
    + fmt::Display
    + fmt::Debug
    + std::iter::Sum
    + num_traits::NumAssignOps
    + FromStr
    + Copy
    + num_traits::Bounded
    + num_traits::NumCast
    + SafeSubtract
+ Send + Sync
    ;
}

pub trait SafeSubtract {
    // type Upscale;
    type Output: std::hash::Hash
        + num_integer::Integer
        + std::ops::AddAssign
        + std::ops::SubAssign
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Send
        + Default;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output;
    fn safe_subtract_inclusive(stop: Self, start: Self) -> <Self as SafeSubtract>::Output;
    fn max_value2() -> Self;
}

pub fn fmt<T: Integer>(items: &BTreeMap<T, T>) -> String {
    items
        .iter()
        .map(|(start, stop)| format!("{start}..={stop}"))
        .join(",")
}

// !!!cmk can I use a Rust range?
// !!!cmk0 could it use the sorted ranges and automatically convert from/to RangeIntSet at the end?

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RangeSetInt<T: Integer> {
    len: <T as SafeSubtract>::Output,
    items: BTreeMap<T, T>,
}

// !!!cmk support =, and single numbers
// !!!cmk error to use -
// !!!cmk are the unwraps OK?
// !!!cmk what about bad input?

impl<T: Integer> From<&str> for RangeSetInt<T>
where
    // !!! cmk understand this
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    fn from(s: &str) -> Self {
        Merger::from_iter(s.split(',').map(|s| {
            let mut range = s.split("..=");
            let start = range.next().unwrap().parse::<T>().unwrap();
            let stop = range.next().unwrap().parse::<T>().unwrap();
            (start, stop)
        }))
        .into()
    }
}

impl<T: Integer> fmt::Debug for RangeSetInt<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", fmt(&self.items))
    }
}

impl<T: Integer> fmt::Display for RangeSetInt<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", fmt(&self.items))
    }
}

impl<T: Integer> RangeSetInt<T> {
    pub fn iter(&self) -> Iter<T, impl Iterator<Item = (T, T)> + '_> {
        let i = self.ranges();
        Iter {
            current: T::zero(),
            option_range: OptionRange::None,
            range_iter: i,
        }
    }

    fn from_sorted_distinct_iter<I>(sorted_distinct_iter: I) -> Self
    where
        I: Iterator<Item = (T, T)>,
    {
        let mut len = <T as SafeSubtract>::Output::zero();
        let sorted_distinct_iter2 = sorted_distinct_iter.map(|(start, stop)| {
            len += T::safe_subtract_inclusive(stop, start);
            (start, stop)
        });

        let items = BTreeMap::<T, T>::from_iter(sorted_distinct_iter2);
        RangeSetInt::<T> { items, len }
    }
}

// !!!cmk00 support iterator instead of slices?
impl<T: Integer> RangeSetInt<T> {
    // !!!cmk0 should part of this be a method on BitOrIter?
    pub fn union<U: AsRef<[RangeSetInt<T>]>>(slice: U) -> Self {
        RangeSetInt::from_sorted_distinct_iter(slice.as_ref().iter().map(|x| x.ranges()).union())
    }

    // !!!cmk00 these should work on iterators not slices
    pub fn intersection<U: AsRef<[RangeSetInt<T>]>>(slice: U) -> Self {
        RangeSetInt::from_sorted_distinct_iter(
            slice.as_ref().iter().map(|x| x.ranges()).intersection(),
        )
    }

    /// !!! cmk understand the 'where for'
    /// !!! cmk understand the operator 'Sub'
    fn _len_slow(&self) -> <T as SafeSubtract>::Output
    where
        for<'a> &'a T: Sub<&'a T, Output = T>,
    {
        self.items
            .iter()
            .fold(<T as SafeSubtract>::Output::zero(), |acc, (start, stop)| {
                acc + T::safe_subtract_inclusive(*stop, *start)
            })
    }

    /// Moves all elements from `other` into `self`, leaving `other` empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut a = RangeSetInt::from("1..=3");
    /// let mut b = RangeSetInt::from("3..=5");
    ///
    /// a.append(&mut b);
    ///
    /// assert_eq!(a.len(), 5usize);
    /// assert_eq!(b.len(), 0usize);
    ///
    /// assert!(a.contains(1));
    /// assert!(a.contains(2));
    /// assert!(a.contains(3));
    /// assert!(a.contains(4));
    /// assert!(a.contains(5));
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        for (start, stop) in other.ranges() {
            self.internal_add(start, stop);
        }
        other.clear();
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.len = <T as SafeSubtract>::Output::zero();
    }

    /// Returns `true` if the set contains an element equal to the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let set = RangeSetInt::from([1, 2, 3]);
    /// assert_eq!(set.contains(1), true);
    /// assert_eq!(set.contains(4), false);
    /// ```
    pub fn contains(&self, value: T) -> bool {
        self.items
            .range(..=value)
            .next_back()
            .map_or(false, |(_, stop)| value <= *stop)
    }

    fn delete_extra(&mut self, start: T, stop: T) {
        let mut after = self.items.range_mut(start..);
        let (start_after, stop_after) = after.next().unwrap(); // there will always be a next
        debug_assert!(start == *start_after && stop == *stop_after); // real assert
                                                                     // !!!cmk would be nice to have a delete_range function
        let mut stop_new = stop;
        let delete_list = after
            .map_while(|(start_delete, stop_delete)| {
                // must check this in two parts to avoid overflow
                if *start_delete <= stop || *start_delete <= stop + T::one() {
                    stop_new = max(stop_new, *stop_delete);
                    self.len -= T::safe_subtract_inclusive(*stop_delete, *start_delete);
                    Some(*start_delete)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if stop_new > stop {
            self.len += T::safe_subtract(stop_new, stop);
            *stop_after = stop_new;
        }
        for start in delete_list {
            self.items.remove(&start);
        }
    }

    pub fn insert(&mut self, item: T) {
        self.internal_add(item, item);
    }

    // https://stackoverflow.com/questions/49599833/how-to-find-next-smaller-key-in-btreemap-btreeset
    // https://stackoverflow.com/questions/35663342/how-to-modify-partially-remove-a-range-from-a-btreemap
    fn internal_add(&mut self, start: T, stop: T) {
        // internal_add(&mut self.items, &mut self.len, start, stop);
        assert!(start <= stop && stop <= T::max_value2());
        // !!! cmk would be nice to have a partition_point function that returns two iterators
        let mut before = self.items.range_mut(..=start).rev();
        if let Some((start_before, stop_before)) = before.next() {
            // Must check this in two parts to avoid overflow
            if *stop_before < start && *stop_before + T::one() < start {
                self.internal_add2(start, stop);
            } else if *stop_before < stop {
                self.len += T::safe_subtract(stop, *stop_before);
                *stop_before = stop;
                let start_before = *start_before;
                self.delete_extra(start_before, stop);
            } else {
                // completely contained, so do nothing
            }
        } else {
            self.internal_add2(start, stop);
        }
    }

    fn internal_add2(&mut self, start: T, stop: T) {
        let was_there = self.items.insert(start, stop);
        debug_assert!(was_there.is_none()); // real assert
        self.delete_extra(start, stop);
        self.len += T::safe_subtract_inclusive(stop, start);
    }

    pub fn len(&self) -> <T as SafeSubtract>::Output {
        self.len.clone()
    }

    pub fn new() -> RangeSetInt<T> {
        RangeSetInt {
            items: BTreeMap::new(),
            len: <T as SafeSubtract>::Output::zero(),
        }
    }

    // !!!cmk0 add .map(|(start, stop)| (*start, *stop)) to ranges()?
    pub fn ranges(&self) -> Ranges<'_, T>
// impl Iterator<Item = (T, T)> + ExactSizeIterator<Item = (T, T)> + Clone + '_
    {
        let ranges = Ranges {
            items: self.items.iter(),
        };
        ranges
    }

    pub fn ranges_len(&self) -> usize {
        self.items.len()
    }
}

#[derive(Clone)]
pub struct Ranges<'a, T: Integer> {
    items: btree_map::Iter<'a, T, T>,
}

impl<T: Integer> SortedByKey for Ranges<'_, T> {}
impl<T: Integer> ExactSizeIterator for Ranges<'_, T> {
    fn len(&self) -> usize {
        self.items.len()
    }
}

impl<'a, T: Integer> Iterator for Ranges<'a, T> {
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(|(start, stop)| (*start, *stop))
    }
}

// !!!cmk00 don't use this or from_iter explicitly. Instead use 'collect'
impl<T: Integer> FromIterator<T> for RangeSetInt<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Merger::from_iter(iter).into()
    }
}

// impl<I, T: Integer> FromIterator<I> for RangeSetInt<T>
// where
//     I: IntoIterator<Item = (T, T)> + SortedByKey,
// {
//     RangeSetInt::from_sorted_distinct_iter(iter.into_iter())
// }

// !!!cmk00 what about combos?
impl<T: Integer> BitOrAssign<&RangeSetInt<T>> for RangeSetInt<T> {
    /// Returns the union of `self` and `rhs` as a new `RangeSetInt`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut a = RangeSetInt::from([1, 2, 3]);
    /// let b = RangeSetInt::from([3, 4, 5]);
    ///
    /// a |= &b;
    /// assert_eq!(a, RangeSetInt::from([1, 2, 3, 4, 5]));
    /// assert_eq!(b, RangeSetInt::from([3, 4, 5]));
    /// ```
    fn bitor_assign(&mut self, rhs: &Self) {
        for (start, stop) in rhs.ranges() {
            self.internal_add(start, stop);
        }
    }
}

#[derive(Clone)]
pub struct BitOrIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)> + Clone,
{
    merged_ranges: I,
    range: Option<(T, T)>,
}

// !!!cmk0 should I0,I1 be I,J to match itertools?
pub type BitOrIterOfMergeBy<T, I0, I1> = BitOrIter<T, MergeByRanges<T, I0, I1>>;
pub type BitOrIterOfKMergeBy<T, I> = BitOrIter<T, KMergeByRanges<T, I>>;
pub type MergeByRanges<T, I0, I1> = MergeBy<I0, I1, fn(&(T, T), &(T, T)) -> bool>;
pub type KMergeByRanges<T, I> = KMergeBy<I, fn(&(T, T), &(T, T)) -> bool>;
pub type BitAndIterMerge<T, I0, I1> = NotIter<T, BitOrOfMergeNots<T, I0, I1>>;
pub type BitAndIterKMerge<T, I> = NotIter<T, BitOrOfKMergeNots<T, I>>;
pub type BitSubIter<T, I0, I1> = BitAndIterMerge<T, I0, NotIter<T, I1>>;
pub type BitOrOfMergeNots<T, I0, I1> = BitOrIterOfMergeBy<T, NotIter<T, I0>, NotIter<T, I1>>;
pub type BitOrOfKMergeNots<T, I> = BitOrIterOfKMergeBy<T, NotIter<T, I>>;

impl<T: Integer, I: Clone + Iterator<Item = (T, T)>> SortedByKey for BitOrIter<T, I> {}
impl<T: Integer, I: Clone + Iterator<Item = (T, T)>> SortedByKey for NotIter<T, I> {}

impl<T, I0, I1> BitOrIterOfMergeBy<T, I0, I1>
where
    T: Integer,
    I0: Iterator<Item = (T, T)> + std::clone::Clone,
    I1: Iterator<Item = (T, T)> + Clone,
{
    // !!!cmk0 understand this better
    fn new(lhs: I0, rhs: I1) -> BitOrIterOfMergeBy<T, I0, I1> {
        Self {
            merged_ranges: lhs.merge_by(rhs, |a, b| a.0 <= b.0),
            range: None,
        }
    }
}

pub trait ItertoolsPlus: Iterator + Clone {
    // !!!cmk00 where is two input merge?
    // !!!cmk0 is it an issue that all inputs by the be the same type?

    fn union<T, I1>(self) -> BitOrIterOfKMergeBy<T, I1>
    where
        Self: Iterator<Item = I1>,
        I1: Iterator<Item = (T, T)> + Clone + SortedByKey,
        T: Integer,
    {
        // !!!cmk00 that is hard to say '<Self as ItertoolsPlus>::kmerge_cmk'
        // let merged_ranges = <Self as ItertoolsPlus>::kmerge_cmk(self);
        BitOrIter {
            merged_ranges: self.kmerge_by(|pair0, pair1| pair0.0 < pair1.0),
            range: None,
        }
    }

    fn bitor<T, J>(self, other: J) -> BitOrIterOfMergeBy<T, Self, J>
    where
        T: Integer,
        Self: Iterator<Item = (T, T)> + Sized,
        J: Iterator<Item = Self::Item> + Clone + SortedByKey,
    {
        BitOrIter::new(self, other)
    }

    fn intersection<T, I1>(self) -> BitAndIterKMerge<T, I1>
    where
        Self: Iterator<Item = I1>,
        I1: Iterator<Item = (T, T)> + Clone + SortedByKey,
        T: Integer,
    {
        self.map(|seq| seq.not()).union().not()
    }
    fn bitand<T, J>(self, other: J) -> BitAndIterMerge<T, Self, J>
    where
        T: Integer,
        Self: Iterator<Item = (T, T)> + Sized + SortedByKey,
        J: Iterator<Item = Self::Item> + Clone + SortedByKey,
    {
        self.not().bitor(other.not()).not()
    }

    fn sub<T, J>(self, other: J) -> BitSubIter<T, Self, J>
    where
        T: Integer,
        Self: Iterator<Item = (T, T)> + Sized + SortedByKey,
        J: Iterator<Item = Self::Item> + Clone + SortedByKey,
    {
        self.bitand(other.not())
    }

    fn not<T>(self) -> NotIter<T, Self>
    where
        T: Integer,
        Self: Iterator<Item = (T, T)> + Sized + SortedByKey,
    {
        NotIter::new(self)
    }

    fn bitxor<T, J>(
        self,
        other: J,
    ) -> BitOrIterOfMergeBy<T, BitSubIter<T, Self, J>, BitSubIter<T, J, Self>>
    where
        T: Integer,
        Self: Iterator<Item = (T, T)> + Sized + SortedByKey,
        J: Iterator<Item = Self::Item> + Clone + SortedByKey,
    {
        self.clone().sub(other.clone()).bitor(other.sub(self))
    }
}

// !!!cmk00 support multiple inputs to bitor,bitand
// !!!cmk00 allow rhs to be of a different type

impl<I: Iterator + Clone> ItertoolsPlus for I {}

impl<T, I> Iterator for BitOrIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)> + Clone,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<(T, T)> {
        if let Some((start, stop)) = self.merged_ranges.next() {
            if let Some((current_start, current_stop)) = self.range {
                debug_assert!(current_start <= start); // panic if not sorted
                if current_stop < T::max_value2() && start <= current_stop + T::one() {
                    self.range = Some((current_start, max(current_stop, stop)));
                    self.next()
                } else {
                    let result = self.range;
                    self.range = Some((start, stop));
                    result
                }
            } else {
                self.range = Some((start, stop));
                self.next()
            }
        } else {
            let result = self.range;
            self.range = None;
            result
        }
    }
}

#[derive(Clone)]
pub struct NotIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)> + Clone,
{
    ranges: I,
    start_not: T,
    next_time_return_none: bool,
}

impl<T, I> NotIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)> + Clone,
{
    fn new(ranges: I) -> Self {
        NotIter {
            ranges,
            start_not: T::min_value(),
            next_time_return_none: false,
        }
    }
}

// !!!cmk0 create coverage tests
impl<T, I> Iterator for NotIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)> + Clone,
{
    type Item = (T, T);
    fn next(&mut self) -> Option<(T, T)> {
        debug_assert!(T::min_value() <= T::max_value2()); // real assert
        if self.next_time_return_none {
            return None;
        }
        let next_item = self.ranges.next();
        if let Some((start, stop)) = next_item {
            if self.start_not < start {
                // We can subtract with underflow worry because
                // we know that start > start_not and so not min_value
                let result = Some((self.start_not, start - T::one()));
                if stop < T::max_value2() {
                    self.start_not = stop + T::one();
                } else {
                    self.next_time_return_none = true;
                }
                result
            } else if stop < T::max_value2() {
                self.start_not = stop + T::one();
                self.next()
            } else {
                self.next_time_return_none = true;
                None
            }
        } else {
            self.next_time_return_none = true;
            Some((self.start_not, T::max_value2()))
        }
    }
}

// cmk00 - also merge as iterator method
// cmk can we define ! & etc on iterators?

gen_ops_ex!(
    <T>;
    types ref RangeSetInt<T>, ref RangeSetInt<T> => RangeSetInt<T>;
    // Returns the union of `self` and `rhs` as a new `RangeSetInt`.
    //
    // # Examples
    //
    // ```
    // use range_set_int::RangeSetInt;
    //
    // let a = RangeSetInt::from([1, 2, 3]);
    // let b = RangeSetInt::from([3, 4, 5]);
    //
    // let result = &a | &b;
    // assert_eq!(result, RangeSetInt::from([1, 2, 3, 4, 5]));
    // let result = a | b;
    // assert_eq!(result, RangeSetInt::from([1, 2, 3, 4, 5]));
    // ```
    for | call |a: &RangeSetInt<T>, b: &RangeSetInt<T>| {
        RangeSetInt::from_sorted_distinct_iter(a.ranges().bitor(b.ranges()))
    };
    for & call |a: &RangeSetInt<T>, b: &RangeSetInt<T>| {
        RangeSetInt::from_sorted_distinct_iter(a.ranges().bitand(b.ranges()))
    };
    for ^ call |a: &RangeSetInt<T>, b: &RangeSetInt<T>| {
        RangeSetInt::from_sorted_distinct_iter(a.ranges().bitxor(b.ranges()))
    };
    for - call |a: &RangeSetInt<T>, b: &RangeSetInt<T>| {
        RangeSetInt::from_sorted_distinct_iter(a.ranges().sub(b.ranges()))
    };

    where T: Integer //Where clause for all impl's
);

gen_ops_ex!(
    <T>;
    types ref RangeSetInt<T> => RangeSetInt<T>;
    for ! call |a: &RangeSetInt<T>| {
        RangeSetInt::from_sorted_distinct_iter(a.ranges().not())
    };

    where T: Integer //Where clause for all impl's
);

// cmk00 - also merge as iterator method
// cmk can we define ! & etc on iterators?
// cmk0 should we even provide the Assign methods, since only bitor_assign could be better than bitor?
// cmk00 use from/into to shorten xor's expression

impl<T: Integer, const N: usize> From<[T; N]> for RangeSetInt<T> {
    fn from(arr: [T; N]) -> Self {
        RangeSetInt::from(arr.as_slice())
    }
}

enum OptionRange<T: Integer> {
    None,
    Some { start: T, stop: T },
}

impl<T: Integer> IntoIterator for RangeSetInt<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Gets an iterator for moving out the `RangeSetInt`'s contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let set = RangeSetInt::from([1, 2, 3, 4]);
    ///
    /// let v: Vec<_> = set.into_iter().collect();
    /// assert_eq!(v, [1, 2, 3, 4]);
    /// ```
    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            option_range: OptionRange::None,
            range_into_iter: self.items.into_iter(),
        }
    }
}

pub struct Iter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    current: T,
    option_range: OptionRange<T>,
    range_iter: I,
}

impl<T: Integer, I> Iterator for Iter<T, I>
where
    I: Iterator<Item = (T, T)>,
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if let OptionRange::Some { start, stop } = self.option_range {
            self.current = start;
            if start < stop {
                self.option_range = OptionRange::Some {
                    start: start + T::one(),
                    stop,
                };
            } else {
                self.option_range = OptionRange::None;
            }
            Some(self.current)
        } else if let Some((start, stop)) = self.range_iter.next() {
            self.option_range = OptionRange::Some { start, stop };
            self.next()
        } else {
            None
        }
    }
}

pub struct IntoIter<T: Integer> {
    option_range: OptionRange<T>,
    range_into_iter: std::collections::btree_map::IntoIter<T, T>,
}

impl<T: Integer> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let OptionRange::Some { start, stop } = self.option_range {
            if start < stop {
                self.option_range = OptionRange::Some {
                    start: start + T::one(),
                    stop,
                };
            } else {
                self.option_range = OptionRange::None;
            }
            Some(start)
        } else if let Some((start, stop)) = self.range_into_iter.next() {
            self.option_range = OptionRange::Some { start, stop };
            self.next()
        } else {
            None
        }
    }
}

impl<T: Integer> From<&[T]> for RangeSetInt<T> {
    fn from(slice: &[T]) -> Self {
        RangeSetInt::from_iter(slice.iter().copied())
    }
}

// !!!cmk can we make a version that takes another RangeSetInt and doesn't use Merger?
impl<T: Integer> Extend<T> for RangeSetInt<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        // !!!!cmk00 !!!! likely error: this may fail is range_set_int is not empty
        Merger::from_iter(iter).collect_into(self);
    }
}

impl<'a, T: 'a + Integer> Extend<&'a T> for RangeSetInt<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

pub struct MemorylessData {
    current_is_empty: bool,
    current_lower: u64,
    current_upper: u64,
    rng: StdRng,
    len: u128,
    range_len: u64,
    average_coverage_per_clump: f64,
}

impl MemorylessData {
    pub fn new(seed: u64, range_len: u64, len: u128, coverage_goal: f64) -> Self {
        let average_coverage_per_clump = 1.0 - (1.0 - coverage_goal).powf(1.0 / (range_len as f64));
        Self {
            rng: StdRng::seed_from_u64(seed),
            current_is_empty: true,
            current_lower: 0,
            current_upper: 0,
            len,
            range_len,
            average_coverage_per_clump,
        }
    }
}

impl Iterator for MemorylessData {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.current_is_empty {
            let value = self.current_lower;
            if self.current_lower == self.current_upper {
                self.current_is_empty = true;
            } else {
                self.current_lower += 1u64;
            }
            Some(value)
        } else if self.range_len == 0 {
            None
        } else {
            self.range_len -= 1;
            let mut start_fraction = self.rng.gen::<f64>();
            let mut width_fraction = self.rng.gen::<f64>() * self.average_coverage_per_clump * 2.0;
            if start_fraction + width_fraction > 1.0 {
                if self.rng.gen::<f64>() < 0.5 {
                    width_fraction = 1.0 - (start_fraction + width_fraction);
                    start_fraction = 0.0;
                } else {
                    width_fraction = 1.0 - start_fraction;
                }
            }
            self.current_is_empty = false;
            let len_f64: f64 = self.len as f64;
            let current_lower_f64: f64 = len_f64 * start_fraction;
            self.current_lower = current_lower_f64 as u64;
            let delta = (len_f64 * width_fraction) as u64;
            self.current_upper = self.current_lower + delta;
            self.next()
        }
    }
}

// https://stackoverflow.com/questions/30540766/how-can-i-add-new-methods-to-iterator
