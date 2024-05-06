#![cfg(test)]
#![cfg(not(target_arch = "wasm32"))]

use super::*;
use core::cmp::Ordering;
use core::ops::Bound;
use core::ops::RangeInclusive;
use itertools::Itertools;
use num_traits::{One, Zero};
use quickcheck_macros::quickcheck;
use rand::{rngs::StdRng, SeedableRng};
use std::fmt::Debug;
use std::{
    any::Any,
    collections::{hash_map::DefaultHasher, BTreeSet},
    fmt::Display,
    hash::Hash,
    iter::FusedIterator,
    panic::{RefUnwindSafe, UnwindSafe},
}; // , time::Instant
   // use sorted_iter::assume::AssumeSortedByKeyExt;
   // use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

use syntactic_for::syntactic_for;
use tests_common::{How, MemorylessIter, MemorylessRange};
// use thousands::Separable;
use core::array;
use core::ops::BitAndAssign;
use rand::Rng;
type I32SafeLen = <i32 as crate::Integer>::SafeLen;

#[test]
fn insert_255u8() {
    let range_set_blaze = RangeSetBlaze::<u8>::from_iter([255]);
    assert!(range_set_blaze.to_string() == "255..=255");
}

#[test]
#[should_panic]
fn insert_max_u128() {
    let a = RangeSetBlaze::<u128>::from_iter([u128::MAX]);
    println!("a: {a}");
}

#[test]
fn sub() {
    for start in i8::MIN..i8::MAX {
        for end in start..i8::MAX {
            let diff = i8::safe_len(&(start..=end));
            let diff2 = (end as i16) - (start as i16) + 1;
            assert_eq!(diff as i16, diff2);
        }
    }
    for start in u8::MIN..u8::MAX {
        for end in start..u8::MAX {
            let diff = u8::safe_len(&(start..=end));
            let diff2 = (end as i16) - (start as i16) + 1;
            assert_eq!(diff as i16, diff2);
        }
    }

    let before = 127i8.overflowing_sub(-128i8).0;
    let after = before as u8;
    println!("before: {before}, after: {after}");
}

#[test]
fn complement0() {
    syntactic_for! { ty in [i8, u8 , isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
        let empty = RangeSetBlaze::<$ty>::new();
        let full = !&empty;
        println!("empty: {empty} (len {}), full: {full} (len {})", empty.len(), full.len());
        )*
    }};
}

#[test]
fn repro_bit_and() {
    let a = RangeSetBlaze::from_iter([1u8, 2, 3]);
    let b = RangeSetBlaze::from_iter([2u8, 3, 4]);

    let result = &a & &b;
    println!("{result}");
    assert_eq!(result, RangeSetBlaze::from_iter([2u8, 3]));
}

#[test]
fn doctest1() {
    let a = RangeSetBlaze::<u8>::from_iter([1, 2, 3]);
    let b = RangeSetBlaze::<u8>::from_iter([3, 4, 5]);

    let result = &a | &b;
    assert_eq!(result, RangeSetBlaze::<u8>::from_iter([1, 2, 3, 4, 5]));
}

#[test]
fn doctest2() {
    let set = RangeSetBlaze::<u8>::from_iter([1, 2, 3]);
    assert!(set.contains(1));
    assert!(!set.contains(4));
}

#[test]
fn doctest3() {
    let mut a = RangeSetBlaze::from_iter([1..=3]);
    let mut b = RangeSetBlaze::from_iter([3..=5]);

    a.append(&mut b);

    assert_eq!(a.len(), 5 as I32SafeLen);
    assert_eq!(b.len(), 0 as I32SafeLen);

    assert!(a.contains(1));
    assert!(a.contains(2));
    assert!(a.contains(3));
    assert!(a.contains(4));
    assert!(a.contains(5));
}

#[test]
fn doctest4() {
    let a = RangeSetBlaze::<i8>::from_iter([1, 2, 3]);

    let result = !&a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");
}

#[test]
fn compare() {
    let mut btree_set = BTreeSet::<u128>::new();
    btree_set.insert(3);
    btree_set.insert(1);
    let string = btree_set.iter().join(", ");
    println!("{string:#?}");
    assert!(string == "1, 3");
}

#[test]
fn demo_f1() {
    // before_or_equal_exists	0
    //     INSERT, etc

    let mut range_set_blaze = RangeSetBlaze::from_iter([11..=14, 22..=26]);
    range_set_blaze.internal_add(10..=10);
    assert!(range_set_blaze.to_string() == "10..=14, 22..=26");
    println!(
        "demo_1 range_set_blaze = {:?}, len_slow = {}, len = {}",
        range_set_blaze,
        range_set_blaze.len_slow(),
        range_set_blaze.len()
    );

    assert!(range_set_blaze.len_slow() == range_set_blaze.len());
}

#[test]
fn demo_d1() {
    // before_or_equal_exists	1
    // equal?	1
    // is_included	n/a
    // fits?	1
    //     DONE

    let mut range_set_blaze = RangeSetBlaze::from_iter([10..=14]);
    range_set_blaze.internal_add(10..=10);
    assert!(range_set_blaze.to_string() == "10..=14");
    assert!(range_set_blaze.len_slow() == range_set_blaze.len());
}

#[test]
fn demo_e1() {
    // before_or_equal_exists	1
    // equal?	1
    // is_included	n/a
    // fits?	0
    // next?    0
    //     DONE

    let mut range_set_blaze = RangeSetBlaze::from_iter([10..=14, 16..=16]);
    range_set_blaze.internal_add(10..=19);
    assert!(range_set_blaze.to_string() == "10..=19");
    assert!(range_set_blaze.len_slow() == range_set_blaze.len());
}

#[test]
fn demo_b1() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	0
    // next?    0
    //     DONE

    let mut range_set_blaze = RangeSetBlaze::from_iter([10..=14]);
    range_set_blaze.internal_add(12..=17);
    assert!(range_set_blaze.to_string() == "10..=17");
    assert!(range_set_blaze.len_slow() == range_set_blaze.len());
}

#[test]
fn demo_b2() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	0
    // next?    1
    // delete how many? 1
    //     DONE

    let mut range_set_blaze = RangeSetBlaze::from_iter([10..=14, 16..=16]);
    range_set_blaze.internal_add(12..=17);
    assert!(range_set_blaze.to_string() == "10..=17");
    assert!(range_set_blaze.len_slow() == range_set_blaze.len());
}

#[test]
fn add_in_order() {
    let mut range_set = RangeSetBlaze::new();
    for i in 0u64..1000 {
        range_set.insert(i);
    }
}

