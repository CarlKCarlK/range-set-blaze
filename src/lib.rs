// https://docs.rs/range_bounds_map/latest/range_bounds_map/range_bounds_set/struct.RangeBoundsSet.html
// Here are some relevant crates I found whilst searching around the topic area:

// https://docs.rs/rangemap Very similar to this crate but can only use Ranges and RangeInclusives as keys in it's map and set structs (separately).
// https://docs.rs/btree-range-map
// https://docs.rs/ranges Cool library for fully-generic ranges (unlike std::ops ranges), along with a Ranges datastructure for storing them (Vec-based unfortunately)
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
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output;
}

impl SafeSubtract for i8 {
    type Output = u8;
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output
    }
}

impl SafeSubtract for u8 {
    type Output = u8;
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a - b
    }
}

impl SafeSubtract for i16 {
    type Output = u16;
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output
    }
}

impl SafeSubtract for u16 {
    type Output = u16;
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a - b
    }
}

impl SafeSubtract for i32 {
    type Output = u32;
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output
    }
}

impl SafeSubtract for u32 {
    type Output = u32;
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a - b
    }
}

impl SafeSubtract for i64 {
    type Output = u64;
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output
    }
}

impl SafeSubtract for u64 {
    type Output = u64;
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a - b
    }
}

impl SafeSubtract for isize {
    type Output = usize;
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output
    }
}

impl SafeSubtract for usize {
    type Output = usize;
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a - b
    }
}

impl SafeSubtract for i128 {
    type Output = u128;
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output
    }
}

impl SafeSubtract for u128 {
    type Output = u128;
    fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a - b
    }
}

pub fn test_me<T>(i: T)
where
    T: Integer,
{
    let _j = i;
}

pub fn test_me_i32(i: i32) {
    test_me(i);
}

pub fn fmt<T: Integer>(items: &BTreeMap<T, T>) -> String {
    items
        .iter()
        .map(|(start, end)| format!("{start}..{end}"))
        .join(",")
}

/// !!! cmk understand this
fn len_slow<T: Integer>(items: &BTreeMap<T, T>) -> <T as SafeSubtract>::Output
where
    for<'a> &'a T: Sub<&'a T, Output = T>,
{
    items.iter().fold(
        <T as SafeSubtract>::Output::default(),
        |acc, (start, end)| acc + T::safe_subtract(*end, *start),
    )
}

pub fn internal_add<T: Integer>(
    items: &mut BTreeMap<T, T>,
    len: &mut <T as SafeSubtract>::Output,
    start: T,
    end: T,
) {
    if start == end {
        return;
    }
    assert!(start < end); // !!!cmk check that length is not zero
                          // !!! cmk would be nice to have a partition_point function that returns two iterators
    let mut before = items.range_mut(..=start).rev();
    if let Some((start_before, end_before)) = before.next() {
        if *end_before < start {
            insert(items, len, start, end);
            *len += T::safe_subtract(end, start);
        } else if *end_before < end {
            *len += T::safe_subtract(end, *end_before);
            *end_before = end;
            let start_before = *start_before;
            delete_extra(items, len, start_before, end);
        } else {
            // completely contained, so do nothing
        }
    } else {
        insert(items, len, start, end);
        // !!!cmk 0
        *len += T::safe_subtract(end, start);
    }
}

