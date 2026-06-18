//! Tests

#![cfg(test)]
// use range_set_blaze::{
//     AssumeSortedStarts, F32, F64, IntoIter, IntoRangesIter, Iter, KMerge,
//     Merge, RangeOnce, RangeValuesIter, RangeValuesToRangesIter, RangesIter, RangeSetBlaze
// };
use range_set_blaze::{F32, F64, RangeSetBlaze};

#[test]
// I don't quite understand why clippy complains here
#[expect(clippy::from_iter_instead_of_collect)]
fn float_test() {
    let _ = RangeSetBlaze::<F64>::new();
    let _ = RangeSetBlaze::<F32>::new();

    let _ = RangeSetBlaze::from_iter([F64(3.0)..=F64(5.0)]);
    let _ = RangeSetBlaze::from_iter([F32(3.0)..=F32(5.0)]);
    let _ = RangeSetBlaze::from_iter([F64(1.0), F64(2.0), F64(3.0)]);
    let _ = RangeSetBlaze::from_iter([F32(1.0), F32(2.0), F32(3.0)]);

    let _ = RangeSetBlaze::from(F64(3.0)..=F64(5.0));
    let _ = RangeSetBlaze::from(F32(3.0)..=F32(5.0));

    let _ = RangeSetBlaze::from(F64::range(3.0..=5.0));
    let _ = RangeSetBlaze::from(F32::range(3.0..=5.0));

    let _ = RangeSetBlaze::from_iter(F64::ranges([3.0..=5.0, 7.0..=9.0]));
    let _ = RangeSetBlaze::from_iter(F32::ranges([3.0..=5.0, 7.0..=9.0]));

    let _ = RangeSetBlaze::from_iter(F64::slice(&[1.0, 2.0, 3.0]));
    let _ = RangeSetBlaze::from_iter(F32::slice(&[1.0, 2.0, 3.0]));
    let _ = RangeSetBlaze::from_iter(F64::values([1.0, 2.0, 3.0]));
    let _ = RangeSetBlaze::from_iter(F32::values([1.0, 2.0, 3.0]));

    let _ = RangeSetBlaze::from(F64::ALL_VALUES);
    let _ = RangeSetBlaze::from(F32::ALL_VALUES);
    let _ = RangeSetBlaze::from(F64::MIN_VALUE..=F64::MAX_VALUE);
    let _ = RangeSetBlaze::from(F32::MIN_VALUE..=F32::MAX_VALUE);

    let foo = RangeSetBlaze::from_iter(F64::ranges([3.0..=5.0, 7.0..=9.0]));
    assert!(foo.contains(F64(3.0)));
    assert!(foo.contains(F64(5.0)));
    assert!(foo.contains(F64(7.0)));
    assert!(foo.contains(F64(9.0)));

    assert!(foo.contains(F64(3.01)));
    assert!(foo.contains(F64(4.99)));
    assert!(foo.contains(F64(7.01)));
    assert!(foo.contains(F64(8.99)));

    assert!(!foo.contains(F64(2.99)));
    assert!(!foo.contains(F64(5.01)));
    assert!(!foo.contains(F64(6.99)));
    assert!(!foo.contains(F64(9.01)));

    assert!(!foo.contains(F64(3.0).prev()));
    assert!(!foo.contains(F64(5.0).next()));
    assert!(!foo.contains(F64(7.0).prev()));
    assert!(!foo.contains(F64(9.0).next()));

    assert!(foo.contains(F64(3.0).next()));
    assert!(foo.contains(F64(5.0).prev()));
    assert!(foo.contains(F64(7.0).next()));
    assert!(foo.contains(F64(9.0).prev()));
}
