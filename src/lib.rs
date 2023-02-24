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
// https://stackoverflow.com/questions/30540766/how-can-i-add-new-methods-to-iterator
// !!!cmk0 how could you write your own subtraction that subtracted many sets from one set via iterators?
// !!!cmk0 sort by start then by larger stop

mod safe_subtract;
mod simple;
mod sorted_disjoint_from_iter;
mod tests;

use gen_ops::gen_ops_ex;
use itertools::Itertools;
use itertools::KMergeBy;
use itertools::MergeBy;
use itertools::Tee;
use num_traits::Zero;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use sorted_disjoint_from_iter::SortedDisjointFromIter;
use sorted_disjoint_from_iter::SortedDisjointWithLenSoFar;
use sorted_disjoint_from_iter::UnsortedDisjoint;
use std::cmp::max;
use std::collections::btree_map;
use std::collections::BTreeMap;
use std::convert::From;
use std::fmt;
use std::ops;
use std::ops::Sub;
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

// !!!cmk0 define these for SortedDisjoint, too?
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
}

impl<'a, T: Integer + 'a> RangeSetInt<T> {
    // !!!cmk0 should part of this be a method on BitOrIter?
    pub fn union<I>(input: I) -> Self
    where
        I: IntoIterator<Item = &'a RangeSetInt<T>>,
    {
        input.into_iter().map(|x| x.ranges()).union().into()
    }

    pub fn intersection<I>(input: I) -> Self
    where
        I: IntoIterator<Item = &'a RangeSetInt<T>>,
    {
        input.into_iter().map(|x| x.ranges()).intersection().into()
    }
}

impl<T: Integer> RangeSetInt<T> {
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
    ///
    /// ```
    /// cmk add a note about the performance compared
    /// to bitor
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

impl<'a, T: Integer> AsRef<Ranges<'a, T>> for Ranges<'a, T> {
    fn as_ref(&self) -> &Self {
        // Self is Ranges<'a>, the type for which we impl AsRef
        self
    }
}

impl<T: Integer> SortedDisjoint for Ranges<'_, T> {}
impl<T: Integer, I: Iterator<Item = (T, T)>> SortedDisjoint for BitOrIter<T, I> {}
impl<T: Integer, I: Iterator<Item = (T, T)>> SortedDisjoint for NotIter<T, I> {}

// cmk rules define "pass through" functions
impl<T: Integer, I: Iterator<Item = (T, T)> + SortedDisjoint> SortedDisjoint for Tee<I> {}

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

impl<T: Integer> FromIterator<T> for RangeSetInt<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        iter.into_iter().map(|x| (x, x)).collect()
    }
}

impl<T: Integer> FromIterator<(T, T)> for RangeSetInt<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, T)>,
    {
        let sorted_disjoint: SortedDisjointFromIter<_> = iter.into_iter().collect();
        RangeSetInt::from_sorted_disjoint(sorted_disjoint)
    }
}

// cmk rules: When should use Iterator and when IntoIterator?
// cmk rules: When should use: from_iter, from, new from_something?
impl<T: Integer> RangeSetInt<T> {
    fn from_sorted_disjoint<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, T)>,
        <I as IntoIterator>::IntoIter: SortedDisjoint,
    {
        let mut iter_with_len = SortedDisjointWithLenSoFar::new(iter.into_iter());
        let btree_map = BTreeMap::from_iter(&mut iter_with_len);
        let len = iter_with_len.len();
        RangeSetInt {
            items: btree_map,
            len,
        }
    }
}

#[derive(Clone)]
pub struct BitOrIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    merged_ranges: I,
    range: Option<(T, T)>,
}

// !!!cmk0 should I0,I1 be I,J to match itertools?
pub type Merge<T, I0, I1> = MergeBy<I0, I1, fn(&(T, T), &(T, T)) -> bool>;
pub type KMerge<T, I> = KMergeBy<I, fn(&(T, T), &(T, T)) -> bool>;
pub type BitOrMerge<T, I0, I1> = BitOrIter<T, Merge<T, I0, I1>>;
pub type BitOrKMerge<T, I> = BitOrIter<T, KMerge<T, I>>;
pub type BitAndMerge<T, I0, I1> = NotIter<T, BitNandMerge<T, I0, I1>>;
pub type BitAndKMerge<T, I> = NotIter<T, BitNandKMerge<T, I>>;
pub type BitNandMerge<T, I0, I1> = BitOrMerge<T, NotIter<T, I0>, NotIter<T, I1>>;
pub type BitNandKMerge<T, I> = BitOrKMerge<T, NotIter<T, I>>;
pub type BitNorMerge<T, J, I> = NotIter<T, BitOrMerge<T, J, I>>;
// !!!cmk0 why is there no BitSubKMerge? and BitXorKMerge?
pub type BitSubMerge<T, I0, I1> = NotIter<T, BitOrMerge<T, NotIter<T, I0>, I1>>;
pub type BitXOrTee<T, I0, I1> =
    BitOrMerge<T, BitSubMerge<T, Tee<I0>, Tee<I1>>, BitSubMerge<T, Tee<I1>, Tee<I0>>>;