#[test]
fn optimize() {
    let end = 8u8;
    for a in 0..=end {
        for b in 0..=end {
            for c in 0..=end {
                for d in 0..=end {
                    let restart = (a >= 2 && a - 2 >= d) || (c >= 2 && c - 2 >= b);
                    print!("{a}\t{b}\t{c}\t{d}\t");
                    if a > b {
                        println!("impossible");
                    } else if c > d {
                        println!("error");
                    } else {
                        let mut range_set_blaze = RangeSetBlaze::new();
                        range_set_blaze.internal_add(a..=b);
                        range_set_blaze.internal_add(c..=d);
                        if range_set_blaze.ranges_len() == 1 {
                            let vec = range_set_blaze.into_iter().collect::<Vec<u8>>();
                            println! {"combine\t{}\t{}", vec[0], vec[vec.len()-1]};
                            assert!(!restart);
                        } else {
                            println!("restart");
                            assert!(restart);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn understand_into_iter() {
    let btree_set = BTreeSet::from([1, 2, 3, 4, 5]);
    for i in btree_set.iter() {
        println!("{i}");
    }

    let s = "abc".to_string();
    for c in s.chars() {
        println!("{c}");
    }
    println!("{s:?}");
    // println!("{btree_set:?}");

    // let ri = 1..=5;
    // let rii = ri.into_iter();
    // let val = rii.next();
    // let len = rii.len();
    // // for i in ri() {
    // //     println!("{i} {}", ri.len());
    // // }
    // // println!("{ri:?}");
    let s = "hello".to_string();
    let mut si = s.bytes();
    let _val = si.next();
    let _len = si.len();
    let _len2 = s.len();

    let arr = [1, 2, 3, 4, 5];
    for i in arr.iter() {
        println!("{i}");
    }

    for i in arr {
        println!("{i}");
    }

    // let rsi = RangeSetBlaze::from_iter(1..=5);
    // for i in rsi.iter() {
    //     println!("{i}");
    // }
    // let len = rsi.len();
}

#[derive(Debug, PartialEq)]
struct BooleanVector(Vec<bool>);

impl BitAndAssign for BooleanVector {
    // `rhs` is the "right-hand side" of the expression `a &= b`.
    fn bitand_assign(&mut self, rhs: Self) {
        assert_eq!(self.0.len(), rhs.0.len());
        *self = BooleanVector(
            self.0
                .iter()
                .zip(rhs.0.iter())
                .map(|(x, y)| *x && *y)
                .collect(),
        );
    }
}

#[test]
fn understand_bitand_assign() {
    let mut a = 3u8;
    let b = 5u8;
    a &= b;
    println!("{a}");
    println!("{b}");

    let mut bv = BooleanVector(vec![true, true, false, false]);
    let bv2 = BooleanVector(vec![true, false, true, false]);
    bv &= bv2;
    let expected = BooleanVector(vec![true, false, false, false]);
    assert_eq!(bv, expected);
    // println!("{bv2:?}");
}

#[test]
fn iters() {
    let range_set_blaze = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    assert!(range_set_blaze.len() == 13 as I32SafeLen);
    for i in range_set_blaze.iter() {
        println!("{i}");
    }
    for range in range_set_blaze.ranges() {
        println!("{range:?}");
    }
    let mut rs = range_set_blaze.ranges();
    println!("{:?}", rs.next());
    println!("{range_set_blaze}");
    println!("{:?}", rs.len());
    println!("{:?}", rs.next());
    for i in range_set_blaze.iter() {
        println!("{i}");
    }
    // range_set_blaze.len();

    let mut rs = !range_set_blaze.ranges();
    println!("{:?}", rs.next());
    println!("{range_set_blaze}");
    // !!! assert that can't use range_set_blaze again
}

#[test]
fn missing_doctest_ops() {
    // note that may be borrowed or owned in any combination.

    // Returns the union of `self` and `rhs` as a new [`RangeSetBlaze`].
    let a = RangeSetBlaze::from_iter([1, 2, 3]);
    let b = RangeSetBlaze::from_iter([3, 4, 5]);

    let result = &a | &b;
    assert_eq!(result, RangeSetBlaze::from_iter([1, 2, 3, 4, 5]));
    let result = a | &b;
    assert_eq!(result, RangeSetBlaze::from_iter([1, 2, 3, 4, 5]));

    // Returns the complement of `self` as a new [`RangeSetBlaze`].
    let a = RangeSetBlaze::<i8>::from_iter([1, 2, 3]);

    let result = !&a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");
    let result = !a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");

    // Returns the intersection of `self` and `rhs` as a new `RangeSetBlaze<T>`.

    let a = RangeSetBlaze::from_iter([1, 2, 3]);
    let b = RangeSetBlaze::from_iter([2, 3, 4]);

    let result = a & &b;
    assert_eq!(result, RangeSetBlaze::from_iter([2, 3]));
    let a = RangeSetBlaze::from_iter([1, 2, 3]);
    let result = a & b;
    assert_eq!(result, RangeSetBlaze::from_iter([2, 3]));

    // Returns the symmetric difference of `self` and `rhs` as a new `RangeSetBlaze<T>`.
    let a = RangeSetBlaze::from_iter([1, 2, 3]);
    let b = RangeSetBlaze::from_iter([2, 3, 4]);

    let result = a ^ b;
    assert_eq!(result, RangeSetBlaze::from_iter([1, 4]));

    // Returns the set difference of `self` and `rhs` as a new `RangeSetBlaze<T>`.
    let a = RangeSetBlaze::from_iter([1, 2, 3]);
    let b = RangeSetBlaze::from_iter([2, 3, 4]);

    let result = a - b;
    assert_eq!(result, RangeSetBlaze::from_iter([1]));
}

// https://stackoverflow.com/questions/21747136/how-do-i-print-in-rust-the-type-of-a-variable/58119924#58119924
// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>())
// }

#[test]
fn from_string() {
    let a = RangeSetBlaze::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=65535]);
    assert_eq!(a, RangeSetBlaze::from_iter([0..=65535]));
}

#[test]
fn nand_repro() {
    let b = &RangeSetBlaze::from_iter([5u8..=13, 18..=29]);
    let c = &RangeSetBlaze::from_iter([38..=42]);
    println!("about to nand");
    let d = !b | !c;
    assert_eq!(
        d,
        RangeSetBlaze::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=255])
    );
}

// cmk0
// #[test]
// fn bit_or_iter() {
//     let i = UnionIter::from([1, 3, 4, 2, 2, 43, -1, 4, 22]);
//     let j = UnionIter::from([11, 3, 4, 42, 2, 43, 23, 2, 543]);

//     let _not_i = !i.clone();
//     let k = i - j;
//     assert_eq!(k.into_string(), "-1..=-1, 1..=1, 22..=22");
// }

// cmk0
// #[test]
// fn empty() {
//     let universe: UnionIter<u8, _> = [0..=255].into_iter().collect();
//     let arr: [u8; 0] = [];
//     let a0 = RangeSetBlaze::<u8>::from_iter(arr);
//     assert!(!(a0.ranges()).equal(universe.clone()));
//     assert!((!a0).ranges().equal(universe));
//     let _a0 = RangeSetBlaze::from_iter([0..=0; 0]);
//     let _a = RangeSetBlaze::<i32>::new();

//     let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
//     let a = a_iter.collect::<RangeSetBlaze<i32>>();
//     let arr: [i32; 0] = [];
//     let b = RangeSetBlaze::from_iter(arr);
//     let mut c3 = a.clone();
//     let mut c5 = a.clone();

//     let c0 = (&a).bitor(&b);
//     let c1a = &a | &b;
//     let c1b = &a | b.clone();
//     let c1c = a.clone() | &b;
//     let c1d = a.clone() | b.clone();
//     let c2: RangeSetBlaze<_> = (a.ranges() | b.ranges()).into_range_set_blaze();
//     c3.append(&mut b.clone());
//     c5.extend(b);

//     let answer = RangeSetBlaze::from_iter(arr);
//     assert_eq!(&c0, &answer);
//     assert_eq!(&c1a, &answer);
//     assert_eq!(&c1b, &answer);
//     assert_eq!(&c1c, &answer);
//     assert_eq!(&c1d, &answer);
//     assert_eq!(&c2, &answer);
//     assert_eq!(&c3, &answer);
//     assert_eq!(&c5, &answer);

//     let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
//     let a = a_iter.collect::<RangeSetBlaze<i32>>();
//     let b = RangeSetBlaze::from_iter([0i32; 0]);

//     let c0 = a.ranges() | b.ranges();
//     let c1 = [a.ranges(), b.ranges()].union();
//     let c_list2: [RangesIter<i32>; 0] = [];
//     let c2 = c_list2.clone().union();
//     let c3 = union_dyn!(a.ranges(), b.ranges());
//     let c4 = c_list2.map(DynSortedDisjoint::new).union();

//     let answer = RangeSetBlaze::from_iter(arr);
//     assert!(c0.equal(answer.ranges()));
//     assert!(c1.equal(answer.ranges()));
//     assert!(c2.equal(answer.ranges()));
//     assert!(c3.equal(answer.ranges()));
//     assert!(c4.equal(answer.ranges()));

//     let c0 = !(a.ranges() & b.ranges());
//     let c1 = ![a.ranges(), b.ranges()].intersection();
//     let c_list2: [RangesIter<i32>; 0] = [];
//     let c2 = !!c_list2.clone().intersection();
//     let c3 = !intersection_dyn!(a.ranges(), b.ranges());
//     let c4 = !!c_list2.map(DynSortedDisjoint::new).intersection();

//     let answer = !RangeSetBlaze::from_iter([0i32; 0]);
//     assert!(c0.equal(answer.ranges()));
//     assert!(c1.equal(answer.ranges()));
//     assert!(c2.equal(answer.ranges()));
//     assert!(c3.equal(answer.ranges()));
//     assert!(c4.equal(answer.ranges()));
// }

// // Can't implement fmt::Display fmt must take ownership
// impl<T, I> UnsortedDisjoint<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T>>,
// {
//     #[allow(clippy::inherent_to_string)]
//     #[allow(clippy::wrong_self_convention)]
//     pub(crate) fn to_string(self) -> String {
//         self.map(|range| format!("{range:?}")).join(", ")
//     }
// }

// !!!cmk resort this test ???
// #[allow(clippy::reversed_empty_ranges)]
// #[test]
// fn private_constructor() {
//     let unsorted_disjoint = UnsortedDisjoint::from([5..=6, 1..=5, 1..=0, -12..=-10, 3..=3]);
//     // println!("{}", unsorted_disjoint.fmt());
//     assert_eq!(unsorted_disjoint.into_string(), "1..=6, -12..=-10, 3..=3");

//     let unsorted_disjoint = UnsortedDisjoint::from([5..=6, 1..=5, 1..=0, -12..=-10, 3..=3]);
//     let union_iter = UnionIter::from(unsorted_disjoint);
//     // println!("{}", union_iter.fmt());
//     assert_eq!(union_iter.into_string(), "-12..=-10, 1..=6");

//     let union_iter: UnionIter<_, _> = [5, 6, 1, 2, 3, 4, 5, -12, -11, -10, 3]
//         .into_iter()
//         .collect();
//     assert_eq!(union_iter.into_string(), "-12..=-10, 1..=6");
// }

fn is_ddcppdheo<
    T: std::fmt::Debug
        + Display
        + Clone
        + PartialEq
        + PartialOrd
        + Default
        + std::hash::Hash
        + Eq
        + Ord
        + Send
        + Sync,
>() {
}

fn is_sssu<T: Sized + Send + Sync + Unpin>() {}
fn is_like_btreeset_iter<T: Clone + std::fmt::Debug + FusedIterator + Iterator>() {}
// removed DoubleEndedIterator +ExactSizeIterator for now

// cmk0 add others iterators and test
// #[test]
// fn iter_traits() {
//     type ARangesIter<'a> = RangesIter<'a, i32>;
//     type AIter<'a> = Iter<i32, ARangesIter<'a>>;
//     is_sssu::<AIter>();
//     is_like_btreeset_iter::<AIter>();
// }

fn is_like_btreeset_into_iter<T: std::fmt::Debug + FusedIterator + Iterator>() {}

fn is_like_btreeset<
    T: Clone
        + std::fmt::Debug
        + Default
        + Eq
        + std::hash::Hash
        + IntoIterator
        + Ord
        + PartialEq
        + PartialOrd
        + RefUnwindSafe
        + Send
        + Sync
        + Unpin
        + UnwindSafe
        + Any
        + ToOwned,
>() {
}

fn is_like_check_sorted_disjoint<
    T: Clone
        + std::fmt::Debug
        + Default
        + IntoIterator
        + RefUnwindSafe
        + Send
        + Sync
        + Unpin
        + UnwindSafe
        + Any
        + ToOwned,
>() {
}

fn is_like_dyn_sorted_disjoint<T: IntoIterator + Unpin + Any>() {}

// !!!cmk0 test traits of other iterators
// #[test]
// fn check_traits() {
//     // Debug/Display/Clone/PartialEq/PartialOrd/Default/Hash/Eq/Ord/Send/Sync
//     type ARangeSetBlaze = RangeSetBlaze<i32>;
//     is_sssu::<ARangeSetBlaze>();
//     is_ddcppdheo::<ARangeSetBlaze>();
//     is_like_btreeset::<ARangeSetBlaze>();

//     type ARangesIter<'a> = RangesIter<'a, i32>;
//     is_sssu::<ARangesIter>();
//     is_like_btreeset_iter::<ARangesIter>();

//     type AIter<'a> = Iter<i32, ARangesIter<'a>>;
//     is_sssu::<AIter>();
//     is_like_btreeset_iter::<AIter>();

//     is_sssu::<IntoIter<i32>>();
//     is_like_btreeset_into_iter::<IntoIter<i32>>();

//     type AMerge<'a> = Merge<i32, ARangesIter<'a>, ARangesIter<'a>>;
//     is_sssu::<AMerge>();
//     is_like_btreeset_iter::<AMerge>();

//     let a = RangeSetBlaze::from_iter([1..=2, 3..=4]);
//     // println!("{:?}", a.ranges()); // cmk get this working again?

//     type AKMerge<'a> = KMerge<i32, ARangesIter<'a>>;
//     is_sssu::<AKMerge>();
//     is_like_btreeset_iter::<AKMerge>();

//     type ANotIter<'a> = NotIter<i32, ARangesIter<'a>>;
//     is_sssu::<ANotIter>();
//     is_like_btreeset_iter::<ANotIter>();

//     type AIntoRangesIter<'a> = IntoRangesIter<i32>;
//     is_sssu::<AIntoRangesIter>();
//     is_like_btreeset_into_iter::<AIntoRangesIter>();

//     type ACheckSortedDisjoint<'a> = CheckSortedDisjoint<i32, ARangesIter<'a>>;
//     is_sssu::<ACheckSortedDisjoint>();
//     type BCheckSortedDisjoint =
//         CheckSortedDisjoint<i32, std::array::IntoIter<RangeInclusive<i32>, 0>>;
//     is_like_check_sorted_disjoint::<BCheckSortedDisjoint>();

//     type ADynSortedDisjoint<'a> = DynSortedDisjoint<'a, i32>;
//     is_like_dyn_sorted_disjoint::<ADynSortedDisjoint>();

//     type AUnionIter<'a> = UnionIter<i32, ARangesIter<'a>>;
//     is_sssu::<AUnionIter>();
//     is_like_btreeset_iter::<AUnionIter>();

//     type AAssumeSortedStarts<'a> = AssumeSortedStarts<i32, ARangesIter<'a>>;
//     is_sssu::<AAssumeSortedStarts>();
//     is_like_btreeset_iter::<AAssumeSortedStarts>();
// }

#[test]
fn integer_coverage() {
    syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
            let len = <$ty as Integer>::SafeLen::one();
            let a = $ty::zero();
            assert_eq!($ty::safe_len_to_f64(len), 1.0);
            assert_eq!($ty::add_len_less_one(a,len), a);
            assert_eq!($ty::sub_len_less_one(a,len), a);
            assert_eq!($ty::f64_to_safe_len(1.0), len);
            assert!($ty::safe_max_value()<=$ty::max_value());
            assert!(<$ty as Integer>::safe_max_value()<=$ty::max_value());

        )*
    }};
}

