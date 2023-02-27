#![feature(prelude_import)]
#![cfg(test)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use itertools::Itertools;
use std::{collections::BTreeSet, ops::BitOr};
use syntactic_for::syntactic_for;
use range_set_int::{
    intersection_dyn, union_dyn, BitOrIter, DynSortedDisjointExt, ItertoolsPlus2,
    RangeSetInt, Ranges, SortedDisjointIterator,
};
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "insert_255u8"]
pub const insert_255u8: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("insert_255u8"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(insert_255u8())),
};
fn insert_255u8() {
    let range_set_int = RangeSetInt::from([255u8]);
    if !(range_set_int.to_string() == "255..=255") {
        ::core::panicking::panic(
            "assertion failed: range_set_int.to_string() == \\\"255..=255\\\"",
        )
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "insert_max_u128"]
pub const insert_max_u128: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("insert_max_u128"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::Yes,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(insert_max_u128())),
};
#[should_panic]
fn insert_max_u128() {
    let _ = RangeSetInt::<u128>::from([u128::MAX]);
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "complement0"]
pub const complement0: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("complement0"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(complement0())),
};
fn complement0() {
    let empty = RangeSetInt::<i8>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<u8>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<isize>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<usize>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<i16>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<u16>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<i32>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<u32>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<i64>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<u64>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<isize>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<usize>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<i128>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
    let empty = RangeSetInt::<u128>::new();
    let full = !&empty;
    {
        ::std::io::_print(
            format_args!(
                "empty: {2} (len {0}), full: {3} (len {1})\n", empty.len(), full.len(),
                empty, full
            ),
        );
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "repro_bit_and"]
pub const repro_bit_and: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("repro_bit_and"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(repro_bit_and())),
};
fn repro_bit_and() {
    let a = RangeSetInt::from([1u8, 2, 3]);
    let b = RangeSetInt::from([2u8, 3, 4]);
    let result = &a & &b;
    {
        ::std::io::_print(format_args!("{0}\n", result));
    };
    match (&result, &RangeSetInt::from([2u8, 3])) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "doctest1"]
pub const doctest1: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("doctest1"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(doctest1())),
};
fn doctest1() {
    let a = RangeSetInt::<u8>::from([1, 2, 3]);
    let b = RangeSetInt::<u8>::from([3, 4, 5]);
    let result = &a | &b;
    match (&result, &RangeSetInt::<u8>::from([1, 2, 3, 4, 5])) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "doctest2"]
