//! Tests

#![cfg(test)]
#![cfg(feature = "float_experimental")]
#![cfg_attr(feature = "float_nightly_experimental", feature(f16))]
#![cfg_attr(feature = "float_nightly_experimental", feature(f128))]

use num_traits::identities::{One, Zero};
#[cfg(feature = "float_nightly_experimental")]
use range_set_blaze::UIntPlusOne;
#[cfg(feature = "float_nightly_experimental")]
use range_set_blaze::finite::{ff16, ff128};
use range_set_blaze::finite::{ff32, ff64};
use range_set_blaze::total::{TotalRangeExt, tf32, tf64};
#[cfg(feature = "float_nightly_experimental")]
use range_set_blaze::total::{tf16, tf128};
#[cfg(feature = "float_nightly_experimental")]
use range_set_blaze::{FiniteF16, FiniteF128, TotalF16, TotalF128};
use range_set_blaze::{
    FiniteF32, FiniteF64, Integer, RangeMapBlaze, RangeSetBlaze, TotalF32, TotalF64,
};
use syntactic_for::syntactic_for;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

const CONST_FINITE_F32: FiniteF32 = ff32(-0.0);
const CONST_FINITE_F64: FiniteF64 = ff64(-0.0);
#[cfg(feature = "float_nightly_experimental")]
const CONST_FINITE_F16: FiniteF16 = ff16(1.0);
#[cfg(feature = "float_nightly_experimental")]
const CONST_FINITE_F128: FiniteF128 = ff128(-0.0);
macro_rules! assert_empty_complement {
    (map, $ty:ty) => {
        let empty = RangeMapBlaze::<$ty, u8>::new();
        assert_eq!(empty.len(), <$ty as Integer>::SafeLen::zero());
        let full = !&empty;
        assert_eq!(full.len(), <$ty>::MAX_SIZE);
        let empty = !&full;
        assert_eq!(empty.len(), <$ty as Integer>::SafeLen::zero());
    };
    (set, $ty:ty) => {
        let empty = RangeSetBlaze::<$ty>::new();
        assert_eq!(empty.len(), <$ty as Integer>::SafeLen::zero());
        let full = !&empty;
        assert_eq!(full.len(), <$ty>::MAX_SIZE);
        let empty = !&full;
        assert_eq!(empty.len(), <$ty as Integer>::SafeLen::zero());
    };
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn finite_shorthands_are_const() {
    assert_eq!(CONST_FINITE_F32, ff32(0.0));
    assert_eq!(CONST_FINITE_F64.into_inner().to_bits(), 0.0_f64.to_bits());
    assert_eq!(ff32(-0.0).into_inner().to_bits(), 0.0_f32.to_bits());
    assert_eq!(ff64(-0.0).into_inner().to_bits(), 0.0_f64.to_bits());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn finite_nightly_shorthands_are_const() {
    assert_eq!(CONST_FINITE_F16, ff16(1.0));
    assert_eq!(CONST_FINITE_F128.into_inner().to_bits(), 0.0_f128.to_bits());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_complement0() {
    assert!(<f64 as PartialEq>::eq(&0.0, &-0.0));
    assert_ne!(tf64(0.0), tf64(-0.0));
    assert_eq!(ff64(0.0), ff64(-0.0));
    assert_ne!(tf32(0.0), tf32(-0.0));
    assert_eq!(ff32(0.0), ff32(-0.0));

    syntactic_for! { ty in [TotalF32, TotalF64, FiniteF32, FiniteF64] {
        $(assert_empty_complement!(map, $ty);)*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn map_complement0_nightly() {
    assert_ne!(tf16(0.0), tf16(-0.0));
    assert_eq!(ff16(0.0), ff16(-0.0));
    assert_ne!(tf128(0.0), tf128(-0.0));
    assert_eq!(ff128(0.0), ff128(-0.0));

    syntactic_for! { ty in [TotalF16, TotalF128, FiniteF16, FiniteF128] {
        $(assert_empty_complement!(map, $ty);)*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn set_complement0() {
    syntactic_for! { ty in [TotalF32, TotalF64, FiniteF32, FiniteF64] {
        $(assert_empty_complement!(set, $ty);)*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn set_complement_nightly() {
    syntactic_for! { ty in [TotalF16, TotalF128, FiniteF16, FiniteF128] {
        $(assert_empty_complement!(set, $ty);)*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::cognitive_complexity, clippy::float_cmp)]
fn integer_coverage() {
    syntactic_for! { ty in [TotalF32, TotalF64, FiniteF32, FiniteF64] {
        $(
            let len = <$ty as Integer>::SafeLen::one();
            let a = $ty::new(42.0);
            assert_eq!($ty::safe_len_to_f64_lossy(len), 1.0);
            assert_eq!($ty::inclusive_end_from_start(a,len), a);
            assert_eq!($ty::start_from_inclusive_end(a,len), a);
            assert_eq!($ty::f64_to_safe_len_lossy(1.0), len);

        )*
    }};
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
#[allow(clippy::cognitive_complexity, clippy::float_cmp)]
fn integer_coverage_nightly() {
    syntactic_for! { ty in [TotalF16, TotalF128, FiniteF16, FiniteF128] {
        $(
            let len = <$ty as Integer>::SafeLen::one();
            let a = $ty::new(42.0);
            assert_eq!($ty::safe_len_to_f64_lossy(len), 1.0);
            assert_eq!($ty::inclusive_end_from_start(a,len), a);
            assert_eq!($ty::start_from_inclusive_end(a,len), a);
            assert_eq!($ty::f64_to_safe_len_lossy(1.0), len);

        )*
    }};
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
// I don't quite understand why clippy complains here
#[expect(clippy::from_iter_instead_of_collect)]
fn float_test() {
    let _ = RangeSetBlaze::<TotalF64>::new();
    let _ = RangeSetBlaze::<TotalF32>::new();

    let _ = RangeSetBlaze::from_iter([tf64(3.0)..=tf64(5.0)]);
    let _ = RangeSetBlaze::from_iter([tf32(3.0)..=tf32(5.0)]);
    let _ = RangeSetBlaze::from_iter([tf64(1.0), tf64(2.0), tf64(3.0)]);
    let _ = RangeSetBlaze::from_iter([tf32(1.0), tf32(2.0), tf32(3.0)]);

    let _ = RangeSetBlaze::from(tf64(3.0)..=tf64(5.0));
    let _ = RangeSetBlaze::from(tf32(3.0)..=tf32(5.0));

    let _ = RangeSetBlaze::from(TotalF64::from_primitive_range(3.0..=5.0));
    let _ = RangeSetBlaze::from(TotalF32::from_primitive_range(3.0..=5.0));

    let _ = RangeSetBlaze::from_iter(TotalF64::from_primitive_ranges([3.0..=5.0, 7.0..=9.0]));
    let _ = RangeSetBlaze::from_iter(TotalF32::from_primitive_ranges([3.0..=5.0, 7.0..=9.0]));

    let _ = RangeSetBlaze::from_iter(TotalF64::from_primitive_slice(&[1.0, 2.0, 3.0]));
    let _ = RangeSetBlaze::from_iter(TotalF32::from_primitive_slice(&[1.0, 2.0, 3.0]));
    let _ = RangeSetBlaze::from_iter(TotalF64::values([1.0, 2.0, 3.0]));
    let _ = RangeSetBlaze::from_iter(TotalF32::values([1.0, 2.0, 3.0]));

    let foo = RangeSetBlaze::from_iter(TotalF64::from_primitive_ranges([3.0..=5.0, 7.0..=9.0]));
    assert!(foo.contains(tf64(3.0)));
    assert!(foo.contains(tf64(5.0)));
    assert!(foo.contains(tf64(7.0)));
    assert!(foo.contains(tf64(9.0)));

    assert!(foo.contains(tf64(3.01)));
    assert!(foo.contains(tf64(4.99)));
    assert!(foo.contains(tf64(7.01)));
    assert!(foo.contains(tf64(8.99)));

    assert!(!foo.contains(tf64(2.99)));
    assert!(!foo.contains(tf64(5.01)));
    assert!(!foo.contains(tf64(6.99)));
    assert!(!foo.contains(tf64(9.01)));

    assert!(!foo.contains(tf64(3.0).before()));
    assert!(!foo.contains(tf64(5.0).after()));
    assert!(!foo.contains(tf64(7.0).before()));
    assert!(!foo.contains(tf64(9.0).after()));

    assert!(foo.contains(tf64(3.0).after()));
    assert!(foo.contains(tf64(5.0).before()));
    assert!(foo.contains(tf64(7.0).after()));
    assert!(foo.contains(tf64(9.0).before()));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_inclusive() {
    syntactic_for! { ty in [TotalF32, TotalF64, FiniteF32, FiniteF64] {
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn test_inclusive_nightly() {
    syntactic_for! { ty in [TotalF16, TotalF128, FiniteF16, FiniteF128] {
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn inclusive_endpoints_cross_zero() {
    syntactic_for! { ty in [TotalF32, TotalF64, FiniteF32, FiniteF64] {
        $(
            let start = <$ty>::new(-1.0);
            let end = <$ty>::new(1.0);
            let len = <$ty>::safe_len(&(start..=end));
            assert_eq!(<$ty>::inclusive_end_from_start(start, len), end);
            assert_eq!(<$ty>::start_from_inclusive_end(end, len), start);
        )*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn inclusive_endpoints_cross_zero_nightly() {
    syntactic_for! { ty in [TotalF16, TotalF128, FiniteF16, FiniteF128] {
        $(
            let start = <$ty>::new(-1.0);
            let end = <$ty>::new(1.0);
            let len = <$ty>::safe_len(&(start..=end));
            assert_eq!(<$ty>::inclusive_end_from_start(start, len), end);
            assert_eq!(<$ty>::start_from_inclusive_end(end, len), start);
        )*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_floats2() {
    syntactic_for! { ty in [TotalF32, TotalF64, FiniteF32, FiniteF64] {
        $(
            let mut a = $ty::from_primitive_range(0.0..=0.0);
            assert_eq!($ty::range_next_back(&mut a), Some($ty::new(0.0)));
            assert_eq!($ty::range_next(&mut a), None);

            let mut b = $ty::new(0.0);
            $ty::assign_sub_one(&mut b);
            assert_eq!(b, $ty::new(0.0).before());
        )*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn test_floats2_nightly() {
    syntactic_for! { ty in [TotalF16, TotalF128, FiniteF16, FiniteF128] {
        $(
            let mut a = $ty::from_primitive_range(0.0..=0.0);
            assert_eq!($ty::range_next_back(&mut a), Some($ty::new(0.0)));
            assert_eq!($ty::range_next(&mut a), None);

            let mut b = $ty::new(0.0);
            $ty::assign_sub_one(&mut b);
            assert_eq!(b, $ty::new(0.0).before());
        )*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn total_iterators() {
    syntactic_for! { ty in [TotalF32, TotalF64, FiniteF32, FiniteF64] {
        $(
            // MAX forward
            let set = RangeSetBlaze::from_iter([$ty::MAX..=$ty::MAX]);
            let mut iter = set.iter();
            assert_eq!(iter.next(), Some($ty::MAX));
            assert_eq!(iter.next(), None);

            // MAX reverse
            let set = RangeSetBlaze::from_iter([$ty::MAX..=$ty::MAX]);
            let mut iter = set.iter().rev();
            assert_eq!(iter.next(), Some($ty::MAX));
            assert_eq!(iter.next(), None);

            // MIN forward
            let set = RangeSetBlaze::from_iter([$ty::MIN..=$ty::MIN]);
            let mut iter = set.iter();
            assert_eq!(iter.next(), Some($ty::MIN));
            assert_eq!(iter.next(), None);

            // MIN reverse
            let set = RangeSetBlaze::from_iter([$ty::MIN..=$ty::MIN]);
            let mut iter = set.iter().rev();
            assert_eq!(iter.next(), Some($ty::MIN));
            assert_eq!(iter.next(), None);
        )*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn total_iterators_nightly() {
    syntactic_for! { ty in [TotalF16, TotalF128, FiniteF16, FiniteF128] {
        $(
            // MAX forward
            let set = RangeSetBlaze::from_iter([$ty::MAX..=$ty::MAX]);
            let mut iter = set.iter();
            assert_eq!(iter.next(), Some($ty::MAX));
            assert_eq!(iter.next(), None);

            // MAX reverse
            let set = RangeSetBlaze::from_iter([$ty::MAX..=$ty::MAX]);
            let mut iter = set.iter().rev();
            assert_eq!(iter.next(), Some($ty::MAX));
            assert_eq!(iter.next(), None);

            // MIN forward
            let set = RangeSetBlaze::from_iter([$ty::MIN..=$ty::MIN]);
            let mut iter = set.iter();
            assert_eq!(iter.next(), Some($ty::MIN));
            assert_eq!(iter.next(), None);

            // MIN reverse
            let set = RangeSetBlaze::from_iter([$ty::MIN..=$ty::MIN]);
            let mut iter = set.iter().rev();
            assert_eq!(iter.next(), Some($ty::MIN));
            assert_eq!(iter.next(), None);
        )*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn total_complement() {
    syntactic_for! { ty in [TotalF32, TotalF64, FiniteF32, FiniteF64] {
        $(
            let set = RangeSetBlaze::from_iter([$ty::MAX..=$ty::MAX]);
            assert!(set.contains($ty::MAX));
            assert!(!set.contains($ty::MAX.before()));
            assert_eq!(set.len(), 1);
            let set = !set;
            assert!(!set.contains($ty::MAX));
            assert!(set.contains($ty::MAX.before()));
            assert_eq!(set.len(), $ty::MAX_SIZE - 1);

            let set = RangeSetBlaze::from_iter([$ty::MIN..=$ty::MIN]);
            assert!(set.contains($ty::MIN));
            assert!(!set.contains($ty::MIN.after()));
            assert_eq!(set.len(), 1);
            let set = !set;
            assert!(!set.contains($ty::MIN));
            assert!(set.contains($ty::MIN.after()));
            assert_eq!(set.len(), $ty::MAX_SIZE - 1);

            let set = RangeSetBlaze::from_iter([$ty::MIN..=$ty::MIN.after()]);
            assert!(set.contains($ty::MIN));
            assert!(set.contains($ty::MIN.after()));
            assert!(!set.contains($ty::MIN.after().after()));
            assert_eq!(set.len(), 2);
            let set = !set;
            assert!(!set.contains($ty::MIN));
            assert!(!set.contains($ty::MIN.after()));
            assert!(set.contains($ty::MIN.after().after()));
            assert_eq!(set.len(), $ty::MAX_SIZE - 2);
        )*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn total_complement_nightly() {
    // Total128::SafeLen is UIntPlusOne, which doesn't work smoothly here, so it's a separate test below
    syntactic_for! { ty in [TotalF16, /*TotalF128,*/ FiniteF16, FiniteF128] {
        $(
            let set = RangeSetBlaze::from_iter([$ty::MAX..=$ty::MAX]);
            assert!(set.contains($ty::MAX));
            assert!(!set.contains($ty::MAX.before()));
            assert_eq!(set.len(), 1);
            let set = !set;
            assert!(!set.contains($ty::MAX));
            assert!(set.contains($ty::MAX.before()));
            assert_eq!(set.len(), $ty::MAX_SIZE - 1);

            let set = RangeSetBlaze::from_iter([$ty::MIN..=$ty::MIN]);
            assert!(set.contains($ty::MIN));
            assert!(!set.contains($ty::MIN.after()));
            assert_eq!(set.len(), 1);
            let set = !set;
            assert!(!set.contains($ty::MIN));
            assert!(set.contains($ty::MIN.after()));
            assert_eq!(set.len(), $ty::MAX_SIZE - 1);

            let set = RangeSetBlaze::from_iter([$ty::MIN..=$ty::MIN.after()]);
            assert!(set.contains($ty::MIN));
            assert!(set.contains($ty::MIN.after()));
            assert!(!set.contains($ty::MIN.after().after()));
            assert_eq!(set.len(), 2);
            let set = !set;
            assert!(!set.contains($ty::MIN));
            assert!(!set.contains($ty::MIN.after()));
            assert!(set.contains($ty::MIN.after().after()));
            assert_eq!(set.len(), $ty::MAX_SIZE - 2);
        )*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn total_complement_total128() {
    let set = RangeSetBlaze::from_iter([TotalF128::MAX..=TotalF128::MAX]);
    assert!(set.contains(TotalF128::MAX));
    assert!(!set.contains(TotalF128::MAX.before()));
    assert_eq!(set.len(), UIntPlusOne::<u128>::UInt(1));
    let set = !set;
    assert!(!set.contains(TotalF128::MAX));
    assert!(set.contains(TotalF128::MAX.before()));
    assert_eq!(set.len(), UIntPlusOne::<u128>::UInt(u128::MAX));

    let set = RangeSetBlaze::from_iter([TotalF128::MIN..=TotalF128::MIN]);
    assert!(set.contains(TotalF128::MIN));
    assert!(!set.contains(TotalF128::MIN.after()));
    assert_eq!(set.len(), UIntPlusOne::<u128>::UInt(1));
    let set = !set;
    assert!(!set.contains(TotalF128::MIN));
    assert!(set.contains(TotalF128::MIN.after()));
    assert_eq!(set.len(), UIntPlusOne::<u128>::UInt(u128::MAX));

    let set = RangeSetBlaze::from_iter([TotalF128::MIN..=TotalF128::MIN.after()]);
    assert!(set.contains(TotalF128::MIN));
    assert!(set.contains(TotalF128::MIN.after()));
    assert!(!set.contains(TotalF128::MIN.after().after()));
    assert_eq!(set.len(), UIntPlusOne::<u128>::UInt(2));
    let set = !set;
    assert!(!set.contains(TotalF128::MIN));
    assert!(!set.contains(TotalF128::MIN.after()));
    assert!(set.contains(TotalF128::MIN.after().after()));
    assert_eq!(set.len(), UIntPlusOne::<u128>::UInt(u128::MAX - 1));
}

// ============================================================================
// Construction validation (see specs/finite-construction-validation.md).
// `FiniteF64`'s doc comment (see `FiniteF64` / `Finite<T>`) promises values
// "excluding NaN, -0.0, and infinities." `new()`/`try_new()` enforce that via
// `T::is_finite` + `T::normalize`; `range()`, `ranges()`, and `values()` now
// route through `new()` so they enforce the same contract. `from_primitive_slice()` validates
// (panicking on bad input) and delegates to the `unsafe` `from_primitive_slice_unchecked()`
// for the actual zero-copy view, which — like `new_unchecked()` — requires the
// caller to already guarantee the invariant.
// ============================================================================

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "Finite type requires a finite value")]
fn finite_range_rejects_nan_start() {
    let _ = FiniteF64::from_primitive_range(f64::NAN..=1.0);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "Finite type requires a finite value")]
fn finite_range_rejects_infinite_end() {
    let _ = FiniteF64::from_primitive_range(1.0..=f64::INFINITY);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "Finite type requires a finite value")]
fn finite_values_rejects_nan() {
    // Important: values() is lazy, so force iteration.
    let _ = FiniteF64::values([1.0, f64::NAN]).collect::<Vec<_>>();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn finite_values_normalizes_negative_zero() {
    let value = FiniteF64::values([-0.0])
        .next()
        .expect("one input produces one value");

    assert_eq!(value, FiniteF64::new(0.0));
    assert_eq!(value.into_inner().to_bits(), 0.0f64.to_bits());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "Finite type requires finite, non-negative-zero values")]
fn finite_slice_rejects_nan() {
    let _ = FiniteF64::from_primitive_slice(&[f64::NAN]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "Finite type requires finite, non-negative-zero values")]
fn finite_slice_rejects_negative_zero() {
    let _ = FiniteF64::from_primitive_slice(&[-0.0]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "Finite type requires finite, non-negative-zero values")]
fn finite_slice_rejects_infinity() {
    let _ = FiniteF64::from_primitive_slice(&[f64::INFINITY]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn finite_slice_unchecked_bypasses_validation() {
    // SAFETY: deliberately violates Finite's invariant to document exactly what
    // `from_primitive_slice_unchecked` allows through when its safety precondition is broken —
    // this is the documented "logic error, not UB" escape hatch from the spec.
    let values = unsafe { FiniteF64::from_primitive_slice_unchecked(&[f64::NAN]) };
    assert!(values[0].into_inner().is_nan());
}

// The validation logic in `range`/`values`/`slice` is shared generic code (one
// impl block in `finite.rs`, monomorphized per type) — the tests above
// already prove it works once. These f32/f128 variants aren't re-testing that
// shared logic; they sanity-check that each type's own `FiniteFloat::
// is_finite`/`normalize` leaf impl (which live separately per type in
// `finite_float.rs`) behaves the same way through it.

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "Finite type requires a finite value")]
fn finite_range_rejects_nan_start_f32() {
    let _ = FiniteF32::from_primitive_range(f32::NAN..=1.0);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn finite_values_normalizes_negative_zero_f32() {
    let value = FiniteF32::values([-0.0])
        .next()
        .expect("one input produces one value");

    assert_eq!(value, FiniteF32::new(0.0));
    assert_eq!(value.into_inner().to_bits(), 0.0f32.to_bits());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "Finite type requires finite, non-negative-zero values")]
fn finite_slice_rejects_nan_f32() {
    let _ = FiniteF32::from_primitive_slice(&[f32::NAN]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "Finite type requires finite, non-negative-zero values")]
fn finite_slice_rejects_infinity_f32() {
    let _ = FiniteF32::from_primitive_slice(&[f32::INFINITY]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "Finite type requires finite, non-negative-zero values")]
fn finite_slice_rejects_negative_zero_f32() {
    let _ = FiniteF32::from_primitive_slice(&[-0.0]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn finite_slice_unchecked_bypasses_validation_f32() {
    // SAFETY: deliberately violates Finite's invariant, see the f64 variant above.
    let values = unsafe { FiniteF32::from_primitive_slice_unchecked(&[f32::NAN]) };
    assert!(values[0].into_inner().is_nan());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
#[should_panic(expected = "Finite type requires a finite value")]
fn finite_range_rejects_nan_start_f128() {
    let _ = FiniteF128::from_primitive_range(f128::NAN..=1.0);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn finite_values_normalizes_negative_zero_f128() {
    let value = FiniteF128::values([-0.0]).next().unwrap();

    assert_eq!(value, FiniteF128::new(0.0));
    assert_eq!(value.into_inner().to_bits(), 0.0f128.to_bits());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
#[should_panic(expected = "Finite type requires finite, non-negative-zero values")]
fn finite_slice_rejects_nan_f128() {
    let _ = FiniteF128::from_primitive_slice(&[f128::NAN]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn finite_slice_unchecked_bypasses_validation_f128() {
    // SAFETY: deliberately violates Finite's invariant, see the f64 variant above.
    let values = unsafe { FiniteF128::from_primitive_slice_unchecked(&[f128::NAN]) };
    assert!(values[0].into_inner().is_nan());
}

// ============================================================================
// Why didn't the exhaustive `full_16` test above already catch this?
//
// `full_16` walks the *already-valid* total-order domain: it starts at
// `$ty::MIN` (a finite extreme) and steps forward only via `.after()`, whose
// own logic already knows to skip the -0.0 ordered slot and never produces
// NaN. It exhaustively checks internal self-consistency of that walk
// (safe_len, inclusive_end_from_start, after/before symmetry) — it never routes
// arbitrary/adversarial *input* bit patterns (NaN, -0.0, +/-infinity) through
// the other public entry points (`values`, `range`, `slice`). Below is the
// same exhaustive-over-all-bits idea as `full_16`, but aimed at the
// constructors: every valid bit pattern must round-trip cleanly through all
// three, -0.0 must normalize (or, for `slice`, panic, since it can't
// normalize a borrowed view), and every other invalid bit pattern must panic.
// ============================================================================

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn finite_f16_exhaustive_bit_patterns_via_constructors() {
    let mut failures = Vec::new();

    // Silence panic output for the (many) bit patterns we expect to panic below;
    // restore the previous hook before asserting so a real test failure still prints.
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    for bits in 0..=u16::MAX {
        let x = f16::from_bits(bits);
        let is_neg_zero = x == 0.0 && x.is_sign_negative();

        if x.is_finite() && !is_neg_zero {
            let via_values = FiniteF16::values([x]).next().unwrap();
            if via_values.into_inner() != x {
                failures.push(format!(
                    "values(): bits {bits:#06x} ({x}) did not round-trip, got {:?}",
                    via_values.into_inner()
                ));
            }

            let via_range = *FiniteF16::from_primitive_range(x..=x).start();
            if via_range.into_inner() != x {
                failures.push(format!(
                    "range(): bits {bits:#06x} ({x}) did not round-trip, got {:?}",
                    via_range.into_inner()
                ));
            }

            let via_slice = FiniteF16::from_primitive_slice(core::slice::from_ref(&x))[0];
            if via_slice.into_inner() != x {
                failures.push(format!(
                    "slice(): bits {bits:#06x} ({x}) did not round-trip, got {:?}",
                    via_slice.into_inner()
                ));
            }
        } else if is_neg_zero {
            let via_values = FiniteF16::values([x]).next().unwrap();
            if via_values.into_inner().is_sign_negative() {
                failures.push(format!("values(): bits {bits:#06x} did not normalize -0.0"));
            }

            let via_range = *FiniteF16::from_primitive_range(x..=x).start();
            if via_range.into_inner().is_sign_negative() {
                failures.push(format!("range(): bits {bits:#06x} did not normalize -0.0"));
            }

            if std::panic::catch_unwind(|| {
                FiniteF16::from_primitive_slice(core::slice::from_ref(&x))
            })
            .is_ok()
            {
                failures.push(format!(
                    "slice(): bits {bits:#06x} (-0.0) should have panicked, can't normalize a view"
                ));
            }
        } else {
            // NaN or +/-infinity: all three constructors must panic.
            if std::panic::catch_unwind(|| FiniteF16::values([x]).next()).is_ok() {
                failures.push(format!(
                    "values(): bits {bits:#06x} ({x}) should have panicked"
                ));
            }
            if std::panic::catch_unwind(|| FiniteF16::from_primitive_range(x..=x)).is_ok() {
                failures.push(format!(
                    "range(): bits {bits:#06x} ({x}) should have panicked"
                ));
            }
            if std::panic::catch_unwind(|| {
                FiniteF16::from_primitive_slice(core::slice::from_ref(&x))
            })
            .is_ok()
            {
                failures.push(format!(
                    "slice(): bits {bits:#06x} ({x}) should have panicked"
                ));
            }
        }
    }

    std::panic::set_hook(prev_hook);

    assert!(
        failures.is_empty(),
        "{} invariant violations found across all 65536 f16 bit patterns; first 10:\n{}",
        failures.len(),
        failures[..failures.len().min(10)].join("\n")
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "float_nightly_experimental")]
fn full_16() {
    syntactic_for! { ty in [TotalF16,  FiniteF16] {
        $(
            let mut x = $ty::MIN;
            let mut count : <$ty as Integer>::SafeLen = 0;
            loop {
                assert!($ty::MIN <= x && x <= $ty::MAX);
                assert_eq!($ty::safe_len(&($ty::MIN..=x)), count + 1);
                assert_eq!($ty::safe_len(&(x..=$ty::MAX)), $ty::MAX_SIZE - count );
                assert_eq!($ty::inclusive_end_from_start(x, $ty::MAX_SIZE - count), $ty::MAX);
                assert_eq!($ty::start_from_inclusive_end($ty::MAX, $ty::MAX_SIZE - count), x);
                assert_eq!($ty::inclusive_end_from_start($ty::MIN, count+1), x);
                assert_eq!($ty::start_from_inclusive_end(x, count+1), $ty::MIN);
                if x != $ty::MIN  {
                    assert_eq!(x.before().after(), x);
                    assert!(x.before() < x);
                    assert_eq!($ty::safe_len(&(x.before()..=x)), 2);
                    assert_eq!($ty::start_from_inclusive_end(x, 2), x.before());
                    assert!(x.before() < x);
                    assert!(x > x.before());
                    assert!(x == x);
                }
                if x != $ty::MAX {
                    assert_eq!(x.after().before(), x);
                    assert!(x.after() > x);
                    assert_eq!($ty::safe_len(&(x..=x.after())), 2);
                    assert_eq!($ty::inclusive_end_from_start(x,2), x.after());
                    assert!(x.after() > x);
                    assert!(x < x.after());
                }
                if x == $ty::MAX {
                    break;
                }
                x = x.after();
                count += 1;
            }
        )*
    }}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn tf64_categories() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Category {
        NaN,
        NegInfinity,
        PosInfinity,
        MinusZero,
        Normal,
    }

    // Build a category map using TotalF64's total_cmp order (NaN < -Inf < -0.0 < 0.0 < Inf < NaN).
    // Overlapping ranges have right-to-left precedence, so we go coarse-to-fine: start with
    // NaN everywhere, then carve out Normal, then the infinities and -0.0 as exceptions.
    let category_map = RangeMapBlaze::from_iter([
        (TotalF64::MIN..=TotalF64::MAX, Category::NaN), // everything else is NaN by default
        (
            tf64(f64::NEG_INFINITY)..=tf64(f64::INFINITY),
            Category::Normal,
        ), // everything between the infinities is Normal, inclusive (?!)
        (
            tf64(f64::NEG_INFINITY)..=tf64(f64::NEG_INFINITY),
            Category::NegInfinity,
        ), // carve out the infinities as exceptions
        (
            tf64(f64::INFINITY)..=tf64(f64::INFINITY),
            Category::PosInfinity,
        ), // carve out the infinities as exceptions
        (tf64(-0.0)..=tf64(-0.0), Category::MinusZero), // carve out -0.0 as an exception
    ]);

    for (range, category) in category_map.range_values() {
        let (start, end) = range.into_primitive_inner();
        println!(
            "{start:e} (0x{:016x}) ..= {end:e} (0x{:016x}) -> {category:?}",
            start.to_bits(),
            end.to_bits(),
        );
    }

    /* Output:
    NaN  (0xffffffffffffffff) ..= NaN  (0xfff0000000000001) -> NaN
    -inf (0xfff0000000000000) ..= -inf (0xfff0000000000000) -> NegInfinity
    -1.7976931348623157e308 (0xffefffffffffffff) ..= -5e-324 (0x8000000000000001) -> Normal
    -0e0 (0x8000000000000000) ..= -0e0 (0x8000000000000000) -> MinusZero
    0e0  (0x0000000000000000) ..= 1.7976931348623157e308 (0x7fefffffffffffff) -> Normal
    inf  (0x7ff0000000000000) ..= inf  (0x7ff0000000000000) -> PosInfinity
    NaN  (0x7ff0000000000001) ..= NaN  (0x7fffffffffffffff) -> NaN
    */

    // Spot-check a handful of values land where expected.
    assert_eq!(category_map.get(tf64(f64::NAN)), Some(&Category::NaN));
    assert_eq!(category_map.get(tf64(-f64::NAN)), Some(&Category::NaN));
    assert_eq!(
        category_map.get(tf64(f64::NEG_INFINITY)),
        Some(&Category::NegInfinity)
    );
    assert_eq!(
        category_map.get(tf64(f64::INFINITY)),
        Some(&Category::PosInfinity)
    );
    assert_eq!(category_map.get(tf64(-0.0)), Some(&Category::MinusZero));
    assert_eq!(category_map.get(tf64(0.0)), Some(&Category::Normal));
    assert_eq!(category_map.get(tf64(1.0)), Some(&Category::Normal));
    assert_eq!(
        category_map.get(tf64(f64::MIN_POSITIVE)),
        Some(&Category::Normal)
    );
    assert_eq!(category_map.get(tf64(-1.5e300)), Some(&Category::Normal));
}