#[test]
#[allow(clippy::bool_assert_comparison)]
fn lib_coverage_0() {
    let a = RangeSetBlaze::from_iter([1..=2, 3..=4]);
    let mut hasher = DefaultHasher::new();
    a.hash(&mut hasher);
    let _d = RangeSetBlaze::<i32>::default();
    assert_eq!(a, a);

    let mut set = RangeSetBlaze::new();
    assert_eq!(set.first(), None);
    set.insert(1);
    assert_eq!(set.first(), Some(1));
    set.insert(2);
    assert_eq!(set.first(), Some(1));

    let set = RangeSetBlaze::from_iter([1, 2, 3]);
    assert_eq!(set.get(2), Some(2));
    assert_eq!(set.get(4), None);

    let mut set = RangeSetBlaze::new();
    assert_eq!(set.last(), None);
    set.insert(1);
    assert_eq!(set.last(), Some(1));
    set.insert(2);
    assert_eq!(set.last(), Some(2));

    assert_eq!(a.len(), a.len_slow());

    let mut a = RangeSetBlaze::from_iter([1..=3]);
    let mut b = RangeSetBlaze::from_iter([3..=5]);

    a.append(&mut b);

    assert_eq!(a.len(), 5 as I32SafeLen);
    assert_eq!(b.len(), 0 as I32SafeLen);

    assert!(a.contains(1));
    assert!(a.contains(2));
    assert!(a.contains(3));
    assert!(a.contains(4));
    assert!(a.contains(5));

    let mut v = RangeSetBlaze::new();
    v.insert(1);
    v.clear();
    assert!(v.is_empty());

    let mut v = RangeSetBlaze::new();
    assert!(v.is_empty());
    v.insert(1);
    assert!(!v.is_empty());

    let sup = RangeSetBlaze::from_iter([1..=3]);
    let mut set = RangeSetBlaze::new();

    assert_eq!(set.is_subset(&sup), true);
    set.insert(2);
    assert_eq!(set.is_subset(&sup), true);
    set.insert(4);
    assert_eq!(set.is_subset(&sup), false);

    let sub = RangeSetBlaze::from_iter([1, 2]);
    let mut set = RangeSetBlaze::new();

    assert_eq!(set.is_superset(&sub), false);

    set.insert(0);
    set.insert(1);
    assert_eq!(set.is_superset(&sub), false);

    set.insert(2);
    assert_eq!(set.is_superset(&sub), true);

    let a = RangeSetBlaze::from_iter([1..=3]);
    let mut b = RangeSetBlaze::new();

    assert_eq!(a.is_disjoint(&b), true);
    b.insert(4);
    assert_eq!(a.is_disjoint(&b), true);
    b.insert(1);
    assert_eq!(a.is_disjoint(&b), false);

    let mut set = RangeSetBlaze::new();
    set.insert(3);
    set.insert(5);
    set.insert(8);
    assert_eq!(Some(5), set.range(4..).next());
    assert_eq!(Some(3), set.range(..).next());
    assert_eq!(None, set.range(..=2).next());
    assert_eq!(None, set.range(1..2).next());
    assert_eq!(
        Some(3),
        set.range((Bound::Excluded(2), Bound::Excluded(4))).next()
    );

    let mut set = RangeSetBlaze::new();

    assert_eq!(set.ranges_insert(2..=5), true);
    assert_eq!(set.ranges_insert(5..=6), true);
    assert_eq!(set.ranges_insert(3..=4), false);
    assert_eq!(set.len(), 5 as I32SafeLen);
    let mut set = RangeSetBlaze::from_iter([1, 2, 3]);
    assert_eq!(set.take(2), Some(2));
    assert_eq!(set.take(2), None);

    let mut set = RangeSetBlaze::new();
    assert!(set.replace(5).is_none());
    assert!(set.replace(5).is_some());

    let mut a = RangeSetBlaze::from_iter([1..=3]);
    #[allow(clippy::reversed_empty_ranges)]
    a.internal_add(2..=1);

    assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));

    let mut a = RangeSetBlaze::from_iter([1..=3]);
    a.extend(std::iter::once(4));
    assert_eq!(a.len(), 4 as I32SafeLen);

    let mut a = RangeSetBlaze::from_iter([1..=3]);
    a.extend(4..=5);
    assert_eq!(a.len(), 5 as I32SafeLen);

    let mut set = RangeSetBlaze::new();

    set.insert(1);
    while let Some(n) = set.pop_first() {
        assert_eq!(n, 1);
    }
    assert!(set.is_empty());

    let mut set = RangeSetBlaze::new();

    set.insert(1);
    while let Some(n) = set.pop_last() {
        assert_eq!(n, 1);
    }
    assert!(set.is_empty());

    let a = RangeSetBlaze::from_iter([1..=3]);
    let i = a.iter();
    let j = i.clone();
    assert_eq!(i.size_hint(), j.size_hint());

    let a = RangeSetBlaze::from_iter([1..=3]);
    let _i = a.into_iter();
    // cmk0 get this assert working again
    // assert_eq!(i.size_hint(), j.size_hint());
    // assert_eq!(
    //     format!("{:?}", &i),
    //     "IntoIter { option_range_front: None, option_range_back: None, into_iter: [(1, 3)] }"
    // );

    let mut a = RangeSetBlaze::from_iter([1..=3]);
    a.extend([1..=3]);
    assert_eq!(a.len(), 3 as I32SafeLen);

    let a = RangeSetBlaze::from_iter([1..=3]);
    let b = <RangeSetBlaze<i32> as Clone>::clone(&a);
    assert_eq!(a, b);
    let c = <RangeSetBlaze<i32> as Default>::default();
    assert_eq!(c, RangeSetBlaze::new());

    syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
            let a = RangeSetBlaze::<$ty>::new();
            println!("{a:#?}");
            assert_eq!(a.iter().next(), None);

            let mut a = RangeSetBlaze::from_iter([$ty::one()..=3]);
            let mut b = RangeSetBlaze::from_iter([3..=5]);

            a.append(&mut b);

            // assert_eq!(a.len(), 5usize);
            assert_eq!(b.len(), <$ty as Integer>::SafeLen::zero());

            assert!(a.contains(1));
            assert!(a.contains(2));
            assert!(a.contains(3));
            assert!(a.contains(4));
            assert!(a.contains(5));

            assert!(b.is_empty());

            let a = RangeSetBlaze::from_iter([$ty::one()..=3]);
            let b = RangeSetBlaze::from_iter([3..=5]);
            assert!(!a.is_subset(&b));
            assert!(!a.is_superset(&b));

        )*
    }};

    let a = RangeSetBlaze::from_iter([1u128..=3]);
    assert!(a.contains(1));
    assert!(!a.is_disjoint(&a));
}

