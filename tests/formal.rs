// use std::collections::HashSet;

// // File: test.rs
// #[kani::proof]
// fn main() {
//     assert!(2 == 2);
// }

// fn estimate_size(x: u32) -> u32 {
//     if x < 256 {
//         if x < 128 {
//             return 1;
//         } else {
//             return 3;
//         }
//     } else if x < 1024 {
//         if x > 1022 {
//             panic!("Oh no, a failing corner case!");
//         } else {
//             return 5;
//         }
//     } else {
//         if x < 2048 {
//             return 7;
//         } else {
//             return 9;
//         }
//     }
// }

// #[cfg(kani)]
// #[kani::proof]
// fn check_estimate_size() {
//     let x: u32 = kani::any();
//     estimate_size(x);
// }

fn estimate_size(x: u32) -> u32 {
    assert!(x < 4096);

    if x < 256 {
        if x < 128 {
            return 1;
        } else {
            return 3;
        }
    } else if x < 1024 {
        if x > 1022 {
            return 4;
        } else {
            return 5;
        }
    } else {
        if x < 2048 {
            return 7;
        } else {
            return 9;
        }
    }
}

// #[cfg(kani)]
// #[kani::proof]
// fn verify_success() {
//     let x: u32 = kani::any();
//     kani::assume(x < 2048);
//     let y = estimate_size(x);
//     assert!(y < 10);
// }

// We want to create a hashset of i8 (and then u8) values. Then we want to find the length of
// the set and store it in a u8. This should fail.

// #[cfg(kani)]
// #[kani::proof]
// fn verify_len_i8() {
//     let start: i8 = kani::any();
//     let end: i8 = kani::any(); // end is inclusive
//     kani::assume(start <= end);
//     let len = end - start + 1;
//     assert!((len as i128) == ((end as i128) - (start as i128) + 1));
// }
// Failed Checks: attempt to subtract with overflow
// Failed Checks: attempt to add with overflow

// #[cfg(kani)]
// #[kani::proof]
// fn verify_len_u8() {
//     let start: u8 = kani::any();
//     let end: u8 = kani::any(); // end is inclusive
//     kani::assume(start <= end);
//     let len = end - start + 1;
//     assert!((len as i128) == ((end as i128) - (start as i128) + 1));
// }
// // Failed Checks: attempt to add with overflow

// #[cfg(kani)]
// #[kani::proof]
// fn verify_len_i8_other_sub() {
//     let start: i8 = kani::any();
//     let end: i8 = kani::any(); // end is inclusive
//     kani::assume(start <= end);
//     let len = end.overflowing_sub(start).0 as u8 as usize + 1;
//     assert!((len as i128) == ((end as i128) - (start as i128) + 1));
// }
// // // Success!

// #[cfg(kani)]
// #[kani::proof]
// fn verify_len_u8_other_sub() {
//     let start: u8 = kani::any();
//     let end: u8 = kani::any(); // end is inclusive
//     kani::assume(start <= end);
//     let len = end.overflowing_sub(start).0 as u8 as usize + 1;
//     assert!((len as i128) == ((end as i128) - (start as i128) + 1));
// }
// // Success!

// #[cfg(kani)]
// #[kani::proof]
// fn verify_len_i32_other_sub() {
//     let start: i32 = kani::any();
//     let end: i32 = kani::any(); // end is inclusive
//     kani::assume(start <= end);
//     let len = end.overflowing_sub(start).0 as u32 as usize + 1;
//     assert!((len as i128) == ((end as i128) - (start as i128) + 1));
// }
// // // Success!

// #[cfg(kani)]
// #[kani::proof]
// fn verify_len_u64_other_sub() {
//     let start: u64 = kani::any();
//     let end: u64 = kani::any(); // end is inclusive
//     kani::assume(start <= end);
//     let len = end.overflowing_sub(start).0 as u64 as u128 + 1;
//     assert!((len as i128) == ((end as i128) - (start as i128) + 1));
// }
// // Success!

