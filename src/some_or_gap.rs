use core::ops::RangeInclusive;

use crate::Integer;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SomeOrGap<S, T>
where
    T: Integer,
{
    Some(S),
    Gap(RangeInclusive<T>),
}

impl<S, T> SomeOrGap<S, T>
where
    T: Integer,
{
    pub fn unwrap(self) -> S {
        match self {
            SomeOrGap::Some(value) => value,
            SomeOrGap::Gap(range) => {
                panic!("called `SomeOrGap::unwrap()` on a `Gap` value: {:?}", range)
            }
        }
    }

    // Method to check if the variant is Some
    pub fn is_some(&self) -> bool {
        matches!(self, SomeOrGap::Some(_))
    }

    // Method to check if the variant is Gap
    pub fn is_gap(&self) -> bool {
        matches!(self, SomeOrGap::Gap(_))
    }

    pub fn unwrap_or_else<F>(self, f: F) -> S
    where
        F: FnOnce(RangeInclusive<T>) -> S,
    {
        match self {
            SomeOrGap::Some(value) => value,
            SomeOrGap::Gap(range) => f(range),
        }
    }
}
