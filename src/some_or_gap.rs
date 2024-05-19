#![cfg(feature = "rog-experimental")]
#![deprecated(
    note = "The some_or_gap  module is experimental and may be changed or removed in future versions.
    Changes may not be reflected in the semantic versioning."
)]

use core::ops::RangeInclusive;

use crate::Integer;

/// Experimental: Represents an range or gap in a [`RangeMapBlaze`].
///
/// See [`RangeMapBlaze::get_range_value`] for an example.
///
/// [`RangeMapBlaze`]: crate::RangeMapBlaze
/// [`RangeMapBlaze::get_range_value`]: crate::RangeMapBlaze::get_range_value
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
