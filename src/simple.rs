#![cfg(test)]

use maplit::btreeset;
use sorted_iter::{multiway_union, SortedIterator};

#[test]
fn test11() {
    let primes = btreeset! { 2, 3, 5, 7, 11, 13u64 }.into_iter();
    let fibs = btreeset! { 1, 2, 3, 5, 8, 13u64 }.into_iter();
    let fib_primes = primes.intersection(fibs);
    println!("fib_primes: {:?}", fib_primes.collect::<Vec<_>>());

    let primes = btreeset! { 2, 3, 5, 7, 11, 13u64 }.into_iter();
    let fibs = btreeset! { 1, 2, 3, 5, 8, 13u64 }.into_iter();
    let evens = btreeset! { 2, 4, 6, 8, 10, 12, 14u64 }.into_iter();
    let mw = multiway_union([primes, fibs, evens].into_iter());
    println!("mw: {:?}", mw.collect::<Vec<_>>());

    let fibs = btreeset! { 1, 2, 3, 5, 8, 13u64 }.into_iter();
    let evens = (0..14).filter(|x| x % 2 == 0);
    let fib_evens = fibs.intersection(evens);
    println!("fib_evens: {:?}", fib_evens.collect::<Vec<_>>());

    let primes = btreeset! { 2, 3, 5, 7, 11, 13u64 }.into_iter();
    let fibs = btreeset! { 1, 2, 3, 5, 8, 13u64 }.into_iter();
    let evens = (0..14u64).filter(|x| x % 2 == 0);
    let mw = multiway_union([primes, fibs, evens].into_iter());
    println!("mw: {:?}", mw.collect::<Vec<_>>());
}
