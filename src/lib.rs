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

mod tests;

use itertools::Itertools;
// use num_traits::ops;
// use num_traits::ops::overflowing::OverflowingSub;
// use num_traits::PrimInt;
// use num_traits::ToPrimitive;
use num_traits::Zero;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use std::cmp::max;
// use std::collections::btree_map::Range;
use rayon::prelude::*; // !!! use preludes or not?
use std::collections::BTreeMap;
use std::convert::From;
use std::fmt;
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
    // !!!cmk 0
    // !!!cmk inline?
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
// !!!cmk allow negatives and any size

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
        let mut result = RangeSetInt::new();
        for range in s.split(',') {
            let mut range = range.split("..=");
            let start = range.next().unwrap().parse::<T>().unwrap();
            let stop = range.next().unwrap().parse::<T>().unwrap();
            result.internal_add(start, stop);
        }
        result
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

    pub fn clear(&mut self) {
        self.items.clear();
        self.len = <T as SafeSubtract>::Output::zero();
    }

    // !!!cmk keep this in a field
    pub fn len(&self) -> <T as SafeSubtract>::Output {
        self.len.clone()
    }

    /// !!! cmk understand the 'where for'
    fn _len_slow(&self) -> <T as SafeSubtract>::Output
    where
        for<'a> &'a T: Sub<&'a T, Output = T>,
    {
        self.items.iter().fold(
            <T as SafeSubtract>::Output::default(),
            |acc, (start, stop)| acc + T::safe_subtract_inclusive(*stop, *start),
        )
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
        for (start, stop) in other.items.iter() {
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
        assert!(start <= stop && stop <= T::max_value2()); // !!!cmk check that length is not zero
                                                           // !!! cmk would be nice to have a partition_point function that returns two iterators
        let mut before = self.items.range_mut(..=start).rev();
        if let Some((start_before, stop_before)) = before.next() {
            // Must check this in two parts to avoid overflow
            if *stop_before < start && *stop_before + T::one() < start {
                self.insert_internal(start, stop);
                self.len += T::safe_subtract_inclusive(stop, start);
            } else if *stop_before < stop {
                self.len += T::safe_subtract(stop, *stop_before);
                *stop_before = stop;
                let start_before = *start_before;
                self.delete_extra(start_before, stop);
            } else {
                // completely contained, so do nothing
            }
        } else {
            self.insert_internal(start, stop);
            // !!!cmk 0
            self.len += T::safe_subtract_inclusive(stop, start);
        }
    }

    fn insert_internal(&mut self, start: T, stop: T) {
        let was_there = self.items.insert(start, stop);
        assert!(was_there.is_none());
        // !!!cmk real assert
        self.delete_extra(start, stop);
    }

    fn delete_extra(&mut self, start: T, stop: T) {
        let mut after = self.items.range_mut(start..);
        let (start_after, stop_after) = after.next().unwrap(); // !!! cmk assert that there is a next
        debug_assert!(start == *start_after && stop == *stop_after); // !!! cmk real assert
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

    pub fn range_len(&self) -> usize {
        self.items.len()
    }

    pub fn from_mut_slice(slice: &mut [T]) -> Self {
        slice.sort_unstable();
        let mut range_set_int = RangeSetInt::<T>::new();
        let mut x32 = X32::<T> {
            range_set_int: &mut range_set_int,
            is_empty: true,
            lower: T::zero(),
            upper: T::zero(),
        };
        for item in slice {
            x32.insert(*item);
        }
        range_set_int
    }
}

impl<T: Integer> FromIterator<T> for RangeSetInt<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        // let mut range_set_int = RangeSetInt::<T>::new();
        // let mut x32 = X32::<T> {
        //     range_set_int: &mut range_set_int,
        //     is_empty: true,
        //     lower: T::zero(),
        //     upper: T::zero(),
        // };
        // for item in iter {
        //     x32.insert(item);
        // }
        // range_set_int
        let mut sortie = Sortie {
            sort_list: Vec::new(),
            is_empty: true,
            lower: T::zero(),
            upper: T::zero(),
        };
        for item in iter {
            sortie.insert(item);
        }
        let mut sort_list = sortie.sort_list;
        sort_list.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        let mut range_set_int: RangeSetInt<T> = RangeSetInt {
            items: BTreeMap::new(),
            len: <T as SafeSubtract>::Output::zero(),
        };

        let mut is_empty = true;
        let mut current_start = T::zero();
        let mut current_stop = T::zero();
        for (start, stop) in sort_list {
            if is_empty {
                current_start = start;
                current_stop = stop;
                is_empty = false;
            }
            // !!!cmk check for overflow with the +1
            else if start <= current_stop + T::one() {
                current_stop = max(current_stop, stop);
            } else {
                range_set_int.items.insert(current_start, current_stop);
                range_set_int.len += T::safe_subtract_inclusive(current_stop, current_start);
                current_start = start;
                current_stop = stop;
            }
        }
        if !is_empty {
            range_set_int.items.insert(current_start, current_stop);
            range_set_int.len += T::safe_subtract_inclusive(current_stop, current_start);
        }
        range_set_int
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
        let mut result = self.clone();
        for (start, stop) in rhs.items.iter() {
            result.internal_add(*start, *stop);
        }
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
        for (start, stop) in self.items.iter() {
            // This is always safe because we know that start_not is not None
            let start_not2 = start_not.unwrap();
            if start > &start_not2 {
                // We can subtract with underflow because we know that start > start_not
                result.internal_add(start_not2, *start - T::one());
            }
            if *stop == T::max_value2() {
                start_not = None;
            } else {
                start_not = Some(*stop + T::one());
            }
        }
        if let Some(start_not) = start_not {
            result.internal_add(start_not, T::max_value2());
        }
        result
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
        !&(&(!self) | &(!rhs))
        // !!!cmk would be nice if it didn't allocate a new RangeSetInt for each operation
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
        &(self - rhs) | &(rhs - self)
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

// !!!cmk merge this with from_iter
impl<T: Integer, const N: usize> From<[T; N]> for RangeSetInt<T> {
    fn from(arr: [T; N]) -> Self {
        RangeSetInt::from(arr.as_slice())
    }
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
            start: T::zero(),
            stop: None,
            range_iter: self.items.into_iter(),
        }
    }
}

