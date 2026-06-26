//! Tests

#![cfg(test)]
#![cfg(feature = "total_float_experimental")]

use num_traits::identities::One;
use num_traits::identities::Zero;
use range_set_blaze::{Integer, RangeMapBlaze, RangeSetBlaze, TotalF32, TotalF64};
#[cfg(feature = "total_float_nightly_experimental")]
use range_set_blaze::{TotalF16, TotalF128, UIntPlusOne};
use syntactic_for::syntactic_for;

// fn test_singleton_iter<T, I>(iter: I, value: T)
// where
//     T: Integer,
//     I: IntoIterator<Item = T>,
// {
//     let iter = iter.into_iter();
//     assert_eq!(iter.next(), Some(value));
//     assert_eq!(iter.next(), None);
// }

// #[test]
// fn test_singletons()
// {
//     test_singleton_iter(TotalF64::MAX..=TotalF64::MAX, TotalF64::MAX);
//     test_singleton_iter(TotalF64::MIN..=TotalF64::MIN, TotalF64::MIN);
//     test_singleton_iter(TotalF32::MAX..=TotalF32::MAX, TotalF32::MAX);
//     test_singleton_iter(TotalF32::MIN..=TotalF32::MIN, TotalF32::MIN);
// }

#[test]
fn map_complement0() {
    let empty = RangeMapBlaze::<TotalF64, u8>::new();
    assert_eq!(empty.len(), 0);
    let full = !&empty;
    assert_eq!(full.len(), i128::from(u64::MAX) + 1);
    let empty = !&full;
    assert_eq!(empty.len(), 0);

    let empty = RangeMapBlaze::<TotalF32, u8>::new();
    assert_eq!(empty.len(), 0);
    let full = !&empty;
    assert_eq!(full.len(), i64::from(u32::MAX) + 1);
    let empty = !&full;
    assert_eq!(empty.len(), 0);
}

#[test]
fn set_complement0() {
    let empty = RangeSetBlaze::<TotalF64>::new();
    assert_eq!(empty.len(), 0);
    let full = !&empty;
    assert_eq!(full.len(), i128::from(u64::MAX) + 1);
    let empty = !&full;
    assert_eq!(empty.len(), 0);

    let empty = RangeSetBlaze::<TotalF32>::new();
    assert_eq!(empty.len(), 0);
    let full = !&empty;
    assert_eq!(full.len(), i64::from(u32::MAX) + 1);
    let empty = !&full;
    assert_eq!(empty.len(), 0);
}

#[test]
#[allow(clippy::cognitive_complexity, clippy::float_cmp)]
fn integer_coverage() {
    syntactic_for! { ty in [TotalF32, TotalF64] {
        $(
            let len = <$ty as Integer>::SafeLen::one();
            let a = $ty::zero();
            assert_eq!($ty::safe_len_to_f64_lossy(len), 1.0);
            assert_eq!($ty::inclusive_end_from_start(a,len), a);
            assert_eq!($ty::start_from_inclusive_end(a,len), a);
            assert_eq!($ty::f64_to_safe_len_lossy(1.0), len);

        )*
    }};
}

