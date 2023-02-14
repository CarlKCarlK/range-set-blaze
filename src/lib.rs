// https://docs.rs/range_bounds_map/latest/range_bounds_map/range_bounds_set/struct.RangeBoundsSet.html
// Here are some relevant crates I found whilst searching around the topic area:

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

mod merger;

use itertools::Itertools;
use itertools::MergeBy;
use merger::Merger;
use num_traits::Zero;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::cmp::max;
use std::collections::BTreeMap;
use std::convert::From;
use std::fmt;
use std::ops::BitAndAssign;
use std::ops::BitOrAssign;
use std::ops::BitXorAssign;
use std::ops::SubAssign;
use std::ops::{BitAnd, BitOr, BitXor, Not, Sub};
use std::str::FromStr;
use trait_set::trait_set;

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
impl<T: Integer> RangeSetInt<T> {
    pub fn new() -> RangeSetInt<T> {
        RangeSetInt {
            items: BTreeMap::new(),
            len: <T as SafeSubtract>::Output::zero(),
        }
    }

    // !!!cmk0 add .map(|(start, stop)| (*start, *stop)) to ranges()?
    pub fn ranges(&self) -> impl Iterator<Item = (T, T)> + ExactSizeIterator<Item = (T, T)> + '_ {
        IdentityIter::new(TupleToValuesIter {
            inner_iter: self.items.iter(),
        })
        // !!!cmk0 can we do this without IdentityIter?
        // !!!cmk0 can we do this without TupleToValuesIter?
    }

    // pub fn ranges_not(&self) -> impl Iterator<Item = (T, T)> + '_ {
    //     NotIter::new(TupleToValuesIter {
    //         inner_iter: self.items.iter(),
    //     })
    // }

    pub fn clear(&mut self) {
        self.items.clear();
        self.len = <T as SafeSubtract>::Output::zero();
    }

    pub fn len(&self) -> <T as SafeSubtract>::Output {
        self.len.clone()
    }

    /// !!! cmk understand the 'where for'
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

    pub fn insert(&mut self, item: T) {
        self.internal_add(item, item);
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

    pub fn ranges_len(&self) -> usize {
        self.items.len()
    }
}

impl<T: Integer> FromIterator<T> for RangeSetInt<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Merger::from_iter(iter).into()
    }
}

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

pub struct BitOrIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    merged_ranges: I,
    range: Option<(T, T)>,
}

// impl<T, I> BitOrIter<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = (T, T)>,
// {
//     fn new_cmk2(merged_ranges: I) -> Self {
//         Self {
//             merged_ranges,
//             range: None,
//         }
//     }
// }

// fn sorter<T: Integer>(a: &(T, T), b: &(T, T)) -> bool {
//     a.0 <= b.0
// }

// fn new_cmk1<T, I0, I1>(
//     lhs: I0,
//     rhs: I1,
// ) -> BitOrIter<T, MergeBy<I0, I1, fn(&(T, T), &(T, T)) -> bool>>
// where
//     T: Integer + Sized,
//     I0: Iterator<Item = (T, T)>,
//     I1: Iterator<Item = (T, T)>,
// {
//     BitOrIter::new_cmk2(lhs.merge_by(rhs, |a, b| a.0 <= b.0))
// }

// fn sorter<T: Integer>(a: &(T, T), b: &(T, T)) -> bool {
//     a.0 <= b.0
// }

// !!!cmk 0 should I0,I1 be I,J to match itertools?
pub type BitOrIterOutput<T, I0, I1> = BitOrIter<T, MergeBy<I0, I1, fn(&(T, T), &(T, T)) -> bool>>;
pub type BitAndIterOutput<T, I0, I1> =
    NotIter<T, BitOrIterOutput<T, NotIter<T, I0>, NotIter<T, I1>>>;
pub type BitSubIterOutput<T, I0, I1> = BitAndIterOutput<T, I0, NotIter<T, I1>>;

impl<T, I0, I1> BitOrIterOutput<T, I0, I1>
where
    T: Integer,
    I0: Iterator<Item = (T, T)>,
    I1: Iterator<Item = (T, T)>,
{
    fn new(lhs: I0, rhs: I1) -> BitOrIterOutput<T, I0, I1> {
        Self {
            merged_ranges: lhs.merge_by(rhs, |a, b| a.0 <= b.0),
            range: None,
        }
    }
}

pub trait ItertoolsPlus: Iterator {
    fn bitor<T, J>(self, other: J) -> BitOrIterOutput<T, Self, J>
    where
        T: Integer,
        Self: Iterator<Item = (T, T)> + Sized,
        J: Iterator<Item = Self::Item>,
    {
        BitOrIter::new(self, other)
    }