#[test]
#[should_panic]
fn lib_coverage_2() {
    let v = RangeSetBlaze::<u128>::new();
    v.contains(u128::MAX);
}

#[test]
#[should_panic]
fn lib_coverage_3() {
    let mut v = RangeSetBlaze::<u128>::new();
    v.remove(u128::MAX);
}

#[test]
#[should_panic]
fn lib_coverage_4() {
    let mut v = RangeSetBlaze::<u128>::new();
    v.split_off(u128::MAX);
}

#[test]
#[should_panic]
fn lib_coverage_5() {
    let mut v = RangeSetBlaze::<u128>::new();
    v.internal_add(0..=u128::MAX);
}

#[test]
fn lib_coverage_6() {
    syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
            let mut a = RangeSetBlaze::<$ty>::from_iter([1..=3, 5..=7, 9..=120]);
            a.ranges_insert(2..=100);
            assert_eq!(a, RangeSetBlaze::from_iter([1..=120]));


        )*
    }};
}

// cmk0
// #[test]
// fn merge_coverage_0() {
//     let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
//     let b = CheckSortedDisjoint::new([2..=6]);
//     let m = Merge::new(a, b);
//     let n = m.clone();
//     let p = n.clone();
//     let union1 = UnionIter::new(m);
//     let union2 = UnionIter::new(n);
//     assert!(union1.equal(union2));
//     assert!(format!("{p:?}").starts_with("Merge"));

