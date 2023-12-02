#![feature(portable_simd)]
#![feature(array_chunks)]

use std::simd::Simd;

const LANES: usize = if cfg!(simd_lanes = "2") {
    2
} else if cfg!(simd_lanes = "4") {
    4
} else if cfg!(simd_lanes = "8") {
    8
} else if cfg!(simd_lanes = "16") {
    16
} else if cfg!(simd_lanes = "32") {
    32
} else {
    64
};

#[macro_export]
macro_rules! define_is_consecutive_regular {
    ($function:ident, $type:ty) => {
        #[inline]
        pub fn $function(chunk: &[$type; LANES]) -> bool {
            for i in 1..LANES {
                if chunk[i - 1].checked_add(1) != Some(chunk[i]) {
                    return false;
                }
            }
            true
        }
    };
}

define_is_consecutive_regular!(is_consecutive_regular_i64, i64);

#[test]
fn test_regular() {
    let a: Vec<i64> = (100..100 + LANES as i64).collect();
    let ninety_nines: Vec<i64> = vec![99; LANES];
    let a = Simd::<i64, LANES>::from_slice(&a);
    let ninety_nines = Simd::<i64, LANES>::from_slice(ninety_nines.as_slice());

    assert!(is_consecutive_regular_i64(a.as_array()));
    assert!(!is_consecutive_regular_i64(ninety_nines.as_array()));
}

#[macro_export]
macro_rules! define_is_consecutive_splat0 {
    ($function:ident, $type:ty) => {
        #[inline]
        pub fn $function(chunk: Simd<$type, LANES>) -> bool {
            define_reference_splat0!(reference_splat0, $type);

            if chunk[0].checked_add(LANES as $type - 1) != Some(chunk[LANES - 1]) {
                return false;
            }
            let added = chunk + reference_splat0();
            Simd::splat(added[0]) == added
        }
    };
}
#[allow(unused_macros)]
macro_rules! define_reference_splat0 {
    ($function:ident, $type:ty) => {
        pub const fn $function() -> Simd<$type, LANES> {
            let mut arr: [$type; LANES] = [0; LANES];
            let mut i = 0;
            while i < LANES {
                arr[i] = (LANES - 1 - i) as $type;
                i += 1;
            }
            Simd::from_array(arr)
        }
    };
}

#[macro_export]
macro_rules! define_is_consecutive_splat1 {
    ($function:ident, $type:ty) => {
        #[inline]
        pub fn $function(chunk: Simd<$type, LANES>) -> bool {
            define_reference_splat!(reference_splat, $type);

            let subtracted = chunk - reference_splat();
            Simd::splat(chunk[0]) == subtracted
        }
    };
}

#[allow(unused_macros)]
macro_rules! define_reference_splat {
    ($function:ident, $type:ty) => {
        pub const fn $function() -> Simd<$type, LANES> {
            let mut arr: [$type; LANES] = [0; LANES];
            let mut i = 0;
            while i < LANES {
                arr[i] = i as $type;
                i += 1;
            }
            Simd::from_array(arr)
        }
    };
}

#[macro_export]
macro_rules! define_is_consecutive_splat2 {
    ($function:ident, $type:ty) => {
        #[inline]
        pub fn $function(chunk: Simd<$type, LANES>) -> bool {
            define_reference_splat!(reference_splat, $type);

            let subtracted = chunk - reference_splat();
            Simd::splat(subtracted[0]) == subtracted
        }
    };
}

#[macro_export]
macro_rules! define_is_consecutive_rotate {
    ($function:ident, $type:ty) => {
        #[inline]
        pub fn $function(chunk: Simd<$type, LANES>) -> bool {
            define_reference_rotate!(reference_rotate, $type);

            let rotated = chunk.rotate_lanes_right::<1>();
            chunk - rotated == reference_rotate()
        }
    };
}

#[allow(unused_macros)]
macro_rules! define_reference_rotate {
    ($function:ident, $type:ty) => {
        pub const fn $function() -> Simd<$type, LANES> {
            let mut arr: [$type; LANES] = [1; LANES];
            arr[0] = (1 as $type).wrapping_sub(LANES as $type);
            Simd::from_array(arr)
        }
    };
}

#[test]
fn test_is_consecutive() {
    pub type Integer = i16;

    define_is_consecutive_regular!(is_consecutive_regular, Integer);
    define_is_consecutive_splat0!(is_consecutive_splat0, Integer);
    define_is_consecutive_splat1!(is_consecutive_splat1, Integer);
    define_is_consecutive_splat2!(is_consecutive_splat2, Integer);
    define_is_consecutive_rotate!(is_consecutive_rotate, Integer);

    let a: Vec<Integer> = (100..100 + LANES as Integer).collect();
    let ninety_nines: Vec<Integer> = vec![99; LANES];
    let a = Simd::<Integer, LANES>::from_slice(&a);
    let ninety_nines = Simd::<Integer, LANES>::from_slice(ninety_nines.as_slice());

    assert!(is_consecutive_regular(a.as_array()));
    assert!(!is_consecutive_regular(ninety_nines.as_array()));

    assert!(is_consecutive_splat0(a));
    assert!(!is_consecutive_splat0(ninety_nines));

    assert!(is_consecutive_splat1(a));
    assert!(!is_consecutive_splat1(ninety_nines,));

    assert!(is_consecutive_splat2(a));
    assert!(!is_consecutive_splat2(ninety_nines,));

    assert!(is_consecutive_rotate(a));
    assert!(!is_consecutive_rotate(ninety_nines,));
}
