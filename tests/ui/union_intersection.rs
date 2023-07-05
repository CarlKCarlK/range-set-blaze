use range_set_blaze::SortedDisjoint;

fn main() {
    let _u = SortedDisjoint::union([[1..=3, 2..=4].into_iter()]);
    let _i = SortedDisjoint::intersection([[1..=3, 2..=4].into_iter()]);
}