    fn bitand<T, J>(self, other: J) -> BitAndIterOutput<T, Self, J>
    where
        T: Integer,
        Self: Iterator<Item = (T, T)> + Sized,
        J: Iterator<Item = Self::Item>,
    {
        self.not().bitor(other.not()).not()
    }

    fn sub<T, J>(self, other: J) -> BitSubIterOutput<T, Self, J>
    where
        T: Integer,
        Self: Iterator<Item = (T, T)> + Sized,
        J: Iterator<Item = Self::Item>,
    {
        self.bitand(other.not())
    }

    fn not<T>(self) -> NotIter<T, Self>
    where
        T: Integer,
        Self: Iterator<Item = (T, T)> + Sized,
    {
        NotIter::new(self)
    }
}

// !!!cmk00 allow rhs to be of a different type
impl<T: Integer> BitOr for dyn ItertoolsPlus<Item = (T, T)>
where
    Self: Sized,
{
    type Output = BitOrIterOutput<T, Self, Self>;

    fn bitor(self, rhs: Self) -> Self::Output {
        let result = BitOrIter::new(self, rhs);
        result
    }
}

// impl<T, I0, I1> BitOr for BitOrIterMerge<T, I0, I1>
// where
//     T: Integer,
//     I0: Iterator<Item = (T, T)>,

//     Self: Sized,
// {
//     type Output = BitOrIterMerge<T, Self, Self>;

//     fn bitor(self, rhs: Self) -> Self::Output {
//         let result = BitOrIter::new(self, rhs);
//         result
//     }
// }

// impl<T> Not for ItertoolsPlus<Item=(T, T>)
// where
//     T: Integer,
//     Self: Iterator<Item = (T, T)> + Sized,
// {
//     type Output = NotIter<T, Self>;

//     fn not(self) -> RangeSetInt<T> {
//         RangeSetInt::from_sorted_distinct_iter(self.ranges().not())
//     }
// }

impl<I: Iterator> ItertoolsPlus for I {}

// impl<T, I0, I1, F> BitOrIter<T, MergeBy<I0, I1, F>>
// where
//     T: Integer,
//     I0: Iterator<Item = (T, T)>,
//     I1: Iterator<Item = (T, T)>,
//     F: FnMut(&(T, T), &(T, T)) -> bool,
// {
//     fn new2(lhs: I0, rhs: I1, is_first: F) -> Self {
//         let merged_ranges = lhs.merge_by(rhs, is_first);
//         BitOrIter::new(merged_ranges)
//     }
// }

// fn new_bitor_iter<T, I0, I1, I>(lhs: I0, rhs: I1) -> BitOrIter<T, I>
// where
//     T: Integer,
//     I0: Iterator<Item = (T, T)>,
//     I1: Iterator<Item = (T, T)>,
//     I: Iterator<Item = (T, T)>,
// {
//     BitOrIter::new2(lhs, rhs, sorter)
// }

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

impl<T: Integer> BitOr<&RangeSetInt<T>> for &RangeSetInt<T> {
    type Output = RangeSetInt<T>;

    /// Returns the union of `self` and `rhs` as a new `RangeSetInt`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let a = RangeSetInt::from([1, 2, 3]);
    /// let b = RangeSetInt::from([3, 4, 5]);
    ///
    /// let result = &a | &b;
    /// assert_eq!(result, RangeSetInt::from([1, 2, 3, 4, 5]));
    /// ```
    fn bitor(self, rhs: &RangeSetInt<T>) -> RangeSetInt<T> {
        // cmk00
        RangeSetInt::from_sorted_distinct_iter(self.ranges().bitor(rhs.ranges()))
    }
}

pub struct NotIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    ranges: I,
    start_not: T,
    next_time_return_none: bool,
}

impl<T, I> NotIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    fn new(ranges: I) -> Self {
        NotIter {
            ranges,
            start_not: T::min_value(),
            next_time_return_none: false,
        }
    }
}

// cmk0 remove range_not() because can use range().not()

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

impl<T: Integer> Not for &RangeSetInt<T> {
    type Output = RangeSetInt<T>;

    /// Returns the complement of `self` as a new `RangeSetInt`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let a = RangeSetInt::<i8>::from([1, 2, 3]);
    ///
    /// let result = ! &a;
    /// assert_eq!(result.to_string(), "-128..=0,4..=127");
    /// ```
    fn not(self) -> RangeSetInt<T> {
        RangeSetInt::from_sorted_distinct_iter(self.ranges().not())
    }
}

