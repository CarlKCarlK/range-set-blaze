//! Tests

#![cfg(test)]
#![cfg(feature = "total_float_experimental")]

use num_traits::identities::One;
use num_traits::identities::Zero;
use range_set_blaze::{Integer, RangeMapBlaze, RangeSetBlaze, TotalF32, TotalF64};
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
fn test_use_of_as_00() {
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