pub const doctest2: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("doctest2"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(doctest2())),
};
fn doctest2() {
    let set = RangeSetInt::<u8>::from([1, 2, 3]);
    if !set.contains(1) {
        ::core::panicking::panic("assertion failed: set.contains(1)")
    }
    if !!set.contains(4) {
        ::core::panicking::panic("assertion failed: !set.contains(4)")
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "doctest3"]
pub const doctest3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("doctest3"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(doctest3())),
};
fn doctest3() {
    let mut a = RangeSetInt::<u8>::from("1..=3");
    let mut b = RangeSetInt::<u8>::from("3..=5");
    a.append(&mut b);
    match (&a.len(), &5) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&b.len(), &0) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    if !a.contains(1) {
        ::core::panicking::panic("assertion failed: a.contains(1)")
    }
    if !a.contains(2) {
        ::core::panicking::panic("assertion failed: a.contains(2)")
    }
    if !a.contains(3) {
        ::core::panicking::panic("assertion failed: a.contains(3)")
    }
    if !a.contains(4) {
        ::core::panicking::panic("assertion failed: a.contains(4)")
    }
    if !a.contains(5) {
        ::core::panicking::panic("assertion failed: a.contains(5)")
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "doctest4"]
pub const doctest4: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("doctest4"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(doctest4())),
};
fn doctest4() {
    let a = RangeSetInt::<i8>::from([1, 2, 3]);
    let result = !&a;
    match (&result.to_string(), &"-128..=0,4..=127") {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "compare"]
pub const compare: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("compare"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(compare())),
};
fn compare() {
    let mut btree_set = BTreeSet::<u128>::new();
    btree_set.insert(3);
    btree_set.insert(1);
    let string = btree_set.iter().join(",");
    {
        ::std::io::_print(format_args!("{0:#?}\n", string));
    };
    if !(string == "1,3") {
        ::core::panicking::panic("assertion failed: string == \\\"1,3\\\"")
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "add_in_order"]
pub const add_in_order: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("add_in_order"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(add_in_order())),
};
fn add_in_order() {
    let mut range_set = RangeSetInt::new();
    for i in 0u64..1000 {
        range_set.insert(i);
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "iters"]
pub const iters: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("iters"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(iters())),
};
fn iters() {
    let range_set_int = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    if !(range_set_int.len() == 13) {
        ::core::panicking::panic("assertion failed: range_set_int.len() == 13")
    }
    for i in range_set_int.iter() {
        {
            ::std::io::_print(format_args!("{0}\n", i));
        };
    }
    for (start, stop) in range_set_int.ranges() {
        {
            ::std::io::_print(format_args!("{0} {1}\n", start, stop));
        };
    }
    let mut rs = range_set_int.ranges();
    {
        ::std::io::_print(format_args!("{0:?}\n", rs.next()));
    };
    {
        ::std::io::_print(format_args!("{0}\n", range_set_int));
    };
    {
        ::std::io::_print(format_args!("{0:?}\n", rs.len()));
    };
    {
        ::std::io::_print(format_args!("{0:?}\n", rs.next()));
    };
    for i in range_set_int.iter() {
        {
            ::std::io::_print(format_args!("{0}\n", i));
        };
    }
    range_set_int.len();
    let mut rs = range_set_int.ranges().not();
    {
        ::std::io::_print(format_args!("{0:?}\n", rs.next()));
    };
    {
        ::std::io::_print(format_args!("{0}\n", range_set_int));
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "missing_doctest_ops"]
pub const missing_doctest_ops: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("missing_doctest_ops"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(missing_doctest_ops())),
};
fn missing_doctest_ops() {
    let a = RangeSetInt::from([1, 2, 3]);
    let b = RangeSetInt::from([3, 4, 5]);
    let result = &a | &b;
    match (&result, &RangeSetInt::from([1, 2, 3, 4, 5])) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let result = a | &b;
    match (&result, &RangeSetInt::from([1, 2, 3, 4, 5])) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let a = RangeSetInt::<i8>::from([1, 2, 3]);
    let result = !&a;
    match (&result.to_string(), &"-128..=0,4..=127") {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let result = !a;
    match (&result.to_string(), &"-128..=0,4..=127") {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let a = RangeSetInt::from([1, 2, 3]);
    let b = RangeSetInt::from([2, 3, 4]);
    let result = a & &b;
    match (&result, &RangeSetInt::from([2, 3])) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let a = RangeSetInt::from([1, 2, 3]);
    let result = a & b;
    match (&result, &RangeSetInt::from([2, 3])) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let a = RangeSetInt::from([1, 2, 3]);
    let b = RangeSetInt::from([2, 3, 4]);
    let result = a ^ b;
    match (&result, &RangeSetInt::from([1, 4])) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let a = RangeSetInt::from([1, 2, 3]);
    let b = RangeSetInt::from([2, 3, 4]);
    let result = a - b;
    match (&result, &RangeSetInt::from([1])) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "multi_op"]
pub const multi_op: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("multi_op"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(multi_op())),
};
fn multi_op() {
    let a = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let b = RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = RangeSetInt::<u8>::from("38..=42");
    let d = &(&a | &b) | &c;
    {
        ::std::io::_print(format_args!("{0}\n", d));
    };
    let d = a | b | &c;
    {
        ::std::io::_print(format_args!("{0}\n", d));
    };
    let a = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let b = RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = RangeSetInt::<u8>::from("38..=42");
    let _ = RangeSetInt::union([&a, &b, &c]);
    let d = RangeSetInt::intersection([a, b, c].iter());
    match (&d, &RangeSetInt::new()) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&!RangeSetInt::<u8>::union([]), &RangeSetInt::from("0..=255")) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let a = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let b = RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = RangeSetInt::<u8>::from("1..=42");
    let _ = &a & &b;
    let d = RangeSetInt::intersection([&a, &b, &c]);
    {
        ::std::io::_print(format_args!("{0}\n", d));
    };
    match (&d, &RangeSetInt::from("5..=6,8..=9,11..=13")) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&RangeSetInt::<u8>::intersection([]), &RangeSetInt::from("0..=255")) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "custom_multi"]