#[test]
// I don't quite understand why clippy complains here
#[expect(clippy::from_iter_instead_of_collect)]
fn float_test() {
    let _ = RangeSetBlaze::<TotalF64>::new();
    let _ = RangeSetBlaze::<TotalF32>::new();

    let _ = RangeSetBlaze::from_iter([TotalF64(3.0)..=TotalF64(5.0)]);
    let _ = RangeSetBlaze::from_iter([TotalF32(3.0)..=TotalF32(5.0)]);
    let _ = RangeSetBlaze::from_iter([TotalF64(1.0), TotalF64(2.0), TotalF64(3.0)]);
    let _ = RangeSetBlaze::from_iter([TotalF32(1.0), TotalF32(2.0), TotalF32(3.0)]);

    let _ = RangeSetBlaze::from(TotalF64(3.0)..=TotalF64(5.0));
    let _ = RangeSetBlaze::from(TotalF32(3.0)..=TotalF32(5.0));

    let _ = RangeSetBlaze::from(TotalF64::range(3.0..=5.0));
    let _ = RangeSetBlaze::from(TotalF32::range(3.0..=5.0));

    let _ = RangeSetBlaze::from_iter(TotalF64::ranges([3.0..=5.0, 7.0..=9.0]));
    let _ = RangeSetBlaze::from_iter(TotalF32::ranges([3.0..=5.0, 7.0..=9.0]));

    let _ = RangeSetBlaze::from_iter(TotalF64::slice(&[1.0, 2.0, 3.0]));
    let _ = RangeSetBlaze::from_iter(TotalF32::slice(&[1.0, 2.0, 3.0]));
    let _ = RangeSetBlaze::from_iter(TotalF64::values([1.0, 2.0, 3.0]));
    let _ = RangeSetBlaze::from_iter(TotalF32::values([1.0, 2.0, 3.0]));

    let _ = RangeSetBlaze::from(TotalF64::ALL_VALUES);
    let _ = RangeSetBlaze::from(TotalF32::ALL_VALUES);
    let _ = RangeSetBlaze::from(TotalF64::MIN_VALUE..=TotalF64::MAX_VALUE);
    let _ = RangeSetBlaze::from(TotalF32::MIN_VALUE..=TotalF32::MAX_VALUE);

    let foo = RangeSetBlaze::from_iter(TotalF64::ranges([3.0..=5.0, 7.0..=9.0]));
    assert!(foo.contains(TotalF64(3.0)));
    assert!(foo.contains(TotalF64(5.0)));
    assert!(foo.contains(TotalF64(7.0)));
    assert!(foo.contains(TotalF64(9.0)));

    assert!(foo.contains(TotalF64(3.01)));
    assert!(foo.contains(TotalF64(4.99)));
    assert!(foo.contains(TotalF64(7.01)));
    assert!(foo.contains(TotalF64(8.99)));

    assert!(!foo.contains(TotalF64(2.99)));
    assert!(!foo.contains(TotalF64(5.01)));
    assert!(!foo.contains(TotalF64(6.99)));
    assert!(!foo.contains(TotalF64(9.01)));

    assert!(!foo.contains(TotalF64(3.0).prev()));
    assert!(!foo.contains(TotalF64(5.0).next()));
    assert!(!foo.contains(TotalF64(7.0).prev()));
    assert!(!foo.contains(TotalF64(9.0).next()));

    assert!(foo.contains(TotalF64(3.0).next()));
    assert!(foo.contains(TotalF64(5.0).prev()));
    assert!(foo.contains(TotalF64(7.0).next()));
    assert!(foo.contains(TotalF64(9.0).prev()));
}

#[test]
fn test_inclusive() {
    syntactic_for! { ty in [TotalF32, TotalF64] {
        $(
    let a = <$ty>::min_value();
    let b = <$ty>::max_value();
    let len = <$ty>::safe_len(&(a..=b));
    assert_eq!(<$ty>::inclusive_end_from_start(a, len), b);
    assert_eq!(<$ty>::start_from_inclusive_end(b, len), a);
        )*
    }}
}

#[test]
fn test_floats() {
    let mut a = TotalF32::range(0.0..=0.0);
    assert_eq!(TotalF32::range_next_back(&mut a), Some(TotalF32(0.0)));
    assert_eq!(TotalF32::range_next(&mut a), None);

    let mut a = TotalF64::range(0.0..=0.0);
    assert_eq!(TotalF64::range_next_back(&mut a), Some(TotalF64(0.0)));
    assert_eq!(TotalF64::range_next(&mut a), None);

    let mut b = TotalF64(0.0);
    TotalF64::assign_sub_one(&mut b);
    assert_eq!(b, TotalF64(0.0).prev());

    let mut b = TotalF32(0.0);
    TotalF32::assign_sub_one(&mut b);
    assert_eq!(b, TotalF32(0.0).prev());
}

