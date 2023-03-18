// !!!cmk can you optimize a | b | c to automatically call union([a,b,c])?
use std::{collections::BTreeSet, ops::BitOr};

use range_set_int::{
    union_dyn, DynSortedDisjoint, MultiwaySortedDisjoint, RangeSetInt, SortedDisjointIterator,
};

#[test]
fn b_tree_set() {
    let a = [1, 2, 3].into_iter().collect::<BTreeSet<i32>>();
    let b = BTreeSet::from([2, 3, 4]);
    let mut c3 = a.clone();
    let mut c4 = a.clone();
    let mut c5 = a.clone();

    let c0 = a.bitor(&b);
    let c1 = &a | &b;
    let c2 = BTreeSet::from_iter(a.union(&b).copied());
    c3.append(&mut b.clone());
    c4.extend(&b);
    c5.extend(b);

    let answer = BTreeSet::from([1, 2, 3, 4]);
    assert_eq!(&c0, &answer);
    assert_eq!(&c1, &answer);
    assert_eq!(&c2, &answer);
    assert_eq!(&c3, &answer);
    assert_eq!(&c4, &answer);
    assert_eq!(&c5, &answer);
}

#[test]
fn range_set_int() {
    let a = [1, 2, 3].into_iter().collect::<RangeSetInt<i32>>();
    let b = RangeSetInt::from_iter([2, 3, 4]);
    let b2 = 2;
    let b3 = 3;
    let b4 = 4;
    let b_ref = [&b2, &b3, &b4];
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

    let answer = RangeSetInt::from_iter([1, 2, 3, 4]);
    assert_eq!(&c0, &answer);
    assert_eq!(&c1a, &answer);
    assert_eq!(&c1b, &answer);
    assert_eq!(&c1c, &answer);
    assert_eq!(&c1d, &answer);
    assert_eq!(&c2, &answer);
    assert_eq!(&c3, &answer);
    assert_eq!(&c4, &answer);
    assert_eq!(&c5, &answer);
}

#[test]
fn sorted_disjoint() {
    let a = [1, 2, 3].into_iter().collect::<RangeSetInt<i32>>();
    let b = RangeSetInt::from_iter([2, 3, 4]);

    let c0 = a.ranges() | b.ranges();
    let c1 = [a.ranges(), b.ranges()].union();
    let c2 = [a.ranges(), b.ranges()].union();
    let c3 = union_dyn!(a.ranges(), b.ranges());
    let c4 = [a.ranges(), b.ranges()].map(DynSortedDisjoint::new).union();

    let answer = RangeSetInt::from_iter([1, 2, 3, 4]);
    assert!(c0.equal(answer.ranges()));
    assert!(c1.equal(answer.ranges()));
    assert!(c2.equal(answer.ranges()));
    assert!(c3.equal(answer.ranges()));
    assert!(c4.equal(answer.ranges()));
}

#[test]
fn sorted_disjoint_ops() {
    let a = [1, 2, 3].into_iter().collect::<RangeSetInt<i32>>();
    let a = a.ranges();
    let b = !a.clone();
    let _c = !!b.clone();
    let _d = a.clone() | b.clone();
    let _e = !a.clone() | b.clone();
    let _f = !(!a.clone() | !b.clone());
    let _g = BitOr::bitor(a.clone().not(), b.clone().not()).not();
    let _h = SortedDisjointIterator::union(a.clone().not(), b.clone().not()).not();
    let _z = !(!a | !b);
}