// #[cfg(kani)]
// #[kani::proof]
// fn verify_len_i64_copilot() {
//     let start: i64 = kani::any();
//     let end: i64 = kani::any(); // end is inclusive
//     kani::assume(start <= end);
//     let len = end - start + 1;
//     assert!((len as i128) == ((end as i128) - (start as i128) + 1));
// }
// // Failed Checks: attempt to subtract with overflow
// // Failed Checks: attempt to add with overflow

// #[cfg(kani)]
// #[kani::proof]
// fn verify_len_i32_github() {
//     let start: i32 = kani::any();
//     let end: i32 = kani::any(); // end is inclusive
//     kani::assume(start <= end);
//     let len = end.saturating_sub(start).saturating_add(1);
//     assert!((len as i128) == ((end as i128) - (start as i128) + 1));
// }
// Failed Checks: assertion failed: (len as i128) == ((end as i128) - (start as i128) + 1)

// #[cfg(kani)]
// #[kani::proof]
// fn verify_less_than_i32_naive() {
//     let a: i32 = kani::any();
//     let b: i32 = kani::any();
//     let condition = a + 1 < b;
//     assert!(condition == (a as i128 + 1 < b as i128));
// }
// // Failed Checks: attempt to add with overflow

// #[cfg(kani)]
// #[kani::proof]
// fn verify_less_than_i32_chatgpt_a() {
//     let a: i32 = kani::any();
//     let b: i32 = kani::any();
//     let condition = match a.checked_add(1) {
//         Some(sum) => sum < b,
//         None => false,
//     };
//     assert!(condition == (a as i128 + 1 < b as i128));
// }
// VERIFICATION:- SUCCESSFUL

#[cfg(kani)]
#[kani::proof]
fn verify_less_than_i32_chatgpt_a() {
    let a: i32 = kani::any();
    let b: i32 = kani::any();
    let condition = if let Some(result) = a.checked_add(1) {
        result < b
    } else {
        false
    };
    assert!(condition == (a as i128 + 1 < b as i128));
}

// #[cfg(kani)]
// #[kani::proof]
// fn verify_less_than_i32_chatgpt_b() {
//     let a: i32 = kani::any();
//     let b: i32 = kani::any();
//     let condition = a.checked_add(1).map_or(false, |result| result < b);
//     assert!(condition == (a as i128 + 1 < b as i128));
// }
// VERIFICATION:- SUCCESSFUL

// #[cfg(kani)]
// #[kani::proof]
// fn verify_less_than_i32_human2() {
//     let a: i32 = kani::any();
//     let b: i32 = kani::any();
//     let condition = a.saturating_add(1) < b;
//     assert!(condition == (a as i128 + 1 < b as i128));
// }
// VERIFICATION:- SUCCESSFUL

// #[cfg(kani)]
// #[kani::proof]
// fn verify_less_than_i32_human() {
//     let a: i32 = kani::any();
//     let b: i32 = kani::any();
//     let condition = a < b && a + 1 < b;
//     assert!(condition == (a as i128 + 1 < b as i128));
// }
// // VERIFICATION:- SUCCESSFUL

// #[cfg(kani)]
// #[kani::proof]
// fn verify_less_than_branchless_chatGPT() {
//     let a: i64 = kani::any();
//     let b: i64 = kani::any();
//     let condition = (a < b) & (a.wrapping_add(1).wrapping_sub(b) >> 31 == 0);
//     assert!(condition == (a as i128 + 1 < b as i128));
// }
// //  Description: "assertion failed: condition == (a as i128 + 1 < b as i128)"

// #[cfg(kani)]
// #[kani::proof]
// fn verify_less_than_branchless_hybrid() {
//     let a: i64 = kani::any();
//     let b: i64 = kani::any();
//     let (plus_1_maybe_bad, overflow) = a.overflowing_add(1);
//     let condition = (a < b) & !overflow & (plus_1_maybe_bad < b);
//     assert!(condition == (a as i128 + 1 < b as i128));
// }
// // VERIFICATION:- SUCCESSFUL

// #[cfg(kani)]
// #[kani::proof]
// fn verify_less_than_branchless_hybrid() {
//     let a: i64 = kani::any();
//     let b: i64 = kani::any();
//     let condition = a != i64::MAX && a + 1 < b;
//     assert!(condition == (a as i128 + 1 < b as i128));
// }
// // VERIFICATION:- SUCCESSFUL