impl<T: Integer> BitAndAssign<&RangeSetInt<T>> for RangeSetInt<T> {
    /// Returns the intersection of `self` and `rhs` as a cmk `RangeSetInt<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut a = RangeSetInt::from([1, 2, 3]);
    /// let b = RangeSetInt::from([2, 3, 4]);
    ///
    /// a &= &b;
    /// assert_eq!(a, RangeSetInt::from([2, 3]));
    /// assert_eq!(b, RangeSetInt::from([2, 3, 4]));
    /// ```
    fn bitand_assign(&mut self, rhs: &Self) {
        // !!! cmk0 this does 3 copies of the data, can we do better?
        let mut a = !(&*self);
        a |= &(!rhs);
        *self = !&a;
    }
}

impl<T: Integer> BitAnd<&RangeSetInt<T>> for &RangeSetInt<T> {
    type Output = RangeSetInt<T>;

    /// Returns the intersection of `self` and `rhs` as a new `RangeSetInt<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let a = RangeSetInt::from([1, 2, 3]);
    /// let b = RangeSetInt::from([2, 3, 4]);
    ///
    /// let result = &a & &b;
    /// assert_eq!(result, RangeSetInt::from([2, 3]));
    /// ```
    fn bitand(self, rhs: &RangeSetInt<T>) -> RangeSetInt<T> {
        // cmk00 - also merge and not and xor, etc
        // cmk can we define ! & etc on iterators?
        // cmk00 do we still need the IdentityIter ?
        RangeSetInt::from_sorted_distinct_iter(self.ranges().bitand(rhs.ranges()))
    }
}

impl<T: Integer> BitXorAssign<&RangeSetInt<T>> for RangeSetInt<T> {
    /// Returns the symmetric difference of `self` and `rhs` as a cmk `RangeSetInt<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut a = RangeSetInt::from([1, 2, 3]);
    /// let b = RangeSetInt::from([2, 3, 4]);
    ///
    /// a ^= &b;
    /// assert_eq!(a, RangeSetInt::from([1, 4]));
    /// assert_eq!(b, RangeSetInt::from([2, 3, 4]));
    /// ```
    fn bitxor_assign(&mut self, rhs: &Self) {
        let mut rhs2 = rhs.clone();
        rhs2 -= self;
        *self -= rhs;
        *self |= &rhs2;
    }
}

impl<T: Integer> BitXor<&RangeSetInt<T>> for &RangeSetInt<T> {
    type Output = RangeSetInt<T>;

    /// Returns the symmetric difference of `self` and `rhs` as a new `RangeSetInt<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let a = RangeSetInt::from([1, 2, 3]);
    /// let b = RangeSetInt::from([2, 3, 4]);
    ///
    /// let result = &a ^ &b;
    /// assert_eq!(result, RangeSetInt::from([1, 4]));
    /// ```
    // cmk00 replace with iterator version
    fn bitxor(self, rhs: &RangeSetInt<T>) -> RangeSetInt<T> {
        RangeSetInt::from_sorted_distinct_iter(
            self.ranges()
                .sub(rhs.ranges())
                .bitor(rhs.ranges().sub(self.ranges())),
        )
    }
}

impl<T: Integer> SubAssign<&RangeSetInt<T>> for RangeSetInt<T> {
    /// Returns the set difference of `self` and `rhs` as a cmk `RangeSetInt<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut a = RangeSetInt::from([1, 2, 3]);
    /// let b = RangeSetInt::from([2, 3, 4]);
    ///
    /// a -= &b;
    /// assert_eq!(a, RangeSetInt::from([1]));
    /// assert_eq!(b, RangeSetInt::from([2, 3, 4]));
    /// ```
    // cmk00 if any of these do copies, replace with a more efficient implementation
    fn sub_assign(&mut self, rhs: &Self) {
        *self &= &(!rhs);
    }
}

impl<T: Integer> Sub<&RangeSetInt<T>> for &RangeSetInt<T> {
    type Output = RangeSetInt<T>;

    /// Returns the set difference of `self` and `rhs` as a new `RangeSetInt<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let a = RangeSetInt::from([1, 2, 3]);
    /// let b = RangeSetInt::from([2, 3, 4]);
    ///
    /// let result = &a - &b;
    /// assert_eq!(result, RangeSetInt::from([1]));
    /// ```
    fn sub(self, rhs: &RangeSetInt<T>) -> RangeSetInt<T> {
        // self & &(!rhs)
        RangeSetInt::from_sorted_distinct_iter(self.ranges().sub(rhs.ranges()))
    }
}

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

impl SafeSubtract for i8 {
    #[cfg(target_pointer_width = "16")]
    type Output = usize;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as u8 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as u8 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for u8 {
    #[cfg(target_pointer_width = "16")]
    type Output = usize;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for i32 {
    #[cfg(target_pointer_width = "16")]
    type Output = u64;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as u32 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as u32 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for u32 {
    #[cfg(target_pointer_width = "16")]
    type Output = u64;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for i64 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as u64 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as u64 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for u64 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for i128 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as u128 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as u128 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value() - 1
    }
}

impl SafeSubtract for u128 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value() - 1
    }
}

