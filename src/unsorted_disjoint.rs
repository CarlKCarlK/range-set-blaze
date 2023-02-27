// !!!cmk make the names consistent, start/lower vs stop/upper/end/...
// !!!cmk replace OptionRange with Option<(T, T)>

use num_traits::Zero;
use std::cmp::{max, min};

use crate::{Integer, SafeSubtract, SortedDisjoint, SortedStarts};

pub struct UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    iter: I,
    range: Option<(T, T)>,
    two: T,
}

impl<T, I> SortedStarts for UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
}

impl<T, I> From<I> for UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>, // cmk should this be IntoIterator?
{
    fn from(iter: I) -> Self {
        UnsortedDisjoint {
            iter,
            range: None,
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
            if let Some((self_lower, self_upper)) = self.range {
                if (lower >= self.two && lower - self.two >= self_upper)
                    || (self_lower >= self.two && self_lower - self.two >= upper)
                {
                    let result = Some((self_lower, self_upper));
                    self.range = Some((lower, upper));
                    result
                } else {
                    self.range = Some((min(self_lower, lower), max(self_upper, upper)));
                    self.next()
                }
            } else {
                self.range = Some((lower, upper));
                self.next()
            }
        } else if let Some((start, stop)) = self.range {
            self.range = None;
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

// cmk Rule there is no reason From's should be into iterators
impl<T: Integer, I> From<I> for SortedDisjointWithLenSoFar<T, I::IntoIter>
where
    I: IntoIterator<Item = (T, T)>,
    I::IntoIter: SortedDisjoint,
{
    fn from(into_iter: I) -> Self {
        SortedDisjointWithLenSoFar {
            iter: into_iter.into_iter(),
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
    I: Iterator<Item = (T, T)> + SortedDisjoint, // cmk000 should we name this?
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
impl<T: Integer, I> SortedStarts for SortedDisjointWithLenSoFar<T, I> where
    I: Iterator<Item = (T, T)> + SortedDisjoint
{
}

#[derive(Clone)]
pub struct AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    pub(crate) iter: I,
}

impl<T: Integer, I> SortedStarts for AssumeSortedStarts<T, I> where I: Iterator<Item = (T, T)> {}

impl<T, I> Iterator for AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = (T, T)>,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