pub struct IntoIter<T: Integer> {
    start: T,
    stop: Option<T>,
    range_iter: std::collections::btree_map::IntoIter<T, T>,
}

impl<T: Integer> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(stop) = self.stop {
            if self.start <= stop {
                let result = self.start;
                if self.start < stop {
                    self.start += T::one();
                } else {
                    self.stop = None;
                }
                return Some(result);
            }
        }
        if let Some((start, stop)) = self.range_iter.next() {
            self.start = start;
            self.stop = Some(stop);
            return self.next();
        }
        None
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

//     type Output = u128;
//     fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
//         a - b
//     }
// }
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
// pub fn test_me<T>(i: T)
// where
//     T: Integer,
// {
//     let _j = i;
// }

// pub fn test_me_i32(i: i32) {
//     test_me(i);
// }

#[derive(Debug)]
pub struct Sortie<T: Integer> {
    pub sort_list: Vec<(T, T)>,
    is_empty: bool,
    lower: T,
    upper: T,
}

#[derive(Debug)]
pub struct X32<'a, T: Integer> {
    pub range_set_int: &'a mut RangeSetInt<T>,
    is_empty: bool,
    lower: T,
    upper: T,
}

pub struct X32Own<T: Integer> {
    pub range_set_int: RangeSetInt<T>,
    is_empty: bool,
    lower: T,
    upper: T,
}

impl<'a, T: Integer> X32<'a, T> {
    pub fn new(range_set_int: &'a mut RangeSetInt<T>) -> Self {
        Self {
            range_set_int,
            is_empty: true,
            lower: T::zero(),
            upper: T::zero(),
        }
    }
    pub fn insert(&mut self, i: T) {
        if self.is_empty {
            self.lower = i;
            self.upper = i;
            self.is_empty = false;
        } else {
            if self.lower <= i && i <= self.upper {
                return;
            }
            if T::zero() < self.lower && self.lower - T::one() == i {
                self.lower = i;
                return;
            }
            // !!!cmk max_value2, right?
            if self.upper < T::max_value2() && self.upper + T::one() == i {
                self.upper = i;
                return;
            }
            self.range_set_int.internal_add(self.lower, self.upper);
            self.lower = i;
            self.upper = i;
        }
    }