#[test]
#[cfg(feature = "total_float_nightly_experimental")]
fn test_inclusive_nightly() {
    syntactic_for! { ty in [TotalF16, TotalF128] {
        $(
    let a = <$ty>::min_value();
    let b = <$ty>::max_value();
    let len = <$ty>::safe_len(&(a..=b));
    assert_eq!(<$ty>::inclusive_end_from_start(a, len), b);
    assert_eq!(<$ty>::start_from_inclusive_end(b, len), a);
        )*
    }}
}

#[test]
#[cfg(feature = "total_float_nightly_experimental")]
fn test_floats_nightly() {
    let mut a = TotalF16::range(0.0..=0.0);
    assert_eq!(TotalF16::range_next_back(&mut a), Some(TotalF16(0.0)));
    assert_eq!(TotalF16::range_next(&mut a), None);

    let mut a = TotalF128::range(0.0..=0.0);
    assert_eq!(TotalF128::range_next_back(&mut a), Some(TotalF128(0.0)));
    assert_eq!(TotalF128::range_next(&mut a), None);

    let mut b = TotalF16(0.0);
    TotalF16::assign_sub_one(&mut b);
    assert_eq!(b, TotalF16(0.0).prev());

    let mut b = TotalF128(0.0);
    TotalF128::assign_sub_one(&mut b);
    assert_eq!(b, TotalF128(0.0).prev());

    // convert  UIntPlusOne::MaxPlusOne to f128 and back
    let f = TotalF128::safe_len_to_f64_lossy(UIntPlusOne::MaxPlusOne);
    let i = TotalF128::f64_to_safe_len_lossy(f);
    assert_eq!(i, UIntPlusOne::MaxPlusOne);
}

#[test]
fn total_f64_iterators() {
    // MAX forward
    let set = RangeSetBlaze::from_iter([TotalF64::MAX..=TotalF64::MAX]);
    let mut iter = set.iter();
    assert_eq!(iter.next(), Some(TotalF64::MAX));
    assert_eq!(iter.next(), None);

    // MAX reverse
    let set = RangeSetBlaze::from_iter([TotalF64::MAX..=TotalF64::MAX]);
    let mut iter = set.iter().rev();
    assert_eq!(iter.next(), Some(TotalF64::MAX));
    assert_eq!(iter.next(), None);

    // MIN forward
    let set = RangeSetBlaze::from_iter([TotalF64::MIN..=TotalF64::MIN]);
    let mut iter = set.iter();
    assert_eq!(iter.next(), Some(TotalF64::MIN));
    assert_eq!(iter.next(), None);

    // MIN reverse
    let set = RangeSetBlaze::from_iter([TotalF64::MIN..=TotalF64::MIN]);
    let mut iter = set.iter().rev();
    assert_eq!(iter.next(), Some(TotalF64::MIN));
    assert_eq!(iter.next(), None);
}

#[test]
fn total_f32_iterators() {
    // MAX forward
    let set = RangeSetBlaze::from_iter([TotalF32::MAX..=TotalF32::MAX]);
    let mut iter = set.iter();
    assert_eq!(iter.next(), Some(TotalF32::MAX));
    assert_eq!(iter.next(), None);

    // MAX reverse
    let set = RangeSetBlaze::from_iter([TotalF32::MAX..=TotalF32::MAX]);
    let mut iter = set.iter().rev();
    assert_eq!(iter.next(), Some(TotalF32::MAX));
    assert_eq!(iter.next(), None);

    // MIN forward
    let set = RangeSetBlaze::from_iter([TotalF32::MIN..=TotalF32::MIN]);
    let mut iter = set.iter();
    assert_eq!(iter.next(), Some(TotalF32::MIN));
    assert_eq!(iter.next(), None);

    // MIN reverse
    let set = RangeSetBlaze::from_iter([TotalF32::MIN..=TotalF32::MIN]);
    let mut iter = set.iter().rev();
    assert_eq!(iter.next(), Some(TotalF32::MIN));
    assert_eq!(iter.next(), None);
}

