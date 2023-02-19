#![cfg(test)]

use maplit::btreeset;
use sorted_iter::{
    assume::AssumeSortedByItemExt, multiway_union, sorted_iterator::SortedByItem, SortedIterator,
};

// Inspired by sorted_iter crate.

// Our main trait
pub trait SortedDisjoint {}

// define mathematical set methods, e.g. left_iter.left(right_iter) returns the left_iter.
pub trait SortedDisjointIterator: Iterator + Sized {
    fn left<J>(self, _that: J) -> Self
    where
        J: SortedDisjointIterator<Item = Self::Item>,
    {
        self
    }
    fn right<J>(self, that: J) -> J
    where
        J: SortedDisjointIterator<Item = Self::Item>,
    {
        that
    }
}

// This implements a trait called SortedDisjointIterator
// for any type I that satisfies two conditions: it is an Iterator and it is SortedDisjoint.
// This lets us use the methods defined in the SortedDisjointIterator trait on any type that satisfies these conditions.
// i.e. Iterator + SortedDisjoint isa SortedDisjointIterator
impl<I: Iterator + SortedDisjoint> SortedDisjointIterator for I {}

// A container for an iterator that is assumed to be sorted disjoint.
#[derive(Clone, Debug)]
pub struct AssumeSortedDisjoint<I: Iterator> {
    i: I, // pub(crate) cmk0 put back?
}

// AssumeSortedDisjoint<I> isa SortedDisjoint
impl<I: Iterator> SortedDisjoint for AssumeSortedDisjoint<I> {}

/// Define a method called 'assume_sorted_disjoint' that is Iterator + Sized
pub trait AssumeSortedDisjointExt: Iterator + Sized {
    fn assume_sorted_disjoint(self) -> AssumeSortedDisjoint<Self> {
        AssumeSortedDisjoint { i: self }
    }
}
impl<I: Iterator + Sized> AssumeSortedDisjointExt for I {}

// Implement the Iterator trait for AssumeSortedDisjoint<I>
impl<I: Iterator> Iterator for AssumeSortedDisjoint<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.i.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.i.size_hint()
    }
}

pub fn union<II, I>(iters: II) -> I
where
    II: IntoIterator<Item = I>,
    I: SortedDisjointIterator,
{
    let first = iters.into_iter().next().unwrap();
    println!("union");
    first
}

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

    fn add_dyn<'a>(input: impl Iterator<Item = u64> + 'a) -> Box<dyn Iterator<Item = u64> + 'a> {
        Box::new(input)
    }

    // trait SortedIterator: Iterator + SortedByItem {}

    fn fibs_new() -> Box<dyn Iterator<Item = u64>> {
        Box::new([1, 2, 3, 5, 8, 13u64].into_iter())
    }

    fn even_new() -> Box<dyn Iterator<Item = u64>> {
        Box::new((0..14).filter(|x| x % 2 == 0))
    }

    fn is_sorted(_: &dyn SortedByItem) {
        println!("yep");
    }

    fn is_sorted_disjoint(_: &dyn SortedDisjoint) {
        println!("yep");
    }

    // impl SortedDisjoint0 for Box<dyn SortedDisjoint0> {}

    // trait Sd: SortedByItem + Iterator<Item = u64> {
    //     fn another_next(&mut self) -> Option<u64>
    //     where
    //         Self: Sized,
    //     {
    //         self.next()
    //     }
    // }

    is_sorted_disjoint(
        &btreeset! { 2, 3, 5, 7, 11, 13u64 }
            .into_iter()
            .assume_sorted_disjoint(),
    );
    // is_sorted(&(0..14).filter(|x| x % 2 == 0));
    // is_sorted(&(0..14).rev().assume_sorted_by_item());
    // let p0 = primes_new().as_ref(); //.assume_sorted_by_item();
    // is_sorted(p0);

    // let evens0: Box<dyn Iterator<Item = u64>> =
    //     Box::new((0..14u64).filter(|x| x % 2 == 0)).assume_sorted_disjoint();
    // union0([evens0].into_iter());

    let primes = primes_new();
    let fibs0 = fibs_new();
    let fibs1 = fibs_new();
    let evens0 = even_new();
    let evens1: Box<dyn Iterator<Item = u64>> = Box::new((0..14u64).filter(|x| x % 2 == 0));
    let evens1 = evens1.assume_sorted_by_item();
    let evens2 = add_dyn((0..14u64).filter(|x| x % 2 == 0));
    let mw = multiway_union(
        [
            primes.assume_sorted_by_item(),
            fibs0.assume_sorted_by_item(),
            fibs1.assume_sorted_by_item(),
            evens0.assume_sorted_by_item(),
            evens1,
            evens2.assume_sorted_by_item(),
        ]
        .into_iter(),
    );
    println!("mw: {:?}", mw.collect::<Vec<_>>());
}