pub type BitXOr<T, I0, I1> =
    BitOrMerge<T, BitSubMerge<T, I0, Tee<I1>>, BitSubMerge<T, Tee<I1>, I0>>;
pub type BitEq<T, J, I> = BitOrMerge<
    T,
    NotIter<T, BitOrMerge<T, NotIter<T, Tee<J>>, NotIter<T, Tee<I>>>>,
    NotIter<T, BitOrMerge<T, Tee<J>, Tee<I>>>,
>;
// pub type BitXOrMerge<T, I0, I1> = BitOrMerge<
//     T,
//     BitAndMerge<T, AssumeSortedByKey<Tee<I0>>, NotIter<T, AssumeSortedByKey<Tee<I1>>>>,
//     BitAndMerge<T, AssumeSortedByKey<Tee<I1>>, NotIter<T, AssumeSortedByKey<Tee<I0>>>>,
// >;

// !!!cmk0 do we need any 'new' methods?
impl<T, I0, I1> BitOrMerge<T, I0, I1>
where
    T: Integer,
    I0: Iterator<Item = (T, T)>,
    I1: Iterator<Item = (T, T)>,
{
    // !!!cmk0 understand this better
    fn new(lhs: I0, rhs: I1) -> BitOrMerge<T, I0, I1> {
        Self {
            merged_ranges: lhs.merge_by(rhs, |a, b| a.0 <= b.0),
            range: None,
        }
    }
}

// !!!cmk0 these are too easy to mix up with other things
pub fn union<T, I0, I1>(input: I0) -> BitOrKMerge<T, I1>
where
    I0: IntoIterator<Item = I1>,
    I1: Iterator<Item = (T, T)>,
    T: Integer,
{
    BitOrIter {
        merged_ranges: input
            .into_iter()
            .kmerge_by(|pair0, pair1| pair0.0 < pair1.0),
        range: None,
    }
}

// !!!cmk0 why define standalone function if only ever called from below?
pub fn intersection<T, I0, I1>(input: I0) -> BitAndKMerge<T, I1>
where
    // !!!cmk0 understand I0: Iterator vs I0: IntoIterator
    I0: IntoIterator<Item = I1>,
    I1: Iterator<Item = (T, T)> + SortedDisjoint,
    T: Integer,
{
    input.into_iter().map(|seq| seq.not()).union().not()
}

// !!!cmk rule: Follow the rules of good API design including accepting almost any type of input
impl<I: IntoIterator + Sized> ItertoolsPlus2 for I {}
pub trait ItertoolsPlus2: IntoIterator + Sized {
    // !!!cmk0 where is two input merge?

    fn union<T, I>(self) -> BitOrKMerge<T, I>
    where
        Self: IntoIterator<Item = I>,
        I: Iterator<Item = (T, T)>,
        T: Integer,
    {
        union(self)
    }

    // !!!cmk0 don't have a function and a method. Pick one.
    fn intersection<T, I>(self) -> BitAndKMerge<T, I>
    where
        Self: IntoIterator<Item = I>,
        I: Iterator<Item = (T, T)> + SortedDisjoint,
        T: Integer,
    {
        intersection(self)
    }
}

// impl<I, T> ItertoolsSorted<T> for I
// where
//     I: SortedDisjoint1<T> + Sized,
//     T: Integer,
// {
// }

impl<T, I> From<I> for RangeSetInt<T>
where
    T: Integer,
    // !!!cmk what does IntoIterator's ' IntoIter = I::IntoIter' mean?
    I: Iterator<Item = (T, T)> + SortedDisjoint,
{
    fn from(iter: I) -> Self {
        let mut len = <T as SafeSubtract>::Output::zero();
        let sorted_disjoint_iter = iter.map(|(start, stop)| {
            len += T::safe_subtract_inclusive(stop, start);
            (start, stop)
        });
        let items = BTreeMap::<T, T>::from_iter(sorted_disjoint_iter);
        RangeSetInt::<T> { items, len }
    }
}

