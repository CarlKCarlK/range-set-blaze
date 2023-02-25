// !!!cmk make the names consistent, start/lower vs stop/upper/end/...
// !!!cmk replace OptionRange with Option<(T, T)>

use num_traits::Zero;
use std::{
    cmp::{max, min},
    vec,
};

use itertools::Itertools;

use crate::{Integer, OptionRange, SafeSubtract, SortedDisjoint};

pub struct UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    iter: I,
    range: OptionRange<T>,
    two: T,
}

impl<T, I> From<I> for UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>, // cmk should this be IntoIterator?
{
    fn from(iter: I) -> Self {
        UnsortedDisjoint {
            iter,
            range: OptionRange::None,
            two: T::one() + T::one(),
        }
    }
}

impl<T, I> Iterator for UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((lower, upper)) = self.iter.next() {
            assert!(lower <= upper && upper <= T::max_value2()); // !!!cmk0 raise error on panic?
            if let OptionRange::Some {
                start: self_lower,
                stop: self_upper,
            } = self.range
            {
                if (lower >= self.two && lower - self.two >= self_upper)
                    || (self_lower >= self.two && self_lower - self.two >= upper)
                {
                    let result = Some((self_lower, self_upper));
                    self.range = OptionRange::Some {
                        start: lower,
                        stop: upper,
                    };
                    result
                } else {
                    self.range = OptionRange::Some {
                        start: min(self_lower, lower),
                        stop: max(self_upper, upper),
                    };
                    self.next()
                }
            } else {
                self.range = OptionRange::Some {
                    start: lower,
                    stop: upper,
                };
                self.next()
            }
        } else if let OptionRange::Some { start, stop } = self.range {
            self.range = OptionRange::None;
            Some((start, stop))
        } else {
            None
        }
    }
}

pub(crate) struct SortedDisjointFromIter<T>
where
    T: Integer,
{
    vec_iter: vec::IntoIter<(T, T)>,
    range: OptionRange<T>,
}

impl<T: Integer> SortedDisjointFromIter<T> {
    pub fn new<I>(unsorted_disjoint: UnsortedDisjoint<T, I>) -> Self
    where
        I: Iterator<Item = (T, T)>,
    {
        SortedDisjointFromIter {
            vec_iter: unsorted_disjoint.sorted_by_key(|(start, _)| *start),
            range: OptionRange::None,
        }
    }
}

impl<T: Integer> FromIterator<(T, T)> for SortedDisjointFromIter<T> {
    fn from_iter<I: IntoIterator<Item = (T, T)>>(iter: I) -> Self {
        Self::new(UnsortedDisjoint::from(iter.into_iter()))
    }
}

impl<T: Integer> FromIterator<T> for SortedDisjointFromIter<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        iter.into_iter().map(|x| (x, x)).collect()
    }
}

impl<T: Integer> SortedDisjoint for SortedDisjointFromIter<T> {}

impl<T: Integer> Iterator for SortedDisjointFromIter<T> {
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((start, stop)) = self.vec_iter.next() {
            debug_assert!(start <= stop && stop <= T::max_value2());
            if let OptionRange::Some {
                start: self_start,
                stop: self_stop,
            } = self.range
            {
                if start <= self_stop
                    || (self_stop < T::max_value2() && start <= self_stop + T::one())
                {
                    self.range = OptionRange::Some {
                        start: self_start,
                        stop: max(self_stop, stop),
                    };
                    self.next()
                } else {
                    self.range = OptionRange::Some { start, stop };
                    Some((self_start, self_stop))
                }
            } else {
                self.range = OptionRange::Some { start, stop };
                self.next()
            }
        } else if let OptionRange::Some { start, stop } = self.range {
            self.range = OptionRange::None;
            Some((start, stop))
        } else {
            None
        }
    }
}

pub struct SortedDisjointWithLenSoFar<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)> + SortedDisjoint,
{
    iter: I,
    len: <T as SafeSubtract>::Output,
}

impl<T: Integer, I> From<I> for SortedDisjointWithLenSoFar<T, I>
where
    I: Iterator<Item = (T, T)> + SortedDisjoint,
{
    fn from(iter: I) -> Self {
        SortedDisjointWithLenSoFar {
            iter,
            len: <T as SafeSubtract>::Output::zero(),
        }
    }
}

impl<T: Integer, I> SortedDisjointWithLenSoFar<T, I>
where
    I: Iterator<Item = (T, T)> + SortedDisjoint,
{
    pub fn len(&self) -> <T as SafeSubtract>::Output {
        self.len.clone()
    }
}

impl<T: Integer, I> Iterator for SortedDisjointWithLenSoFar<T, I>
where
    I: Iterator<Item = (T, T)> + SortedDisjoint,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((start, stop)) = self.iter.next() {
            debug_assert!(start <= stop && stop <= T::max_value2());
            self.len += T::safe_subtract_inclusive(stop, start);
            Some((start, stop))
        } else {
            None
        }
    }
}
impl<T: Integer, I> SortedDisjoint for SortedDisjointWithLenSoFar<T, I> where
    I: Iterator<Item = (T, T)> + SortedDisjoint
{
}