impl SafeSubtract for isize {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as usize as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as usize as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for usize {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for i16 {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as u16 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as u16 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for u16 {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
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
        // !!!!cmk0 !!!! likely error: this may fail is range_set_int is not empty
        Merger::from_iter(iter).collect_into(self);
    }

    // fn extend<I>(&mut self, other: RangeSetInt<T>) {
    //     todo!();
    //     //for (start, end) in other.range_iter() {
    //     //     self.internal_add(start, stop)
    //     // }
    //     //}
    //}
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

struct TupleToValuesIter<'a, T, J>
where
    T: Integer + 'a,
    J: Iterator<Item = (&'a T, &'a T)> + ExactSizeIterator<Item = (&'a T, &'a T)>,
{
    inner_iter: J,
}

// implement Iterator for TupleToValuesIter
impl<'a, T: Integer, J> Iterator for TupleToValuesIter<'a, T, J>
where
    J: Iterator<Item = (&'a T, &'a T)> + ExactSizeIterator<Item = (&'a T, &'a T)>,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter.next().map(|(a, b)| (*a, *b))
    }
}

impl<'a, T: Integer, J> ExactSizeIterator for TupleToValuesIter<'a, T, J>
where
    J: Iterator<Item = (&'a T, &'a T)> + ExactSizeIterator<Item = (&'a T, &'a T)>,
{
    fn len(&self) -> usize {
        self.inner_iter.len()
    }
}

struct IdentityIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)> + ExactSizeIterator<Item = (T, T)>,
{
    // !!!cmk0 name all these fields consistently (and consistent with itertools)
    ranges: I,
}

impl<T, I> IdentityIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)> + ExactSizeIterator<Item = (T, T)>,
{
    fn new(ranges: I) -> Self {
        IdentityIter { ranges }
    }
}

impl<T, I> Iterator for IdentityIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)> + ExactSizeIterator<Item = (T, T)>,
{
    type Item = (T, T);
    fn next(&mut self) -> Option<(T, T)> {
        self.ranges.next()
    }
}

impl<T, I> ExactSizeIterator for IdentityIter<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)> + ExactSizeIterator<Item = (T, T)>,
{
    fn len(&self) -> usize {
        self.ranges.len()
    }
}

// // https://stackoverflow.com/questions/30540766/how-can-i-add-new-methods-to-iterator
// pub trait ItertoolsPlus: Iterator {
//     fn merge_bigger<T, J>(self, other: J) -> MergeBy<Self, J, fn(&T, &T) -> bool>
//     //  impl Iterator<Item = Self::Item>
//     where
//         T: Integer,
//         Self: Sized,
//         Self::Item: std::cmp::PartialOrd + Sized,
//         J: Iterator<Item = Self::Item>,
//     {
//         self.merge_by(other, |a, b| b < a)
//     }
// }

// impl<I: Iterator> ItertoolsPlus for I {}

// fn merge_bigger0<I, J, T>(lhs: I, other: J) -> impl Iterator<Item = I::Item>
// where
//     T: std::cmp::PartialOrd + Sized,
//     I: Iterator<Item = T> + Sized,
//     J: Iterator<Item = I::Item>,
// {
//     lhs.merge_by(other, |a, b| b < a)
// }

// https://stackoverflow.com/questions/30540766/how-can-i-add-new-methods-to-iterator
// pub trait ItertoolsPlus: Iterator {
//     fn merge_bigger<J, T>(self, other: J) -> MergeBy<Self, J, fn(&T, &T) -> bool>
//     where
//         Self: Sized + Iterator<Item = T>,
//         T: std::cmp::PartialOrd,
//         J: Iterator<Item = Self::Item>,
//     {
//         self.merge_by(other, |a, b| *b < *a)
//     }
// }

// impl<I: Iterator> ItertoolsPlus for I {}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn test_merge_bigger0() {
//         let lhs = vec![10, 8, 7, 5, 1];
//         let rhs = vec![10, 9, 7, 2, 1];
//         let merged1 = merge_bigger0(lhs.iter(), rhs.iter());
//         let merged2: Vec<i32> = merged1.copied().collect();
//         println!("{merged2:?}");
//         assert_eq!(merged2, vec![10, 10, 9, 8, 7, 7, 5, 2, 1, 1]);

//         let merged1 = lhs.iter().merge_bigger(rhs.iter());
//         let merged2: Vec<i32> = merged1.copied().collect();
//         println!("{merged2:?}");
//         assert_eq!(merged2, vec![10, 10, 9, 8, 7, 7, 5, 2, 1, 1]);
//     }
// }