//     let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
//     let b = CheckSortedDisjoint::new([2..=6]);
//     let c = CheckSortedDisjoint::new([-1..=-1]);
//     let m = KMerge::new([a, b, c]);
//     let n = m.clone();
//     let p = n.clone();
//     let union1 = UnionIter::new(m);
//     let union2 = UnionIter::new(n);
//     assert!(union1.equal(union2));
//     assert!(format!("{p:?}").starts_with("KMerge"));
// }

#[test]
fn not_iter_coverage_0() {
    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    let n = NotIter::new(a);
    let p = n.clone();
    let m = p.clone();
    assert!(n.equal(m));
    assert!(format!("{p:?}").starts_with("NotIter"));
}

// cmk0 get working again
// #[test]
// fn ranges_coverage_0() {
//     let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
//     let r = a.ranges();
//     // let p = r.as_ref();
//     // assert!(format!("{p:?}").starts_with("Ranges"));
//     assert_eq!(r.len(), 2);

//     let r2 = a.into_ranges();
//     let n2 = !!r2;
//     let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
//     assert!(n2.equal(a.ranges()));
//     let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
//     let b = a.into_ranges();
//     let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
//     let c = a.into_ranges();
//     let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
//     assert!((b | c).equal(a.ranges()));

//     let a = RangeSetBlaze::from_iter([1..=2, 5..=100]).into_ranges();
//     let b = RangeSetBlaze::from_iter([1..=2, 5..=100]).into_ranges();
//     assert!((a - b).is_empty());