pub const custom_multi: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("custom_multi"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(custom_multi())),
};
fn custom_multi() {
    let a = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let b = RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = RangeSetInt::<u8>::from("38..=42");
    let union_stream = b.ranges() | c.ranges();
    let a_less = a.ranges().sub(union_stream);
    let d: RangeSetInt<_> = a_less.into();
    {
        ::std::io::_print(format_args!("{0}\n", d));
    };
    let d: RangeSetInt<_> = a.ranges().sub([b.ranges(), c.ranges()].union()).into();
    {
        ::std::io::_print(format_args!("{0}\n", d));
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "from_string"]
pub const from_string: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("from_string"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(from_string())),
};
fn from_string() {
    let a = RangeSetInt::<u16>::from("0..=4,14..=17,30..=255,0..=37,43..=65535");
    match (&a, &RangeSetInt::from("0..=65535")) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "nand_repro"]
pub const nand_repro: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("nand_repro"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(nand_repro())),
};
fn nand_repro() {
    let b = &RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = &RangeSetInt::<u8>::from("38..=42");
    {
        ::std::io::_print(format_args!("about to nand\n"));
    };
    let d = !b | !c;
    {
        ::std::io::_print(format_args!("cmk \'{0}\'\n", d));
    };
    match (&d, &RangeSetInt::from("0..=4,14..=17,30..=255,0..=37,43..=255")) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "parity"]
pub const parity: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("parity"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(parity())),
};
fn parity() {
    let a = &RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let b = &RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = &RangeSetInt::<u8>::from("38..=42");
    match (
        &(a & !b & !c | !a & b & !c | !a & !b & c | a & b & c),
        &RangeSetInt::from("1..=4,7..=7,10..=10,14..=15,18..=29,38..=42"),
    ) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let _d = [a.ranges()].intersection();
    let _parity: RangeSetInt<u8> = [[a.ranges()].intersection()].union().into();
    let _parity: RangeSetInt<u8> = [a.ranges()].intersection().into();
    let _parity: RangeSetInt<u8> = [a.ranges()].union().into();
    {
        ::std::io::_print(format_args!("!b {0}\n", ! b));
    };
    {
        ::std::io::_print(format_args!("!c {0}\n", ! c));
    };
    {
        ::std::io::_print(format_args!("!b|!c {0}\n", ! b | ! c));
    };
    {
        ::std::io::_print(
            format_args!(
                "!b|!c {0}\n", RangeSetInt::from(b.ranges().not() | c.ranges().not())
            ),
        );
    };
    let _a = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let u = [a.ranges().dyn_sorted_disjoint()].union();
    match (&RangeSetInt::from(u), &RangeSetInt::from("1..=6,8..=9,11..=15")) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let u = {
        let arr = [a.ranges().dyn_sorted_disjoint()];
        arr.union()
    };
    match (&RangeSetInt::from(u), &RangeSetInt::from("1..=6,8..=9,11..=15")) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let u = {
        let arr = [
            a.ranges().dyn_sorted_disjoint(),
            b.ranges().dyn_sorted_disjoint(),
            c.ranges().dyn_sorted_disjoint(),
        ];
        arr.union()
    };
    match (&RangeSetInt::from(u), &RangeSetInt::from("1..=15,18..=29,38..=42")) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let u = [
        {
            let arr = [
                a.ranges().dyn_sorted_disjoint(),
                b.ranges().not().dyn_sorted_disjoint(),
                c.ranges().not().dyn_sorted_disjoint(),
            ];
            arr.intersection()
        },
        {
            let arr = [
                a.ranges().not().dyn_sorted_disjoint(),
                b.ranges().dyn_sorted_disjoint(),
                c.ranges().not().dyn_sorted_disjoint(),
            ];
            arr.intersection()
        },
        {
            let arr = [
                a.ranges().not().dyn_sorted_disjoint(),
                b.ranges().not().dyn_sorted_disjoint(),
                c.ranges().dyn_sorted_disjoint(),
            ];
            arr.intersection()
        },
        {
            let arr = [
                a.ranges().dyn_sorted_disjoint(),
                b.ranges().dyn_sorted_disjoint(),
                c.ranges().dyn_sorted_disjoint(),
            ];
            arr.intersection()
        },
    ]
        .union();
    match (
        &RangeSetInt::from(u),
        &RangeSetInt::from("1..=4,7..=7,10..=10,14..=15,18..=29,38..=42"),
    ) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "ui"]