#[test]
#[cfg(feature = "total_float_nightly_experimental")]
fn total_f16_iterators() {
    // MAX forward
    let set = RangeSetBlaze::from_iter([TotalF16::MAX..=TotalF16::MAX]);
    let mut iter = set.iter();
    assert_eq!(iter.next(), Some(TotalF16::MAX));
    assert_eq!(iter.next(), None);

    // MAX reverse
    let set = RangeSetBlaze::from_iter([TotalF16::MAX..=TotalF16::MAX]);
    let mut iter = set.iter().rev();
    assert_eq!(iter.next(), Some(TotalF16::MAX));
    assert_eq!(iter.next(), None);

    // MIN forward
    let set = RangeSetBlaze::from_iter([TotalF16::MIN..=TotalF16::MIN]);
    let mut iter = set.iter();
    assert_eq!(iter.next(), Some(TotalF16::MIN));
    assert_eq!(iter.next(), None);

    // MIN reverse
    let set = RangeSetBlaze::from_iter([TotalF16::MIN..=TotalF16::MIN]);
    let mut iter = set.iter().rev();
    assert_eq!(iter.next(), Some(TotalF16::MIN));
    assert_eq!(iter.next(), None);
}

#[test]
#[cfg(feature = "total_float_nightly_experimental")]
fn total_f128_iterators() {
    // MAX forward
    let set = RangeSetBlaze::from_iter([TotalF128::MAX..=TotalF128::MAX]);
    let mut iter = set.iter();
    assert_eq!(iter.next(), Some(TotalF128::MAX));
    assert_eq!(iter.next(), None);

    // MAX reverse
    let set = RangeSetBlaze::from_iter([TotalF128::MAX..=TotalF128::MAX]);
    let mut iter = set.iter().rev();
    assert_eq!(iter.next(), Some(TotalF128::MAX));
    assert_eq!(iter.next(), None);

    // MIN forward
    let set = RangeSetBlaze::from_iter([TotalF128::MIN..=TotalF128::MIN]);
    let mut iter = set.iter();
    assert_eq!(iter.next(), Some(TotalF128::MIN));
    assert_eq!(iter.next(), None);

    // MIN reverse
    let set = RangeSetBlaze::from_iter([TotalF128::MIN..=TotalF128::MIN]);
    let mut iter = set.iter().rev();
    assert_eq!(iter.next(), Some(TotalF128::MIN));
    assert_eq!(iter.next(), None);
}

#[test]
fn total_f64_complement() {
    let set = RangeSetBlaze::from_iter([TotalF64::MAX..=TotalF64::MAX]);
    assert!(set.contains(TotalF64::MAX));
    assert!(!set.contains(TotalF64::MAX.prev()));
    assert_eq!(set.len(), 1);
    let set = !set;
    assert!(!set.contains(TotalF64::MAX));
    assert!(set.contains(TotalF64::MAX.prev()));
    assert_eq!(set.len(), u64::MAX as <TotalF64 as Integer>::SafeLen);

    let set = RangeSetBlaze::from_iter([TotalF64::MIN..=TotalF64::MIN]);
    assert!(set.contains(TotalF64::MIN));
    assert!(!set.contains(TotalF64::MIN.next()));
    assert_eq!(set.len(), 1);
    let set = !set;
    assert!(!set.contains(TotalF64::MIN));
    assert!(set.contains(TotalF64::MIN.next()));
    assert_eq!(set.len(), u64::MAX as <TotalF64 as Integer>::SafeLen);

    let set = RangeSetBlaze::from_iter([TotalF64::MIN..=TotalF64::MIN.next()]);
    assert!(set.contains(TotalF64::MIN));
    assert!(set.contains(TotalF64::MIN.next()));
    assert!(!set.contains(TotalF64::MIN.next().next()));
    assert_eq!(set.len(), 2);
    let set = !set;
    assert!(!set.contains(TotalF64::MIN));
    assert!(!set.contains(TotalF64::MIN.next()));
    assert!(set.contains(TotalF64::MIN.next().next()));
    assert_eq!(set.len(), (u64::MAX as <TotalF64 as Integer>::SafeLen) - 1);
}