//     let a = RangeSetBlaze::from_iter([1..=2, 5..=100]).into_ranges();
//     let b = RangeSetBlaze::from_iter([1..=2, 5..=100]).into_ranges();
//     assert!((a ^ b).is_empty());

//     let a = RangeSetBlaze::from_iter([1..=2, 5..=100]).into_ranges();
//     let b = RangeSetBlaze::from_iter([1..=2, 5..=100]).into_ranges();
//     assert!((a & b).equal(RangeSetBlaze::from_iter([1..=2, 5..=100]).into_ranges()));

//     assert_eq!(
//         RangeSetBlaze::from_iter([1..=2, 5..=100])
//             .into_ranges()
//             .len(),
//         2
//     );
//     assert!(format!(
//         "{:?}",
//         RangeSetBlaze::from_iter([1..=2, 5..=100]).into_ranges()
//     )
//     .starts_with("IntoRanges"));
// }

#[test]
fn sorted_disjoint_coverage_0() {
    let a = CheckSortedDisjoint::<i32, _>::default();
    assert!(a.is_empty());

    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    let b = CheckSortedDisjoint::new([1..=2, 5..=100]);
    assert!((a & b).equal(CheckSortedDisjoint::new([1..=2, 5..=100])));

    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    let b = CheckSortedDisjoint::new([1..=2, 5..=100]);
    assert!((a - b).is_empty());

    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    let b = CheckSortedDisjoint::new([1..=2, 5..=100]);
    assert!((a ^ b).is_empty());
}

#[test]
#[should_panic]
fn sorted_disjoint_coverage_1() {
    struct SomeAfterNone {
        a: i32,
    }
    impl FusedIterator for SomeAfterNone {}
    impl Iterator for SomeAfterNone {
        type Item = RangeInclusive<i32>;
        fn next(&mut self) -> Option<Self::Item> {
            self.a += 1;
            if self.a % 2 == 0 {
                Some(self.a..=self.a)
            } else {
                None
            }
        }
    }

    let mut a = CheckSortedDisjoint::new(SomeAfterNone { a: 0 });
    a.next();
    a.next();
    a.next();
}

#[test]
#[should_panic]
fn sorted_disjoint_coverage_2() {
    #[allow(clippy::reversed_empty_ranges)]
    let mut a = CheckSortedDisjoint::new([1..=0]);
    a.next();
}

#[test]
#[should_panic]
fn sorted_disjoint_coverage_3() {
    #[allow(clippy::reversed_empty_ranges)]
    let mut a = CheckSortedDisjoint::new([1..=1, 2..=2]);
    a.next();
    a.next();
}

#[test]
#[should_panic]
fn sorted_disjoint_coverage_4() {
    #[allow(clippy::reversed_empty_ranges)]
    let mut a = CheckSortedDisjoint::new([0..=i128::MAX]);
    a.next();
}

#[test]
fn sorted_disjoint_iterator_coverage_0() {
    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    let b = CheckSortedDisjoint::new([1..=2, 5..=101]);
    assert!(b.is_superset(a));
}

// cmk00
// #[test]
// fn union_iter_coverage_0() {
//     let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
//     let b = CheckSortedDisjoint::new([1..=2, 5..=101]);
//     let c = a.union(b);
//     assert!(format!("{c:?}").starts_with("UnionIter"));
// }

// cmk000
// #[test]
// fn unsorted_disjoint_coverage_0() {
//     let a = AssumeSortedStarts::new([1..=2, 5..=100]);
//     assert!(format!("{a:?}").starts_with("AssumeSortedStarts"));
// }

#[test]
fn test_coverage_0() {
    let a = BooleanVector(vec![true, true, false, false]);
    assert!(format!("{a:?}").starts_with("BooleanVector"));

    let a = How::Union;
    #[allow(clippy::clone_on_copy)]
    let _b = a.clone();

    let mut rng = StdRng::seed_from_u64(0);
    let a = MemorylessRange::new(&mut rng, 1000, 0..=10, 0.5, 1, How::Intersection);
    let v: Vec<_> = a.take(100).collect();
    println!("{v:?}");

    let a = MemorylessIter::new(&mut rng, 1000, 0..=10, 0.5, 1, How::Intersection);
    let v: Vec<_> = a.take(100).collect();
    println!("{v:?}");
}

type Element = i64;
type Reference = std::collections::BTreeSet<Element>;

#[quickcheck]
fn disjoint(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    a.is_disjoint(&b) == a_r.is_disjoint(&b_r)
}

#[quickcheck]
fn subset(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    a.is_subset(&b) == a_r.is_subset(&b_r)
}

#[quickcheck]
fn superset(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    a.is_superset(&b) == a_r.is_superset(&b_r)
}

/// just a helper to get good output when a check fails
fn binary_op<E: Debug, R: Eq + Debug>(a: E, b: E, expected: R, actual: R) -> bool {
    let res = expected == actual;
    if !res {
        println!("a:{a:?} b:{b:?} expected:{expected:?} actual:{actual:?}");
    }
    res
}

/// from: https://github.com/rklaehn/sorted-iter
/// just a helper to get good output when a check fails
fn check_size_hint<E: Debug>(
    input: E,
    expected: usize,
    (min, max): (usize, Option<usize>),
) -> bool {
    let res = min <= expected && max.map_or(true, |max| expected <= max && min <= max);
    if !res {
        println!("input:{input:?} expected:{expected:?} min:{min:?} max:{max:?}");
    }
    res
}

#[quickcheck]
fn intersection(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let expected: Reference = a.intersection(&b).cloned().collect();
    let actual: Reference = (a_r & b_r).into_iter().collect();
    binary_op(a, b, expected, actual)
}

#[quickcheck]
fn union(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let expected: Reference = a.union(&b).cloned().collect();
    let actual: Reference = (a_r | b_r).into_iter().collect();
    binary_op(a, b, expected, actual)
}

#[quickcheck]
fn multi_union(inputs: Vec<Reference>) -> bool {
    let expected: Reference = inputs.iter().flatten().copied().collect();
    let actual = inputs.iter().map(RangeSetBlaze::from_iter).union();

    let res = actual.iter().eq(expected.iter().cloned());
    if !res {
        let actual: Reference = actual.iter().collect();
        println!("in:{inputs:?} expected:{expected:?} out:{actual:?}");
    }
    res
}

#[quickcheck]
fn difference(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let expected: Reference = a.difference(&b).cloned().collect();
    let actual: Reference = (a_r - b_r).into_iter().collect();
    binary_op(a, b, expected, actual)
}