#[test]
fn test_s_d() {
    let fibs = btreeset! { 1, 2, 3, 5, 8, 13u64 }
        .into_iter()
        .assume_sorted_disjoint();
    let evens = (0..14).filter(|x| x % 2 == 0).assume_sorted_disjoint();
    let fib_evens = fibs.left(evens);
    println!("fib_evens: {:?}", fib_evens.collect::<Vec<_>>());

    let primes = btreeset! { 2, 3, 5, 7, 11, 13u64 }
        .into_iter()
        .assume_sorted_disjoint();
    let fibs = btreeset! { 1, 2, 3, 5, 8, 13u64 }
        .into_iter()
        .assume_sorted_disjoint();
    let evens = (0..14u64).filter(|x| x % 2 == 0).assume_sorted_disjoint();
    let u = union([primes, fibs].into_iter());
    println!("union: {:?}", u.collect::<Vec<_>>());

    //     fn primes_new() -> Box<dyn Iterator<Item = u64>> {
    //         Box::new(btreeset! { 2, 3, 5, 7, 11, 13u64 }.into_iter())
    //     }

    //     fn add_dyn<'a>(input: impl Iterator<Item = u64> + 'a) -> Box<dyn Iterator<Item = u64> + 'a> {
    //         Box::new(input)
    //     }

    //     // trait SortedIterator: Iterator + SortedByItem {}

    //     fn fibs_new() -> Box<dyn Iterator<Item = u64>> {
    //         Box::new([1, 2, 3, 5, 8, 13u64].into_iter())
    //     }

    //     fn even_new() -> Box<dyn Iterator<Item = u64>> {
    //         Box::new((0..14).filter(|x| x % 2 == 0))
    //     }

    //     fn is_sorted(_: &dyn SortedByItem) {
    //         println!("yep");
    //     }

    //     fn is_sorted_disjoint(_: &dyn SortedDisjoint) {
    //         println!("yep");
    //     }

    //     // impl SortedDisjoint0 for Box<dyn SortedDisjoint0> {}

    //     // trait Sd: SortedByItem + Iterator<Item = u64> {
    //     //     fn another_next(&mut self) -> Option<u64>
    //     //     where
    //     //         Self: Sized,
    //     //     {
    //     //         self.next()
    //     //     }
    //     // }

    //     is_sorted_disjoint(
    //         &btreeset! { 2, 3, 5, 7, 11, 13u64 }
    //             .into_iter()
    //             .assume_sorted_disjoint(),
    //     );
    //     // is_sorted(&(0..14).filter(|x| x % 2 == 0));
    //     // is_sorted(&(0..14).rev().assume_sorted_by_item());
    //     // let p0 = primes_new().as_ref(); //.assume_sorted_by_item();
    //     // is_sorted(p0);

    //     pub fn union0<II, I>(iters: II)
    //     where
    //         II: IntoIterator<Item = I>,
    //         I: SortedDisjointIterator,
    //     {
    //         let _first = iters.into_iter().next().unwrap();
    //         // Box::new(first)
    //     }

    //     // let evens0: Box<dyn Iterator<Item = u64>> =
    //     //     Box::new((0..14u64).filter(|x| x % 2 == 0)).assume_sorted_disjoint();
    //     // union0([evens0].into_iter());

    //     let primes = primes_new();
    //     let fibs0 = fibs_new();
    //     let fibs1 = fibs_new();
    //     let evens0 = even_new();
    //     let evens1: Box<dyn Iterator<Item = u64>> = Box::new((0..14u64).filter(|x| x % 2 == 0));
    //     let evens1 = evens1.assume_sorted_by_item();
    //     let evens2 = add_dyn((0..14u64).filter(|x| x % 2 == 0));
    //     let mw = multiway_union(
    //         [
    //             primes.assume_sorted_by_item(),
    //             fibs0.assume_sorted_by_item(),
    //             fibs1.assume_sorted_by_item(),
    //             evens0.assume_sorted_by_item(),
    //             evens1,
    //             evens2.assume_sorted_by_item(),
    //         ]
    //         .into_iter(),
    //     );
    //     println!("mw: {:?}", mw.collect::<Vec<_>>());
    // }
}
