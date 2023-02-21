use std::{collections::BTreeSet, ops::BitOr};

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