#[quickcheck]
fn symmetric_difference(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let expected: Reference = a.symmetric_difference(&b).cloned().collect();
    let actual: Reference = (a_r ^ b_r).into_iter().collect();
    binary_op(a, b, expected, actual)
}

#[quickcheck]
fn multi_symmetric_difference(inputs: Vec<Reference>) -> bool {
    let mut expected: Reference = BTreeSet::new();
    for input in inputs.iter() {
        expected = expected.symmetric_difference(input).cloned().collect();
    }
    let actual = inputs
        .iter()
        .map(RangeSetBlaze::from_iter)
        .symmetric_difference();

    let res = actual.iter().eq(expected.iter().cloned());
    if !res {
        let actual: Reference = actual.iter().collect();
        println!("in:{inputs:?} expected:{expected:?} out:{actual:?}");
    }
    res
}

#[quickcheck]
fn intersection_size_hint(a: Reference, b: Reference) -> bool {
    let expected = a.intersection(&b).count();
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let actual = (a_r & b_r).into_iter().size_hint();
    check_size_hint((a, b), expected, actual)
}

#[quickcheck]
fn union_size_hint(a: Reference, b: Reference) -> bool {
    let expected = a.union(&b).count();
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let actual = (a_r | b_r).into_iter().size_hint();
    check_size_hint((a, b), expected, actual)
}

#[quickcheck]
fn multi_union_size_hint(inputs: Vec<Reference>) -> bool {
    let expected: Reference = inputs.iter().flatten().copied().collect();
    let actual = inputs
        .iter()
        .map(RangeSetBlaze::from_iter)
        .union()
        .iter()
        .size_hint();
    check_size_hint(inputs, expected.len(), actual)
}

#[quickcheck]
fn difference_size_hint(a: Reference, b: Reference) -> bool {
    let expected = a.difference(&b).count();
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let actual = (a_r - b_r).into_iter().size_hint();
    check_size_hint((a, b), expected, actual)
}

#[quickcheck]
fn symmetric_difference_size_hint(a: Reference, b: Reference) -> bool {
    let expected = a.symmetric_difference(&b).count();
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let actual = (a_r ^ b_r).into_iter().size_hint();
    check_size_hint((a, b), expected, actual)
}

// cmk0 get working again
// #[should_panic]
// #[test]
// fn demo_read() {
//     let _a: RangeSetBlaze<i32> = demo_read_ranges_from_file("tests/no_such_file").unwrap();
// }

// cmk0 get working again
// #[test]
// fn double_end_iter() {
//     let a = RangeSetBlaze::from_iter([3..=10, 12..=12, 20..=25]);

//     assert_eq!(
//         a.iter().rev().collect::<Vec<usize>>(),
//         vec![25, 24, 23, 22, 21, 20, 12, 10, 9, 8, 7, 6, 5, 4, 3]
//     );

//     {
//         let mut iter = a.iter();

//         assert_eq!(iter.next(), Some(3));
//         assert_eq!(iter.next_back(), Some(25));
//         assert_eq!(iter.next(), Some(4));
//         assert_eq!(iter.next_back(), Some(24));
//         assert_eq!(iter.next_back(), Some(23));
//         assert_eq!(iter.next_back(), Some(22));
//         assert_eq!(iter.next_back(), Some(21));
//         assert_eq!(iter.next_back(), Some(20));

//         // Next interval
//         assert_eq!(iter.next_back(), Some(12));

//         // Next interval, now same interval as front of the iterator
//         assert_eq!(iter.next_back(), Some(10));
//         assert_eq!(iter.next(), Some(5));
//     }
// }

#[test]
fn double_end_into_iter() {
    let a = RangeSetBlaze::from_iter([3..=10, 12..=12, 20..=25]);

    assert_eq!(
        a.clone().into_iter().rev().collect::<Vec<usize>>(),
        vec![25, 24, 23, 22, 21, 20, 12, 10, 9, 8, 7, 6, 5, 4, 3]
    );

    let mut iter = a.into_iter();

    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next_back(), Some(25));
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.next_back(), Some(24));
    assert_eq!(iter.next_back(), Some(23));
    assert_eq!(iter.next_back(), Some(22));
    assert_eq!(iter.next_back(), Some(21));
    assert_eq!(iter.next_back(), Some(20));

    // Next interval
    assert_eq!(iter.next_back(), Some(12));

    // Next interval, now same interval as front of the iterator
    assert_eq!(iter.next_back(), Some(10));
    assert_eq!(iter.next(), Some(5));
}
#[test]
fn double_end_range() {
    let a = RangeSetBlaze::from_iter([3..=10, 12..=12, 20..=25]);

    let mut range = a.range(11..=22);
    assert_eq!(range.next_back(), Some(22));
    assert_eq!(range.next(), Some(12));
    assert_eq!(range.next(), Some(20));
}

#[test]
fn set_random_symmetric_difference() {
    use crate::CheckSortedDisjointMap;
    use crate::RangeSetBlaze;

    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut set0 = RangeSetBlaze::new();
        let mut set1 = RangeSetBlaze::new();

        for _ in 0..500 {
            let key = rng.gen_range(0..=255u8);
            set0.insert(key);
            print!("l{key} ");
            let key = rng.gen_range(0..=255u8);
            set1.insert(key);
            print!("r{key} ");

            let symmetric_difference = SymDiffIter::new2(set0.ranges(), set1.ranges());

            // println!(
            //     "left ^ right = {}",
            //     SymDiffIter::new2(set0.ranges(), set1.ranges()).into_string()
            // );

            let map0 = CheckSortedDisjointMap::new(set0.ranges().map(|range| (range.clone(), &())))
                .into_range_map_blaze();
            let map1 = CheckSortedDisjointMap::new(set1.ranges().map(|range| (range.clone(), &())))
                .into_range_map_blaze();
            let mut expected_map = &map0 ^ &map1;

            println!();
            println!("set0: {set0}");
            println!("set1: {set1}");

            for range in symmetric_difference {
                // println!();
                // print!("removing ");
                for k in range {
                    let get0 = set0.get(k);
                    let get1 = set1.get(k);
                    match (get0, get1) {
                        (Some(_k0), Some(_k1)) => {
                            println!();
                            println!("left: {}", set0);
                            println!("right: {}", set1);
                            let s_d = SymDiffIter::new2(set0.ranges(), set1.ranges())
                                .into_range_set_blaze();
                            panic!("left ^ right = {s_d}");
                        }
                        (Some(_k0), None) => {}
                        (None, Some(_k1)) => {}
                        (None, None) => {
                            panic!("should not happen 1");
                        }
                    }
                    assert!(expected_map.remove(k).is_some());
                }
                // println!();
            }
            if !expected_map.is_empty() {
                println!();
                println!("left: {}", set0);
                println!("right: {}", set1);
                let s_d = SymDiffIter::new2(set0.ranges(), set1.ranges()).into_range_set_blaze();
                println!("left ^ right = {s_d}");
                panic!("expected_keys should be empty: {expected_map}");
            }
        }
    }
}

