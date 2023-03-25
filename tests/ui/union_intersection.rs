use range_set_blaze::{intersection, union};

fn main() {
    let _u = union([[1..=3, 2..=4].into_iter()]);
    let _i = intersection([[1..=3, 2..=4].into_iter()]);
}