#[test]
fn total_f32_complement() {
    let set = RangeSetBlaze::from_iter([TotalF32::MAX..=TotalF32::MAX]);
    assert!(set.contains(TotalF32::MAX));
    assert!(!set.contains(TotalF32::MAX.prev()));
    assert_eq!(set.len(), 1);
    let set = !set;
    assert!(!set.contains(TotalF32::MAX));
    assert!(set.contains(TotalF32::MAX.prev()));
    assert_eq!(set.len(), u32::MAX as <TotalF32 as Integer>::SafeLen);

    let set = RangeSetBlaze::from_iter([TotalF32::MIN..=TotalF32::MIN]);
    assert!(set.contains(TotalF32::MIN));
    assert!(!set.contains(TotalF32::MIN.next()));
    assert_eq!(set.len(), 1);
    let set = !set;
    assert!(!set.contains(TotalF32::MIN));
    assert!(set.contains(TotalF32::MIN.next()));
    assert_eq!(set.len(), u32::MAX as <TotalF32 as Integer>::SafeLen);
}

#[test]
#[cfg(feature = "total_float_nightly_experimental")]
fn total_f16_complement() {
    let set = RangeSetBlaze::from_iter([TotalF16::MAX..=TotalF16::MAX]);
    assert!(set.contains(TotalF16::MAX));
    assert!(!set.contains(TotalF16::MAX.prev()));
    assert_eq!(set.len(), 1);
    let set = !set;
    assert!(!set.contains(TotalF16::MAX));
    assert!(set.contains(TotalF16::MAX.prev()));
    assert_eq!(set.len(), u16::MAX as <TotalF16 as Integer>::SafeLen);

    let set = RangeSetBlaze::from_iter([TotalF16::MIN..=TotalF16::MIN]);
    assert!(set.contains(TotalF16::MIN));
    assert!(!set.contains(TotalF16::MIN.next()));
    assert_eq!(set.len(), 1);
    let set = !set;
    assert!(!set.contains(TotalF16::MIN));
    assert!(set.contains(TotalF16::MIN.next()));
    assert_eq!(set.len(), u16::MAX as <TotalF16 as Integer>::SafeLen);
}

#[test]
#[cfg(feature = "total_float_nightly_experimental")]
fn total_f128_complement() {
    let set = RangeSetBlaze::from_iter([TotalF128::MAX..=TotalF128::MAX]);
    assert!(set.contains(TotalF128::MAX));
    assert!(!set.contains(TotalF128::MAX.prev()));
    assert_eq!(set.len(), range_set_blaze::UIntPlusOne::UInt(1));
    let set = !set;
    assert!(!set.contains(TotalF128::MAX));
    assert!(set.contains(TotalF128::MAX.prev()));
    assert_eq!(set.len(), UIntPlusOne::UInt(u128::MAX));

    let set = RangeSetBlaze::from_iter([TotalF128::MIN..=TotalF128::MIN]);
    assert!(set.contains(TotalF128::MIN));
    assert!(!set.contains(TotalF128::MIN.next()));
    assert_eq!(set.len(), range_set_blaze::UIntPlusOne::UInt(1));
    let set = !set;
    assert!(!set.contains(TotalF128::MIN));
    assert!(set.contains(TotalF128::MIN.next()));
    assert_eq!(set.len(), UIntPlusOne::UInt(u128::MAX));
}
