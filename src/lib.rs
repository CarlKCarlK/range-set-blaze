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
    + SafeSubtractInclusive
    + TryFrom<u128>
    + TryInto<u128>
    ;
}

pub trait SafeSubtractInclusive {
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
    fn safe_subtract(end: u128, start: u128) -> <Self as SafeSubtractInclusive>::Output;
    fn safe_subtract_inclusive(stop: u128, start: Self) -> <Self as SafeSubtractInclusive>::Output;
}

pub fn fmt<T: Integer>(items: &BTreeMap<T, u128>) -> String {
    items
        .iter()
        .map(|(start, stop)| format!("{start}..={stop}"))
        .join(",")
}

/// !!! cmk understand this
fn len_slow<T: Integer>(items: &BTreeMap<T, u128>) -> <T as SafeSubtractInclusive>::Output
where
    for<'a> &'a T: Sub<&'a T, Output = T>,
{
    items.iter().fold(
        <T as SafeSubtractInclusive>::Output::default(),
        |acc, (start, stop)| acc + T::safe_subtract_inclusive(*stop, *start),
    )
}

pub fn internal_add<T: Integer>(
    items: &mut BTreeMap<T, u128>,
    len: &mut <T as SafeSubtractInclusive>::Output,
    start: T,
    stop: u128,
) {
    let stop_t = if let Ok(stop_t) = stop.try_into() {
        stop_t
    } else {
        panic!("cmk");
    };
    assert!(start <= stop_t); // !!!cmk check that length is not zero
                              // !!! cmk would be nice to have a partition_point function that returns two iterators
    let mut before = items.range_mut(..=start).rev();
    if let Some((start_before, stop_before)) = before.next() {
        let stop_before_t: Result<T, _> = (*stop_before).try_into();
        let stop_before_t = if let Ok(stop_before_t) = stop_before_t {
            stop_before_t
        } else {
            panic!("cmk");
        };

        // Must check this in two parts to avoid overflow
        if stop_before_t < start && stop_before_t + T::one() < start {
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
    items: &mut BTreeMap<T, u128>,
    len: &mut <T as SafeSubtractInclusive>::Output,
    start: T,
    stop: u128,
) {
    let stop_t = if let Ok(stop_t) = stop.try_into() {
        stop_t
    } else {
        panic!("cmk");
    };

    let mut after = items.range_mut(start..);
    let (start_after, stop_after) = after.next().unwrap(); // !!! cmk assert that there is a next
    assert!(start == *start_after && stop == *stop_after); // !!! cmk real assert
                                                           // !!!cmk would be nice to have a delete_range function
    let mut stop_new = stop;
    let delete_list = after
        .map_while(|(start_delete, stop_delete)| {
            // must check this in two parts to avoid overflow
            if *start_delete <= stop_t || *start_delete <= stop_t + T::one() {
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
    items: &mut BTreeMap<T, u128>,
    len: &mut <T as SafeSubtractInclusive>::Output,
    start: T,
    stop: u128,
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
    len: <T as SafeSubtractInclusive>::Output,
    items: BTreeMap<T, u128>,
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
            let stop = range.next().unwrap().parse::<u128>().unwrap();
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
            len: <T as SafeSubtractInclusive>::Output::zero(),
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.len = <T as SafeSubtractInclusive>::Output::zero();
    }

    // !!!cmk keep this in a field
    pub fn len(&self) -> <T as SafeSubtractInclusive>::Output {
        self.len.clone()
    }

    fn len_slow(&self) -> <T as SafeSubtractInclusive>::Output
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
            .map_or(false, |(_, stop)| {
                let stop_t = (*stop).try_into().ok().unwrap(); // !!! cmk
                value <= stop_t
            })
    }

    // https://stackoverflow.com/questions/49599833/how-to-find-next-smaller-key-in-btreemap-btreeset
    // https://stackoverflow.com/questions/35663342/how-to-modify-partially-remove-a-range-from-a-btreemap
    fn internal_add(&mut self, start: T, stop: u128) {
        internal_add(&mut self.items, &mut self.len, start, stop);
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
    /// use rangeset_int::RangeSetInt;
    ///
    /// let a = RangeSetInt::<i8>::from([1, 2, 3]);
    ///
    /// let result = ! &a;
    /// assert_eq!(result.to_string(), "-128..1,4..127");
    /// ```
    fn not(self) -> RangeSetInt<T> {
        todo!(); // !!!cmk
                 // let mut result = RangeSetInt::new();
                 // let mut start_not = T::min_value();
                 // for (start, stop) in self.items.iter() {
                 //     if start > &start_not {
                 //         let start_i128: i128 = (*start).try_into().ok().unwrap(); // !!!cmk
                 //         result.internal_add(start_not, start_u128 - 1);
                 //     }
                 //     let stop_t: T = (*stop).try_into().ok().unwrap(); // !!!cmk
                 //     start_not = stop_t + T::one();
                 // }
                 // if start_not < T::max_value() {
                 //     let max_value_u128: u128 = T::max_value().try_into().ok().unwrap(); // !!!cmk
                 //     result.internal_add(start_not, max_value_u128);
                 // }
                 // result
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
            let value_u128: u128 = (*value).try_into().ok().unwrap(); // !!!cmk
            result.internal_add(*value, value_u128);
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
            stop: None,
            range_iter: self.items.into_iter(),
        }
    }
}

pub struct IntoIter<T: Integer> {
    start: T,
    stop: Option<u128>,
    range_iter: std::collections::btree_map::IntoIter<T, u128>,
}

impl<T: Integer> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(stop) = self.stop {
            let stop_t = stop.try_into().ok().unwrap(); // !!!cmk
            if self.start <= stop_t {
                let result = self.start;
                if self.start < stop_t {
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
impl SafeSubtractInclusive for i8 {
    type Output = u8;
    fn safe_subtract(end: u128, start: u128) -> <Self as SafeSubtractInclusive>::Output {
        let end = end as i8;
        let start = start as i8;
        end.overflowing_sub(start).0 as <Self as SafeSubtractInclusive>::Output
    }
    fn safe_subtract_inclusive(a: u128, b: Self) -> <Self as SafeSubtractInclusive>::Output {
        let a = a as i8;
        if a == b {
            1
        } else {
            a.overflowing_sub(b).0 as <Self as SafeSubtractInclusive>::Output + 1
        }
    }
}

impl SafeSubtractInclusive for u8 {
    type Output = u8;
    fn safe_subtract(a: u128, b: u128) -> <Self as SafeSubtractInclusive>::Output {
        let a = a as u8;
        let b = b as u8;
        a - b
    }
    fn safe_subtract_inclusive(a: u128, b: Self) -> <Self as SafeSubtractInclusive>::Output {
        let a = a as u8;
        (a - b) + 1
    }
}
// impl SafeSubtract for i16 {
//     type Output = u16;
//     fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
//         a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output
//     }
// }

// impl SafeSubtract for u16 {
//     type Output = u16;
//     fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
//         a - b
//     }
// }

// impl SafeSubtract for i32 {
//     type Output = u32;
//     fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
//         a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output
//     }
// }

// impl SafeSubtract for u32 {
//     type Output = u32;
//     fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
//         a - b
//     }
// }

// impl SafeSubtract for i64 {
//     type Output = u64;
//     fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
//         a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output
//     }
// }

// impl SafeSubtract for u64 {
//     type Output = u64;
//     fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
//         a - b
//     }
// }

// impl SafeSubtract for isize {
//     type Output = usize;
//     fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
//         a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output
//     }
// }

// impl SafeSubtract for usize {
//     type Output = usize;
//     fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
//         a - b
//     }
// }

// impl SafeSubtract for i128 {
//     type Output = u128;
//     fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
//         a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output
//     }
// }

// impl SafeSubtract for u128 {
//     type Output = u128;
//     fn safe_subtract(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
//         a - b
//     }
// }

// pub fn test_me<T>(i: T)
// where
//     T: Integer,
// {
//     let _j = i;
// }

// pub fn test_me_i32(i: i32) {
//     test_me(i);
// }
