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
mod tests;

use itertools::Itertools;
use merger::Merger;
use merger::SortedRanges;
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
    pub fn new() -> RangeSetInt<T> {
        RangeSetInt {
            items: BTreeMap::new(),
            len: <T as SafeSubtract>::Output::zero(),
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            current: T::zero(),
            option_range: OptionRange::None,
            range_iter: self.ranges(),
        }
    }

    pub fn ranges(&self) -> std::collections::btree_map::Iter<'_, T, T> {
        self.items.iter()
    }

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
            self.internal_add(*start, *stop);
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
            self.internal_add(*start, *stop);
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
        let iter = vec![self.ranges(), rhs.ranges()]
            .into_iter()
            .kmerge_by(|a, b| a.0 <= b.0)
            .map(|(start, stop)| (*start, *stop));
        let mut result = RangeSetInt::<T>::new();
        SortedRanges::process(&mut result, iter);
        result
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
        let mut result = RangeSetInt::new();
        let mut start_not = Some(T::min_value());
        for (start, stop) in self.ranges() {
            // This is always safe because we know that start_not is not None
            let start_not2 = start_not.unwrap();
            if start > &start_not2 {
                // We can subtract with underflow worry because
                // we know that start > start_not and so not min_value
                let stop_not2 = *start - T::one();
                result.items.insert(start_not2, stop_not2);
                result.len += T::safe_subtract_inclusive(stop_not2, start_not2);
            }
            if *stop == T::max_value2() {
                start_not = None;
            } else {
                start_not = Some(*stop + T::one());
            }
        }
        if let Some(start_not) = start_not {
            let stop_not2 = T::max_value2();
            result.items.insert(start_not, stop_not2);
            result.len += T::safe_subtract_inclusive(stop_not2, start_not);
        }
        result
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
        // !!! cmk0 this does 3 copies of the data, can we do better?
        // !!! cmk0 also should we sometimes swap the order of the operands?
        let mut a = !self;
        a |= &(!rhs);
        !&a
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
        let a = self.clone();
        let mut a = &a - rhs;
        a |= &(rhs - self);
        *self = a;
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
    fn bitxor(self, rhs: &RangeSetInt<T>) -> RangeSetInt<T> {
        let mut a = self - rhs;
        a |= &(rhs - self);
        a
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
        self & &(!rhs)
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

pub struct Iter<'a, T: Integer> {
    current: T,
    option_range: OptionRange<T>,
    range_iter: std::collections::btree_map::Iter<'a, T, T>,
}

impl<'a, T: Integer> Iterator for Iter<'a, T> {
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
            self.option_range = OptionRange::Some {
                start: *start,
                stop: *stop,
            };
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