// define mathematical set methods, e.g. left_iter.left(right_iter) returns the left_iter.
pub trait SortedDisjointIterator<T: Integer>:
    Iterator<Item = (T, T)> + SortedDisjoint + Sized
{
    fn bitor<J: SortedDisjointIterator<T>>(self, other: J) -> BitOrMerge<T, Self, J> {
        BitOrMerge::new(self, other)
    }
    fn bitand<J>(self, other: J) -> BitAndMerge<T, Self, J>
    where
        J: Iterator<Item = Self::Item> + SortedDisjoint,
    {
        !(self.not().bitor(other.not()))
    }

    fn sub<J>(self, other: J) -> BitSubMerge<T, Self, J>
    where
        J: Iterator<Item = Self::Item> + SortedDisjoint + Sized, // !!!cmk0 why Sized?
    {
        !(self.not().bitor(other))
    }

    fn not(self) -> NotIter<T, Self> {
        NotIter::new(self)
    }

    // !!! cmk0 how do do this without cloning?
    // !!! cmk00 test the speed of this
    fn bitxor<J>(self, other: J) -> BitXOrTee<T, Self, J>
    where
        J: Iterator<Item = Self::Item> + SortedDisjoint + Sized,
    {
        let (lhs0, lhs1) = self.tee();
        let (rhs0, rhs1) = other.tee();
        lhs0.sub(rhs0) | rhs1.sub(lhs1)
    }

    fn equal<I>(self, other: I) -> bool
    where
        I: Iterator<Item = Self::Item> + SortedDisjoint,
    {
        itertools::equal(self, other)
    }
}

// !!!cmk0 allow rhs to be of a different type
impl<T, I> SortedDisjointIterator<T> for I
where
    T: Integer,
    I: Iterator<Item = (T, T)> + SortedDisjoint,
{
}

impl<T, I> Iterator for BitOrIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<(T, T)> {
        if let Some((start, stop)) = self.merged_ranges.next() {
            if let Some((current_start, current_stop)) = self.range {
                debug_assert!(current_start <= start); // panic if not sorted
                if start <= current_stop  // !!!cmk0 this code also appears in SortedRanges
                    || (current_stop < T::max_value2() && start <= current_stop + T::one())
                {
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
    I: Iterator<Item = (T, T)>,
{
    iter: I,
    start_not: T,
    next_time_return_none: bool,
}

// impl<I: Iterator + Clone> Clone for NotIter<T, I>
// where
//     T: Integer,
//     I::Item: Clone,
// {
//     fn clone(&self) -> Self {
//         Self {
//             a: self.a.clone(),
//             b: self.b.clone(),
//         }
//     }
// }

// cmk0 do we even need this?
impl<T, I> NotIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    fn new(iter: I) -> Self {
        NotIter {
            iter,
            start_not: T::min_value(),
            next_time_return_none: false,
        }
    }
}

// !!!cmk0 create coverage tests
impl<T, I> Iterator for NotIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    type Item = (T, T);
    fn next(&mut self) -> Option<(T, T)> {
        debug_assert!(T::min_value() <= T::max_value2()); // real assert
        if self.next_time_return_none {
            return None;
        }
        let next_item = self.iter.next();
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

// cmk0 - also merge as iterator method
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
        (a.ranges()|b.ranges()).into()
    };
    for & call |a: &RangeSetInt<T>, b: &RangeSetInt<T>| {
        (a.ranges() & b.ranges()).into()
    };
    for ^ call |a: &RangeSetInt<T>, b: &RangeSetInt<T>| {
        (a.ranges() ^ b.ranges()).into()
    };
    for - call |a: &RangeSetInt<T>, b: &RangeSetInt<T>| {
        (a.ranges() - b.ranges()).into()
    };
    // cmk0 must/should we support both operators and methods?

    where T: Integer //Where clause for all impl's
);

gen_ops_ex!(
    <T>;
    types ref RangeSetInt<T> => RangeSetInt<T>;
    for ! call |a: &RangeSetInt<T>| {
        (!a.ranges()).into()
    };

    where T: Integer //Where clause for all impl's
);

#[derive(Debug, Clone)]
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

#[derive(Clone)]
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

/// cmk warn that adds one-by-one
/// cmk should the input be named 'iter' or 'into_iter'?
impl<T: Integer> Extend<T> for RangeSetInt<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        for (start, stop) in UnsortedDisjoint::from(iter.map(|x| (x, x))) {
            self.internal_add(start, stop);
        }
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
        // !!!cmk0 this may not handle empty strings correctly
        s.split(',')
            .map(|s| {
                let mut range = s.split("..=");
                let start = range.next().unwrap().parse::<T>().unwrap();
                let stop = range.next().unwrap().parse::<T>().unwrap();
                (start, stop)
            })
            .collect()
    }
}
// cmk0 - also merge as iterator method
// cmk can we define ! & etc on iterators?
// cmk0 should we even provide the Assign methods, since only bitor_assign could be better than bitor?