#[test]
fn set_sym_diff_repro1() {
    use crate::RangeSetBlaze;

    let l = RangeSetBlaze::from_iter([157..=158]);
    let r = RangeSetBlaze::from_iter([158..=158]);
    let iter = SymDiffIter::new2(l.ranges(), r.ranges());
    let v = iter.collect::<Vec<_>>();
    println!("{v:?}");
}

#[test]
fn sdi1() {
    let a = [157..=158, 158..=158].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(157..=157));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=0, 0..=1, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(0..=100));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=1, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(1..=100));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=0, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(2..=100));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=0, 0..=0, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(0..=0));
    assert_eq!(iter.next(), Some(2..=100));
    assert_eq!(iter.next(), None);
    {
        let a = [0..=1, 0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(1..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=1, 0..=0, 0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 0..=0, 0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=0));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), None);

        let a = [0..=0, 1..=1].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 1..=1].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 2..=2].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=0));
        assert_eq!(iter.next(), Some(2..=2));
        assert_eq!(iter.next(), None);

        let a = [0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=0));
        assert_eq!(iter.next(), None);

        let a: array::IntoIter<RangeInclusive<i32>, 0> = [].into_iter();
        let a = AssumeSortedStarts::new(a);
        let iter = SymDiffIter::new(a);
        let v = iter.collect::<Vec<_>>();
        assert_eq!(v, vec![]);
    }
}

// // FUTURE: use fn range to implement one-at-a-time intersection, difference, etc. and then add more inplace ops.
// cmk00 Can we/should we hide MergeMapIter and KMergeMapIter and SymDiffMapIter::new and UnionMapIter::new?
#[test]
// cmk000 challenge: convert from every level to sorted disjoint* for both map and set.
pub fn convert_challenge() {
    use itertools::Itertools;
    use unsorted_disjoint_map::UnsortedPriorityDisjointMap;

    // cmk000 what is the for?
    fn _is_sorted_disjoint_map<T, V, VR, S>(_iter: S)
    where
        T: Integer,
        V: ValueOwned,
        VR: CloneBorrow<V>,
        S: SortedDisjointMap<T, V, VR>,
    {
    }

    //===========================
    // Map - ranges
    //===========================

    // * from sorted_disjoint
    let a = CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
    assert!(a.equal(CheckSortedDisjointMap::new([
        (1..=2, &"a"),
        (5..=100, &"a")
    ])));

    // cmk00 should "to_string" be "into_string" ???

    // * from (priority) sorted_starts
    let a = [(1..=4, &"a"), (5..=100, &"a"), (5..=5, &"b")].into_iter();
    // cmk00 should we reverse the sense of priority_number so lower is better?
    let a = a
        .enumerate()
        .map(|(i, range_value)| Priority::new(range_value, i));
    let a = AssumePrioritySortedStartsMap::new(a);
    let a = UnionIterMap::new(a);
    // is_sorted_disjoint_map::<_, _, _, _>(a);
    assert!(a.equal(CheckSortedDisjointMap::new([(1..=100, &"a"),])));

    // * from unsorted_disjoint
    let iter = [(5..=100, &"a"), (5..=5, &"b"), (1..=4, &"a")].into_iter();
    let iter = iter
        .enumerate()
        .map(|(i, range_value)| Priority::new(range_value, i));
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    assert!(iter.equal(CheckSortedDisjointMap::new([(1..=100, &"a"),])));

    // * anything
    let iter = [(5, &"a"), (5, &"b"), (1, &"a")]
        .into_iter()
        .map(|(x, y)| (x..=x, y));
    let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    assert!(iter.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a"),])));

    //===========================
    // Map - points
    //===========================

    // * from sorted_disjoint
    let a = [(1, &"a"), (5, &"a")].into_iter().map(|(x, y)| (x..=x, y));
    let a = CheckSortedDisjointMap::new(a);
    assert!(a.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a")])));

    // cmk00 should "to_string" be "into_string" ???

    // * from (priority) sorted_starts
    let a = [(1, &"a"), (5, &"a"), (5, &"b")].into_iter();
    // cmk00 should we reverse the sense of priority_number so lower is better?
    let a = a
        .enumerate()
        .map(|(i, (k, v))| Priority::new((k..=k, v), i));
    let a = AssumePrioritySortedStartsMap::new(a);
    let a = UnionIterMap::new(a);
    // is_sorted_disjoint_map::<_, _, _, _>(a);
    assert!(a.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a")])));

    // * from unsorted_disjoint
    let iter = [(5, &"a"), (5, &"b"), (1, &"a")].into_iter();
    let iter = iter
        .enumerate()
        .map(|(i, (k, v))| Priority::new((k..=k, v), i));
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    assert!(iter.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a")])));

    // * anything
    let iter = [(5..=100, &"a"), (5..=5, &"b"), (1..=4, &"a")].into_iter();
    let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    assert!(iter.equal(CheckSortedDisjointMap::new([(1..=100, &"a"),])));

    //===========================
    // Set - ranges
    //===========================

    // * from sorted_disjoint
    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    assert!(a.equal(CheckSortedDisjoint::new([1..=2, 5..=100])));

    // cmk00 should "to_string" be "into_string" ???

    // * from (priority) sorted_starts
    let a = [1..=4, 5..=100, 5..=5].into_iter();
    // cmk00 should we reverse the sense of priority_number so lower is better?
    let a = AssumeSortedStarts::new(a);
    let a = UnionIter::new(a);
    assert!(a.equal(CheckSortedDisjoint::new([1..=100])));

    // * from unsorted_disjoint
    let iter = [5..=100, 5..=5, 1..=4].into_iter();
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumeSortedStarts::new(iter);
    let iter = UnionIter::new(iter);
    assert!(iter.equal(CheckSortedDisjoint::new([1..=100])));

    // * anything
    let iter = [5..=100, 5..=5, 1..=5].into_iter();
    let iter = iter.sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumeSortedStarts::new(iter);
    let iter = UnionIter::new(iter);
    assert!(iter.equal(CheckSortedDisjoint::new([1..=100])));
    // Set - points

    // what about multiple inputs?
}
