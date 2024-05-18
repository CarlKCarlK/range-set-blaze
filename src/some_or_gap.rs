#![cfg(feature = "rog-experimental")]

use core::ops::RangeInclusive;

use crate::Integer;

/// cmk doc
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SomeOrGap<S, T>
where
    T: Integer,
{
    /// cmk doc
    Some(S),
    /// cmk doc
    Gap(RangeInclusive<T>),
}

impl<S, T> SomeOrGap<S, T>
where
    T: Integer,
{
    /// cmk doc
    pub fn unwrap(self) -> S {
        match self {
            SomeOrGap::Some(value) => value,
            SomeOrGap::Gap(range) => {
                panic!("called `SomeOrGap::unwrap()` on a `Gap` value: {:?}", range)
            }
        }
    }

    /// cmk doc Method to check if the variant is Some
    pub fn is_some(&self) -> bool {
        matches!(self, SomeOrGap::Some(_))
    }

    /// cmk doc Method to check if the variant is Gap
    pub fn is_gap(&self) -> bool {
        matches!(self, SomeOrGap::Gap(_))
    }

    /// cmk doc
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