    // !!! cmk what if forget to call this?
    pub fn save(&mut self) {
        if !self.is_empty {
            self.range_set_int.internal_add(self.lower, self.upper);
            self.is_empty = true;
        }
    }
}

impl<T: Integer> X32Own<T> {
    pub fn insert(&mut self, i: T) {
        if self.is_empty {
            self.lower = i;
            self.upper = i;
            self.is_empty = false;
        } else {
            if self.lower <= i && i <= self.upper {
                return;
            }
            if T::zero() < self.lower && self.lower - T::one() == i {
                self.lower = i;
                return;
            }
            // !!!cmk max_value2, right?
            if self.upper < T::max_value2() && self.upper + T::one() == i {
                self.upper = i;
                return;
            }
            self.range_set_int.internal_add(self.lower, self.upper);
            self.lower = i;
            self.upper = i;
        }
    }

    // !!! cmk what if forget to call this?
    pub fn save(&mut self) {
        if !self.is_empty {
            self.range_set_int.internal_add(self.lower, self.upper);
            self.is_empty = true;
        }
    }

    pub fn merge(mut self, mut other: Self) -> Self {
        self.save();
        other.save();
        for (lower, upper) in other.range_set_int.items.iter() {
            self.range_set_int.internal_add(*lower, *upper);
        }
        self
    }
}

impl<T: Integer> Sortie<T> {
    pub fn insert(&mut self, i: T) {
        if self.is_empty {
            self.lower = i;
            self.upper = i;
            self.is_empty = false;
        } else {
            if self.lower <= i && i <= self.upper {
                return;
            }
            if T::zero() < self.lower && self.lower - T::one() == i {
                self.lower = i;
                return;
            }
            // !!!cmk max_value2, right?
            if self.upper < T::max_value2() && self.upper + T::one() == i {
                self.upper = i;
                return;
            }
            self.sort_list.push((self.lower, self.upper));
            self.lower = i;
            self.upper = i;
        }
    }

    // !!! cmk what if forget to call this?
    pub fn save(&mut self) {
        if !self.is_empty {
            self.sort_list.push((self.lower, self.upper));
            self.is_empty = true;
        }
    }

    pub fn merge(mut self, mut other: Self) -> Self {
        self.save();
        other.save();
        self.sort_list.extend(other.sort_list);
        self
    }
}
// !!!cmk can we make a version of Extends that takes a slice and parallelizes it?
// !!!cmk or does Rayon work with iterators in a way that would work for us?
impl<T: Integer> From<&[T]> for RangeSetInt<T> {
    fn from(slice: &[T]) -> Self {
        // let num_s = [1, 2, 1, 2, 1, 2];
        // let result: HashMap<i32, i32> = num_s
        //     .par_iter()
        //     .filter(|x| *x % 2 == 0)
        //     .fold(HashMap::new, |mut acc, x| {
        //         *acc.entry(*x).or_insert(0) += 1;
        //         acc
        //     })
        //     .reduce_with(|mut m1, m2| {
        //         for (k, v) in m2 {
        //             *m1.entry(k).or_default() += v;
        //         }
        //         m1
        //     })
        //     .unwrap();
        let r = slice
            .par_iter()
            .fold(
                || {
                    let range_set_int = RangeSetInt::<T>::new();
                    X32Own {
                        range_set_int,
                        is_empty: true,
                        lower: T::zero(),
                        upper: T::zero(),
                    }
                },
                |mut acc, i| {
                    acc.insert(*i);
                    acc
                },
            )
            .reduce_with(|m1, m2| m1.merge(m2))
            .unwrap();
        r.range_set_int
    }
}
impl<T: Integer> Extend<T> for RangeSetInt<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let mut x32 = X32 {
            range_set_int: self,
            is_empty: true,
            lower: T::zero(),
            upper: T::zero(),
        };
        for value in iter.into_iter() {
            x32.insert(value);
        }
        x32.save();
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