pub const ui: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("ui"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(ui())),
};
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "complement"]
pub const complement: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("complement"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(complement())),
};
fn complement() {
    let a0 = RangeSetInt::<u8>::from("1..=6");
    let a1 = RangeSetInt::from("8..=9,11..=15");
    let a = &a0 | &a1;
    let not_a = !&a;
    let b = a.ranges();
    let c = !not_a.ranges();
    let d = a0.ranges() | a1.ranges();
    let (e, _) = a.ranges().tee();
    let f: BitOrIter<_, _> = [15, 14, 15, 13, 12, 11, 9, 9, 8, 6, 4, 5, 3, 2, 1, 1, 1]
        .into_iter()
        .collect();
    let not_b = !b;
    let not_c = !c;
    let not_d = !d;
    let not_e = e.not();
    let not_f = !f;
    if !not_a.ranges().equal(not_b) {
        ::core::panicking::panic("assertion failed: not_a.ranges().equal(not_b)")
    }
    if !not_a.ranges().equal(not_c) {
        ::core::panicking::panic("assertion failed: not_a.ranges().equal(not_c)")
    }
    if !not_a.ranges().equal(not_d) {
        ::core::panicking::panic("assertion failed: not_a.ranges().equal(not_d)")
    }
    if !not_a.ranges().equal(not_e) {
        ::core::panicking::panic("assertion failed: not_a.ranges().equal(not_e)")
    }
    if !not_a.ranges().equal(not_f) {
        ::core::panicking::panic("assertion failed: not_a.ranges().equal(not_f)")
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "union"]
pub const union: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("union"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(union())),
};
fn union() {
    let a0 = RangeSetInt::<u8>::from("1..=6");
    let (a0_tee, _) = a0.ranges().tee();
    let a1 = RangeSetInt::<u8>::from("8..=9");
    let a2 = RangeSetInt::from("11..=15");
    let a12 = &a1 | &a2;
    let not_a0 = !&a0;
    let a = &a0 | &a1 | &a2;
    let b = a0.ranges() | a1.ranges() | a2.ranges();
    let c = !not_a0.ranges() | a12.ranges();
    let d = a0.ranges() | a1.ranges() | a2.ranges();
    let e = a0_tee.bitor(a12.ranges());
    let f = a0.iter().collect::<BitOrIter<_, _>>()
        | a1.iter().collect::<BitOrIter<_, _>>()
        | a2.iter().collect::<BitOrIter<_, _>>();
    if !a.ranges().equal(b) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(b)")
    }
    if !a.ranges().equal(c) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(c)")
    }
    if !a.ranges().equal(d) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(d)")
    }
    if !a.ranges().equal(e) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(e)")
    }
    if !a.ranges().equal(f) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(f)")
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "sub"]
pub const sub: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("sub"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(sub())),
};
fn sub() {
    let a0 = RangeSetInt::<u8>::from("1..=6");
    let a1 = RangeSetInt::<u8>::from("8..=9");
    let a2 = RangeSetInt::from("11..=15");
    let a01 = &a0 | &a1;
    let (a01_tee, _) = a01.ranges().tee();
    let not_a01 = !&a01;
    let a = &a01 - &a2;
    let b = a01.ranges() - a2.ranges();
    let c = !not_a01.ranges() - a2.ranges();
    let d = (a0.ranges() | a1.ranges()) - a2.ranges();
    let e = a01_tee.sub(a2.ranges());
    let f = a01.iter().collect::<BitOrIter<_, _>>()
        - a2.iter().collect::<BitOrIter<_, _>>();
    if !a.ranges().equal(b) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(b)")
    }
    if !a.ranges().equal(c) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(c)")
    }
    if !a.ranges().equal(d) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(d)")
    }
    if !a.ranges().equal(e) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(e)")
    }
    if !a.ranges().equal(f) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(f)")
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "xor"]
pub const xor: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("xor"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(xor())),
};
fn xor() {
    let a0 = RangeSetInt::<u8>::from("1..=6");
    let a1 = RangeSetInt::<u8>::from("8..=9");
    let a2 = RangeSetInt::from("11..=15");
    let a01 = &a0 | &a1;
    let (a01_tee, _) = a01.ranges().tee();
    let not_a01 = !&a01;
    let a = &a01 ^ &a2;
    let b = a01.ranges() ^ a2.ranges();
    let c = !not_a01.ranges() ^ a2.ranges();
    let d = (a0.ranges() | a1.ranges()) ^ a2.ranges();
    let e = a01_tee.bitxor(a2.ranges());
    let f = a01.iter().collect::<BitOrIter<_, _>>()
        ^ a2.iter().collect::<BitOrIter<_, _>>();
    if !a.ranges().equal(b) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(b)")
    }
    if !a.ranges().equal(c) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(c)")
    }
    if !a.ranges().equal(d) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(d)")
    }
    if !a.ranges().equal(e) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(e)")
    }
    if !a.ranges().equal(f) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(f)")
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "bitand"]
pub const bitand: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("bitand"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(bitand())),
};
fn bitand() {
    let a0 = RangeSetInt::<u8>::from("1..=6");
    let a1 = RangeSetInt::<u8>::from("8..=9");
    let a2 = RangeSetInt::from("11..=15");
    let a01 = &a0 | &a1;
    let (a01_tee, _) = a01.ranges().tee();
    let not_a01 = !&a01;
    let a = &a01 & &a2;
    let b = a01.ranges() & a2.ranges();
    let c = !not_a01.ranges() & a2.ranges();
    let d = (a0.ranges() | a1.ranges()) & a2.ranges();
    let e = a01_tee.bitand(a2.ranges());
    let f = a01.iter().collect::<BitOrIter<_, _>>()
        & a2.iter().collect::<BitOrIter<_, _>>();
    if !a.ranges().equal(b) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(b)")
    }
    if !a.ranges().equal(c) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(c)")
    }
    if !a.ranges().equal(d) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(d)")
    }
    if !a.ranges().equal(e) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(e)")
    }
    if !a.ranges().equal(f) {
        ::core::panicking::panic("assertion failed: a.ranges().equal(f)")
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "empty"]
pub const empty: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("empty"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(empty())),
};
fn empty() {
    let universe: BitOrIter<u8, _> = [(0, 255)].into_iter().collect();
    let a0 = RangeSetInt::<u8>::from([]);
    if !!(a0.ranges()).equal(universe.clone()) {
        ::core::panicking::panic(
            "assertion failed: !(a0.ranges()).equal(universe.clone())",
        )
    }
    if !(!a0).ranges().equal(universe) {
        ::core::panicking::panic("assertion failed: (!a0).ranges().equal(universe)")
    }
    let _a0 = RangeSetInt::<u8>::from("");
    let _a = RangeSetInt::<i32>::new();
    let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
    let a = a_iter.collect::<RangeSetInt<i32>>();
    let b = RangeSetInt::from([]);
    let b_ref: [&i32; 0] = [];
    let mut c3 = a.clone();
    let mut c4 = a.clone();
    let mut c5 = a.clone();
    let c0 = (&a).bitor(&b);
    let c1a = &a | &b;
    let c1b = &a | b.clone();
    let c1c = a.clone() | &b;
    let c1d = a.clone() | b.clone();
    let c2: RangeSetInt<_> = (a.ranges() | b.ranges()).into();
    c3.append(&mut b.clone());
    c4.extend(b_ref);
    c5.extend(b);
    let answer = RangeSetInt::from([]);
    match (&&c0, &&answer) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&&c1a, &&answer) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&&c1b, &&answer) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&&c1c, &&answer) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&&c1d, &&answer) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&&c2, &&answer) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&&c3, &&answer) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&&c4, &&answer) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&&c5, &&answer) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    use range_set_int::ItertoolsPlus2;
    let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
    let a = a_iter.collect::<RangeSetInt<i32>>();
    let b = RangeSetInt::from([]);
    let c0 = a.ranges() | b.ranges();
    let c1 = range_set_int::union([a.ranges(), b.ranges()]);
    let c_list2: [Ranges<i32>; 0] = [];
    let c2 = c_list2.clone().union();
    let c3 = {
        let arr = [a.ranges().dyn_sorted_disjoint(), b.ranges().dyn_sorted_disjoint()];
        arr.union()
    };
    let c4 = c_list2.map(|x| x.dyn_sorted_disjoint()).union();
    let answer = RangeSetInt::from([]);
    if !c0.equal(answer.ranges()) {
        ::core::panicking::panic("assertion failed: c0.equal(answer.ranges())")
    }
    if !c1.equal(answer.ranges()) {
        ::core::panicking::panic("assertion failed: c1.equal(answer.ranges())")
    }
    if !c2.equal(answer.ranges()) {
        ::core::panicking::panic("assertion failed: c2.equal(answer.ranges())")
    }
    if !c3.equal(answer.ranges()) {
        ::core::panicking::panic("assertion failed: c3.equal(answer.ranges())")
    }
    if !c4.equal(answer.ranges()) {
        ::core::panicking::panic("assertion failed: c4.equal(answer.ranges())")
    }
    let c0 = !(a.ranges() & b.ranges());
    let c1 = !range_set_int::intersection([a.ranges(), b.ranges()]);
    let c_list2: [Ranges<i32>; 0] = [];
    let c2 = c_list2.clone().intersection();
    let c3a = {
        let arr = [a.ranges().dyn_sorted_disjoint(), b.ranges().dyn_sorted_disjoint()];
        arr.union()
    };
    let c3 = !c3a;
    let c3_22 = !({
        let arr = [a.ranges().dyn_sorted_disjoint(), b.ranges().dyn_sorted_disjoint()];
        arr.intersection()
    });
    let c3c = !([a.ranges(), b.ranges()]
        .into_iter()
        .map(|x| x.dyn_sorted_disjoint())
        .intersection());
    let c3d = ![a.ranges().dyn_sorted_disjoint(), b.ranges().dyn_sorted_disjoint()]
        .intersection();
    let c4 = !(c_list2.map(|x| x.dyn_sorted_disjoint()).intersection());
    let answer = !RangeSetInt::from([]);
    if !c0.equal(answer.ranges()) {
        ::core::panicking::panic("assertion failed: c0.equal(answer.ranges())")
    }
    if !c1.equal(answer.ranges()) {
        ::core::panicking::panic("assertion failed: c1.equal(answer.ranges())")
    }
    if !c2.equal(answer.ranges()) {
        ::core::panicking::panic("assertion failed: c2.equal(answer.ranges())")
    }
    if !c4.equal(answer.ranges()) {
        ::core::panicking::panic("assertion failed: c4.equal(answer.ranges())")
    }
}
#[rustc_main]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(
        &[
            &add_in_order,
            &bitand,
            &compare,
            &complement,
            &complement0,
            &custom_multi,
            &doctest1,
            &doctest2,
            &doctest3,
            &doctest4,
            &empty,
            &from_string,
            &insert_255u8,
            &insert_max_u128,
            &iters,
            &missing_doctest_ops,
            &multi_op,
            &nand_repro,
            &parity,
            &repro_bit_and,
            &sub,
            &ui,
            &union,
            &xor,
        ],
    )
}
