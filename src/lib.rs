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
use std::cmp::max;
// use std::collections::btree_map::Range;
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

/// !!! cmk understand this
fn _len_slow<T: Integer>(items: &BTreeMap<T, T>) -> <T as SafeSubtract>::Output
where
    for<'a> &'a T: Sub<&'a T, Output = T>,
{
    items.iter().fold(
        <T as SafeSubtract>::Output::default(),
        |acc, (start, stop)| acc + T::safe_subtract_inclusive(*stop, *start),
    )
}

pub fn internal_add<T: Integer>(
    items: &mut BTreeMap<T, T>,
    len: &mut <T as SafeSubtract>::Output,
    start: T,
    stop: T,
) {
    assert!(start <= stop && stop <= T::max_value2()); // !!!cmk check that length is not zero
                                                       // !!! cmk would be nice to have a partition_point function that returns two iterators
    let mut before = items.range_mut(..=start).rev();
    if let Some((start_before, stop_before)) = before.next() {
        // Must check this in two parts to avoid overflow
        if *stop_before < start && *stop_before + T::one() < start {
            insert(items, len, start, stop);
            *len += T::safe_subtract_inclusive(stop, start);
        } else if *stop_before < stop {
            *len += T::safe_subtract(stop, *stop_before);
            *stop_before = stop;
            let start_before = *start_before;
            delete_extra(items, len, start_before, stop);
        } else {
            // completely contained, so do nothing
        }
    } else {
        insert(items, len, start, stop);
        // !!!cmk 0
        *len += T::safe_subtract_inclusive(stop, start);
    }
}

fn delete_extra<T: Integer>(
    items: &mut BTreeMap<T, T>,
    len: &mut <T as SafeSubtract>::Output,
    start: T,
    stop: T,
) {
    let mut after = items.range_mut(start..);
    let (start_after, stop_after) = after.next().unwrap(); // !!! cmk assert that there is a next
    debug_assert!(start == *start_after && stop == *stop_after); // !!! cmk real assert
                                                                 // !!!cmk would be nice to have a delete_range function
    let mut stop_new = stop;
    let delete_list = after
        .map_while(|(start_delete, stop_delete)| {
            // must check this in two parts to avoid overflow
            if *start_delete <= stop || *start_delete <= stop + T::one() {
                stop_new = max(stop_new, *stop_delete);
                *len -= T::safe_subtract_inclusive(*stop_delete, *start_delete);
                Some(*start_delete)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if stop_new > stop {
        *len += T::safe_subtract(stop_new, stop);
        *stop_after = stop_new;
    }
    for start in delete_list {
        items.remove(&start);
    }
}
fn insert<T: Integer>(
    items: &mut BTreeMap<T, T>,
    len: &mut <T as SafeSubtract>::Output,
    start: T,
    stop: T,
) {
    let was_there = items.insert(start, stop);
    assert!(was_there.is_none());
    // !!!cmk real assert
    delete_extra(items, len, start, stop);
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

impl<T: Integer> From<&[T]> for RangeSetInt<T> {
    fn from(slice: &[T]) -> Self {
        let mut range_set_int = RangeSetInt::<T>::new();
        let mut x32 = X32::new(&mut range_set_int);
        for i in slice {
            x32.insert(*i);
        }
        x32.save();
        range_set_int
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

    fn _len_slow(&self) -> <T as SafeSubtract>::Output
    where
        for<'a> &'a T: Sub<&'a T, Output = T>,
    {
        _len_slow(&self.items)
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
        internal_add(&mut self.items, &mut self.len, start, stop);
    }

    pub fn range_len(&self) -> usize {
        self.items.len()
    }
}

impl<T: Integer> FromIterator<T> for RangeSetInt<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut range_set_int = RangeSetInt::<T>::new();
        let mut x32 = X32::new(&mut range_set_int);
        for value in iter.into_iter() {
            x32.insert(value);
        }
        x32.save();
        range_set_int
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
        let mut result = RangeSetInt::new();
        for value in arr.iter() {
            result.internal_add(*value, *value);
        }
        result
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
pub struct X32<'a, T: Integer> {
    pub range_set_int: &'a mut RangeSetInt<T>,
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
