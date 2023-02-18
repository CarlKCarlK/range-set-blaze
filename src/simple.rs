#![cfg(test)]

use maplit::btreeset;
use sorted_iter::{assume::AssumeSortedByItemExt, multiway_union, SortedIterator};

#[test]
fn test11() {
    let fibs = btreeset! { 1, 2, 3, 5, 8, 13u64 }.into_iter();
    let evens = (0..14).filter(|x| x % 2 == 0);
    let fib_evens = fibs.union(evens);
    println!("fib_evens: {:?}", fib_evens.collect::<Vec<_>>());

    // let primes = btreeset! { 2, 3, 5, 7, 11, 13u64 }.into_iter();
    // let fibs = btreeset! { 1, 2, 3, 5, 8, 13u64 }.into_iter();
    // let evens = (0..14u64).filter(|x| x % 2 == 0);
    // let mw = multiway_union([primes, fibs, evens].into_iter());
    // println!("mw: {:?}", mw.collect::<Vec<_>>());

    fn primes_new() -> Box<dyn Iterator<Item = u64>> {
        Box::new(btreeset! { 2, 3, 5, 7, 11, 13u64 }.into_iter())
    }

    fn fibs_new() -> Box<dyn Iterator<Item = u64>> {
        Box::new([1, 2, 3, 5, 8, 13u64].into_iter())
    }

    fn even_new() -> Box<dyn Iterator<Item = u64>> {
        Box::new((0..14).filter(|x| x % 2 == 0))
    }

    let primes = primes_new();
    let fibs0 = fibs_new();
    let fibs1 = fibs_new();
    let evens = even_new();
    let mw = multiway_union(
        [
            primes.assume_sorted_by_item(),
            fibs0.assume_sorted_by_item(),
            fibs1.assume_sorted_by_item(),
            evens.assume_sorted_by_item(),
        ]
        .into_iter(),
    );
    println!("mw: {:?}", mw.collect::<Vec<_>>());
}