impl<T: Integer, const N: usize> From<[T; N]> for RangeSetInt<T> {
    fn from(arr: [T; N]) -> Self {
        RangeSetInt::from(arr.as_slice())
    }
}

impl<T: Integer> From<&[T]> for RangeSetInt<T> {
    fn from(slice: &[T]) -> Self {
        slice.iter().cloned().collect()
    }
}

// impl<T> From<RangeSetInt<T>> for Ranges<'_, T>
// where
//     T: Integer,
// {
//     fn from(range_set_int: RangeSetInt<T>) -> Ranges<'_, T> {
//         range_set_int.ranges()
//     }
// }

// impl<T, I> From<I> for RangeSetInt<T>
// where
//     T: Integer,
//     I: SortedDisjoint<T> + Clone + Sized,
// {
//     fn from(iter: I) -> RangeSetInt<T> {
//         RangeSetInt::from_sorted_disjoint_iter(iter)
//     }
// }
pub trait SortedDisjoint {}
// cmk This code from sorted-iter shows how to define clone when possible
// impl<I: Iterator + Clone, J: Iterator + Clone> Clone for Union<I, J>
// where
//     I::Item: Clone,
//     J::Item: Clone,
// {
//     fn clone(&self) -> Self {
//         Self {
//             a: self.a.clone(),
//             b: self.b.clone(),
//         }
//     }
// }

// cmk sort-iter uses peekable. Is that better?

pub struct DynSortedDisjoint<'a, T> {
    iter: Box<dyn Iterator<Item = T> + 'a>,
}
impl<'a, T> SortedDisjoint for DynSortedDisjoint<'a, T> {}

impl<'a, T> DynSortedDisjoint<'a, T> {
    pub fn new(iter: impl Iterator<Item = T> + SortedDisjoint + 'a) -> Self {
        DynSortedDisjoint {
            iter: Box::new(iter),
        }
    }
}

impl<'a, T> Iterator for DynSortedDisjoint<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// extension trait for any iterator to add a assume_sorted_by_item method
pub trait DynSortedDisjointExt<'a>: Iterator + Sized + SortedDisjoint + 'a {
    /// create dynamic version of the iterator
    fn dyn_sorted_disjoint(self) -> DynSortedDisjoint<'a, Self::Item> {
        DynSortedDisjoint::new(self)
    }
}

impl<'a, I: Iterator + Sized + SortedDisjoint + 'a> DynSortedDisjointExt<'a> for I {}

#[macro_export]
macro_rules! intersection_dyn {
    ($($val:expr),*) => {{
        let arr = [$($val.dyn_sorted_disjoint()),*];
        arr.intersection()
    }}
}

#[macro_export]
macro_rules! union_dyn {
    ($($val:expr),*) => {{
        let arr = [$($val.dyn_sorted_disjoint()),*];
        arr.union()
    }}
}

// Not: Ranges, NotIter, BitOrMerge
impl<T: Integer> ops::Not for Ranges<'_, T> {
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        NotIter::new(self)
    }
}

impl<T: Integer, I> ops::Not for NotIter<T, I>
where
    I: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = I;

    fn not(self) -> Self::Output {
        self.iter
    }
}

impl<T: Integer, I0, I1> ops::Not for BitOrMerge<T, I0, I1>
where
    I0: Iterator<Item = (T, T)> + SortedDisjoint,
    I1: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        NotIter::new(self)
    }
}

// BitOr: Ranges, NotIter, BitOrMerge
impl<T: Integer, I> ops::BitOr<I> for Ranges<'_, T>
where
    I: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = BitOrMerge<T, Self, I>;

    fn bitor(self, rhs: I) -> Self::Output {
        BitOrIter::new(self, rhs)
    }
}

impl<T: Integer, I, J> ops::BitOr<I> for NotIter<T, J>
where
    I: Iterator<Item = (T, T)> + SortedDisjoint,
    J: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = BitOrMerge<T, Self, I>;

    fn bitor(self, rhs: I) -> Self::Output {
        BitOrIter::new(self, rhs)
    }
}