fn delete_extra<T: Integer>(
    items: &mut BTreeMap<T, T>,
    len: &mut <T as SafeSubtract>::Output,
    start: T,
    end: T,
) {
    let mut after = items.range_mut(start..);
    let (start_after, start_end) = after.next().unwrap(); // !!! cmk assert that there is a next
    assert!(start == *start_after && end == *start_end); // !!! cmk real assert
                                                         // !!!cmk would be nice to have a delete_range function
    let mut end_new = end;
    let delete_list = after
        .map_while(|(start_delete, end_delete)| {
            if *start_delete <= end {
                end_new = max(end_new, *end_delete);
                *len -= T::safe_subtract(*end_delete, *start_delete);
                Some(*start_delete)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if end_new > end {
        *len += T::safe_subtract(end_new, end);
        *start_end = end_new;
    }
    for start in delete_list {
        items.remove(&start);
    }
}
fn insert<T: Integer>(
    items: &mut BTreeMap<T, T>,
    len: &mut <T as SafeSubtract>::Output,
    start: T,
    end: T,
) {
    let was_there = items.insert(start, end);
    assert!(was_there.is_none());
    // !!!cmk real assert
    delete_extra(items, len, start, end);
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
            let mut range = range.split("..");
            let start = range.next().unwrap().parse::<T>().unwrap();
            let end = range.next().unwrap().parse::<T>().unwrap();
            result.internal_add(start, end);
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

    fn len_slow(&self) -> <T as SafeSubtract>::Output
    where
        for<'a> &'a T: Sub<&'a T, Output = T>,
    {
        len_slow(&self.items)
    }

    /// Moves all elements from `other` into `self`, leaving `other` empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use rangeset_int::RangeSetInt;
    ///
    /// let mut a = RangeSetInt::from("1..4");
    /// let mut b = RangeSetInt::from("3..6");
    ///
    /// a.append(&mut b);
    ///
    /// assert_eq!(a.len(), 5u32);
    /// assert_eq!(b.len(), 0u32);
    ///
    /// assert!(a.contains(1));
    /// assert!(a.contains(2));
    /// assert!(a.contains(3));
    /// assert!(a.contains(4));
    /// assert!(a.contains(5));
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        for (start, end) in other.items.iter() {
            self.internal_add(*start, *end);
        }
        other.clear();
    }

    /// Returns `true` if the set contains an element equal to the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use rangeset_int::RangeSetInt;
    ///
    /// let set = RangeSetInt::from([1, 2, 3]);
    /// assert_eq!(set.contains(1), true);
    /// assert_eq!(set.contains(4), false);
    /// ```
    pub fn contains(&self, value: T) -> bool {
        self.items
            .range(..=value)
            .next_back()
            .map_or(false, |(_, end)| value < *end)
    }

    // https://stackoverflow.com/questions/49599833/how-to-find-next-smaller-key-in-btreemap-btreeset
    // https://stackoverflow.com/questions/35663342/how-to-modify-partially-remove-a-range-from-a-btreemap
    fn internal_add(&mut self, start: T, end: T) {
        internal_add(&mut self.items, &mut self.len, start, end);
    }
}

impl<T: Integer> BitOr<&RangeSetInt<T>> for &RangeSetInt<T> {
    type Output = RangeSetInt<T>;

    /// Returns the union of `self` and `rhs` as a new `RangeSetInt`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rangeset_int::RangeSetInt;
    ///
    /// let a = RangeSetInt::from([1, 2, 3]);
    /// let b = RangeSetInt::from([3, 4, 5]);
    ///
    /// let result = &a | &b;
    /// assert_eq!(result, RangeSetInt::from([1, 2, 3, 4, 5]));
    /// ```
    fn bitor(self, rhs: &RangeSetInt<T>) -> RangeSetInt<T> {
        let mut result = self.clone();
        for (start, end) in rhs.items.iter() {
            result.internal_add(*start, *end);
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
    /// use rangeset_int::RangeSetInt;
    ///
    /// let a = RangeSetInt::<i8>::from([1, 2, 3]);
    ///
    /// let result = ! &a;
    /// assert_eq!(result.to_string(), "-128..1,4..127");
    /// ```
    fn not(self) -> RangeSetInt<T> {
        let mut result = RangeSetInt::new();
        let mut start_not = T::min_value();
        for (start, end) in self.items.iter() {
            result.internal_add(start_not, *start);
            start_not = *end;
        }
        result.internal_add(start_not, T::max_value());
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
    /// use rangeset_int::RangeSetInt;
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
    /// use rangeset_int::RangeSetInt;
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
    /// use rangeset_int::RangeSetInt;
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
        let mut result = RangeSetInt::new();
        for value in arr.iter() {
            result.internal_add(*value, *value + T::one());
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
    /// use rangeset_int::RangeSetInt;
    ///
    /// let set = RangeSetInt::from([1, 2, 3, 4]);
    ///
    /// let v: Vec<_> = set.into_iter().collect();
    /// assert_eq!(v, [1, 2, 3, 4]);
    /// ```
    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            start: T::zero(),
            end: T::zero(),
            range_iter: self.items.into_iter(),
        }
    }
}

pub struct IntoIter<T: Integer> {
    start: T,
    end: T,
    range_iter: std::collections::btree_map::IntoIter<T, T>,
}

impl<T: Integer> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let result = self.start;
            self.start += T::one();
            return Some(result);
        }
        if let Some((start, end)) = self.range_iter.next() {
            self.start = start;
            self.end = end;
            return self.next();
        }
        None
    }
}
