use std::array;

const LANES: usize = 16;
fn is_consecutive_regular(slice: &[u32; LANES]) -> bool {
    for i in 1..LANES {
        if slice[i - 1] == u32::MAX || slice[i - 1] + 1 != slice[i] {
            return false;
        }
    }
    true
}

#[test]
fn test_is_consecutive() {
    let a: [u32; LANES] = array::from_fn(|i| 100 + i as u32);
    assert!(is_consecutive_regular(&a));
    assert!(!is_consecutive_regular(&[99; LANES]));
}

fn main() {
    let a = array::from_fn(|i| 100 + i as u32);
    println!(
        "is_consecutive_regular({:?}) = {}",
        a,
        is_consecutive_regular(&a)
    );
}