impl<T: Integer, I0, I1, I2> ops::BitOr<I2> for BitOrMerge<T, I0, I1>
where
    I0: Iterator<Item = (T, T)> + SortedDisjoint,
    I1: Iterator<Item = (T, T)> + SortedDisjoint,
    I2: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = BitOrMerge<T, Self, I2>;

    fn bitor(self, rhs: I2) -> Self::Output {
        // cmk00 should we optimize a|b|c into union(a,b,c)?
        BitOrIter::new(self, rhs)
    }
}

// Sub: Ranges, NotIter, BitOrMerge

impl<T: Integer, I> ops::Sub<I> for Ranges<'_, T>
where
    I: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = BitSubMerge<T, Self, I>;

    fn sub(self, rhs: I) -> Self::Output {
        !(!self | rhs)
    }
}

impl<T: Integer, I, J> ops::Sub<I> for NotIter<T, J>
where
    I: Iterator<Item = (T, T)> + SortedDisjoint,
    J: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = BitNorMerge<T, J, I>;

    fn sub(self, rhs: I) -> Self::Output {
        // optimize !!self.iter into self.iter
        !self.iter.bitor(rhs)
    }
}

impl<T: Integer, I0, I1, I2> ops::Sub<I2> for BitOrMerge<T, I0, I1>
where
    I0: Iterator<Item = (T, T)> + SortedDisjoint,
    I1: Iterator<Item = (T, T)> + SortedDisjoint,
    I2: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = BitSubMerge<T, Self, I2>;

    fn sub(self, rhs: I2) -> Self::Output {
        !(!self | rhs)
    }
}

// BitXor: Ranges, NotIter, BitOrMerge

impl<T: Integer, I> ops::BitXor<I> for Ranges<'_, T>
where
    I: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = BitXOr<T, Self, I>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitxor(self, rhs: I) -> Self::Output {
        // optimize by using self.clone() instead of tee
        let lhs1 = self.clone();
        let (rhs0, rhs1) = rhs.tee();
        (self - rhs0) | (rhs1.sub(lhs1))
    }
}

impl<T: Integer, I, J> ops::BitXor<I> for NotIter<T, J>
where
    I: Iterator<Item = (T, T)> + SortedDisjoint,
    J: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = BitEq<T, J, I>;

    fn bitxor(self, rhs: I) -> Self::Output {
        let (not_lhs0, not_lhs1) = self.iter.tee();
        let (rhs0, rhs1) = rhs.tee();
        // optimize !!self.iter into self.iter
        // ¬(¬n ∨ ¬r) ∨ ¬(n ∨ r) // https://www.wolframalpha.com/input?i=%28not+n%29+xor+r
        !(not_lhs0.not() | rhs0.not()) | !not_lhs1.bitor(rhs1)
    }
}

impl<T: Integer, I0, I1, I2> ops::BitXor<I2> for BitOrMerge<T, I0, I1>
where
    I0: Iterator<Item = (T, T)> + SortedDisjoint,
    I1: Iterator<Item = (T, T)> + SortedDisjoint,
    I2: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = BitXOrTee<T, Self, I2>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitxor(self, rhs: I2) -> Self::Output {
        let (lhs0, lhs1) = self.tee();
        let (rhs0, rhs1) = rhs.tee();
        lhs0.sub(rhs0) | rhs1.sub(lhs1)
    }
}

// BitAnd: Ranges, NotIter, BitOrMerge

impl<T: Integer, I> ops::BitAnd<I> for Ranges<'_, T>
where
    I: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = BitAndMerge<T, Self, I>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitand(self, rhs: I) -> Self::Output {
        !(!self | rhs.not())
    }
}

impl<T: Integer, I, J> ops::BitAnd<I> for NotIter<T, J>
where
    I: Iterator<Item = (T, T)> + SortedDisjoint,
    J: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = NotIter<T, BitOrMerge<T, J, NotIter<T, I>>>;

    fn bitand(self, rhs: I) -> Self::Output {
        // optimize !!self.iter into self.iter
        !self.iter.bitor(rhs.not())
    }
}

// cmk name all generics in a sensible way
impl<T: Integer, I0, I1, I2> ops::BitAnd<I2> for BitOrMerge<T, I0, I1>
where
    I0: Iterator<Item = (T, T)> + SortedDisjoint,
    I1: Iterator<Item = (T, T)> + SortedDisjoint,
    I2: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Output = BitAndMerge<T, Self, I2>;

    fn bitand(self, rhs: I2) -> Self::Output {
        !(!self | rhs.not())
    }
}
