use std::ops::RangeInclusive;

use itertools::{Itertools, MergeBy};

use crate::{Integer, SortedDisjoint, SortedStarts};

#[derive(Clone, Debug)]
pub(crate) enum CMKMerge<T: Integer, I>
where
    I: SortedDisjoint<T>,
{
    None,
    Some(MergeBy<I, Box<CMKMerge<T, I>>, fn(&RangeInclusive<T>, &RangeInclusive<T>) -> bool>),
}

impl<T: Integer, I> SortedDisjoint<T> for CMKMerge<T, I> where I: SortedDisjoint<T> {}
impl<T: Integer, I> SortedStarts<T> for CMKMerge<T, I> where I: SortedDisjoint<T> {}

impl<T: Integer, I> Iterator for CMKMerge<T, I>
where
    I: SortedDisjoint<T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CMKMerge::None => None,
            CMKMerge::Some(i2) => i2.next(),
        }
    }
}

impl<'a, T: Integer, I> CMKMerge<T, I>
where
    I: SortedDisjoint<T>,
{
    fn compare_start(a: &RangeInclusive<T>, b: &RangeInclusive<T>) -> bool {
        a.start() < b.start()
    }

    pub fn new<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = I>,
    {
        iter.into_iter().fold(CMKMerge::None, |kmerge, item| {
            CMKMerge::Some(item.merge_by(Box::new(kmerge), Self::compare_start))
        })
    }
}
