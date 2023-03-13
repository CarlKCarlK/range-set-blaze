rangeset-int
==========

cmk
[![github](https://img.shields.io/badge/github-anyinput-8da0cb?style=flat&labelColor=555555&logo=github)](https://github.com/CarlKCarlK/anyinput)
[![crates.io](https://img.shields.io/crates/v/anyinput.svg?flat&color=fc8d62&logo=rust")](https://crates.io/crates/anyinput)
[![docs.rs](https://img.shields.io/badge/docs.rs-anyinput-66c2a5?flat&labelColor=555555&logoColor=white&logo=core:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/anyinput)[![CI](https://github.com/CarlKCarlK/anyinput/actions/workflows/ci.yml/badge.svg)](https://github.com/CarlKCarlK/anyinput/actions/workflows/ci.yml)

A crate for efficiently manipulating ranges of integers (including negatives and up to u128) using set operations such as `union()`, `intersection()`, and `difference()`.

The crate differs from sets in the standard library (such as `BTreeSet` and `HashSet`) because it does not need to store every element in the set, only for every contiguous range of elements. It differs from other interval libraries (that we know of) by being specialized and optimized for integer elements.

Example 1
---------

Here we take the union (operator “|”) of two RangeSetInt's:

cmkpicturegoes here

```rust
use rangeset_int::RangeSetInt;

let a = RangeSetInt::from([100..=499,501..=999]); // a is the set of integers from 100 to 499 (inclusive) and 501 to 1000 (inclusive)
let b = RangeSetInt::from([-20..=-20,400..=599]); // b is the set of integers -20 and the range 400 to 599 (inclusive)
let c = a | b;                           // c is the union of a and b, namely -20 and 100 to 999 (inclusive)
assert_eq!(c, RangeSetInt::from([-20..=-20,100..=999]));
```

Example 2
---------

In biology, suppose we want to find the intron regions of a gene but we are given only the transcription region and the exon regions.

cmkpicturegoes here

```rust
use rangeset_int::RangeSetInt;

let line = "chr15   29370   37380   29370,32358,36715   30817,32561,37380";
// split the line on white space
let mut iter = line.split_whitespace();
let _chr = iter.next().unwrap();
let trans_start: i32 = iter.next().unwrap().parse()?;
let trans_end: i32 = iter.next().unwrap().parse()?;
// creates a RangeSetInt from 29370 (inclusive) to 37380 (inclusive)
let range_set_int = RangeSetInt::from([trans_start..=trans_end]);
assert_eq!(range_set_int, RangeSetInt::from([29370..=37380]));
# Ok(())
```

As we add nesting and multiple inputs, the macro becomes more useful.
Here we create a function with two inputs. One input accepts any iterator-like
thing of `usize`. The second input accepts any iterator-like thing of string-like things. The function returns the sum of the numbers and string lengths.

We apply the function to the range `1..=10` and a slice of `&str`'s.

```rust
use anyinput::anyinput;

#[anyinput]
fn two_iterator_sum(iter1: AnyIter<usize>, iter2: AnyIter<AnyString>) -> usize {
    let mut sum = iter1.sum();
    for any_string in iter2 {
        // Needs .as_ref to turn the nested AnyString into a &str.
        sum += any_string.as_ref().len();
    }
    sum
}

assert_eq!(two_iterator_sum(1..=10, ["a", "bb", "ccc"]), 61);
```

Create a function that accepts an array-like thing of path-like things.
Return the number of path components at an index.

```rust
use anyinput::anyinput;
use anyhow::Result;

#[anyinput]
fn indexed_component_count(
    array: AnyArray<AnyPath>,
    index: usize,
) -> Result<usize, anyhow::Error> {
    // Needs .as_ref to turn the nested AnyPath into a &Path.
    let path = array[index].as_ref();
    let count = path.iter().count();
    Ok(count)
}

assert_eq!(
    indexed_component_count(vec!["usr/files/home", "usr/data"], 1)?,
    2
);
# // '# OK...' needed for doctest
# Ok::<(), anyhow::Error>(())
```

You can easily apply `NdArray` functions to any array-like thing of numbers. For example,
here we create  a function that accepts an `NdArray`-like thing of `f32` and returns the mean.
We apply the function to both a `Vec` and an `Array1<f32>`.

Support for `NdArray` is provided by the optional feature `ndarray`.

```rust
use anyinput::anyinput;
use anyhow::Result;

# // '#[cfg...' needed for doctest
# #[cfg(feature = "ndarray")]
#[anyinput]
fn any_mean(array: AnyNdArray<f32>) -> Result<f32, anyhow::Error> {
    if let Some(mean) = array.mean() {
        Ok(mean)
    } else {
        Err(anyhow::anyhow!("empty array"))
    }
}

// 'AnyNdArray' works with any 1-D array-like thing, but must be borrowed.
# #[cfg(feature = "ndarray")]
assert_eq!(any_mean(&vec![10.0, 20.0, 30.0, 40.0])?, 25.0);
# #[cfg(feature = "ndarray")]
assert_eq!(any_mean(&ndarray::array![10.0, 20.0, 30.0, 40.0])?, 25.0);
# // '# OK...' needed for doctest
# Ok::<(), anyhow::Error>(())
```

The AnyInputs
---------

| AnyInput   | Description                            | Creates Concrete Type           |
| ---------- | -------------------------------------- | ------------------------------- |
| AnyString  | Any string-like thing                  | `&str`                          |
| AnyPath    | Any path-like or string-like thing     | `&Path`                         |
| AnyIter    | Any iterator-like thing                | `<I as IntoIterator>::IntoIter` |
| AnyArray   | Any array-like thing                   | `&[T]`                          |
| AnyNdArray | Any 1-D array-like thing (borrow-only) | `ndarray::ArrayView1<T>`        |

Notes & Features
--------

- Suggestions, feature requests, and contributions are welcome.
- Works with nesting, multiple inputs, and generics.
- Automatically and efficiently converts an top-level AnyInput into a concrete type.
- Elements of AnyArray, AnyIter, and AnyNdArray must be a single type. So, `AnyArray<AnyString>`
  accepts a vector of all `&str` or all `String`, but not mixed.
- When nesting, efficiently convert the nested AnyInput to the concrete type with
  - `.as_ref()` -- AnyString, AnyPath, AnyArray
  - `.into_iter()` -- AnyIter
  - `.into()` -- AnyNdArray

  (The iterator and array examples above show this.)

- Let's you easily apply `NdArray` functions to regular Rust arrays, slices, and `Vec`s.
- Used by [bed-reader](https://docs.rs/bed-reader/latest/bed_reader/) (genomics crate) and [fetch-data](https://crates.io/crates/fetch-data) (sample-file download crate).

How It Works
--------

The `#[anyinput]` macro uses standard Rust generics to support multiple input types. To do this, it
 rewrites your function with the appropriate generics. It also adds lines to your function to efficiently convert from any top-level generic to a concrete type. For example, the macro transforms `len_plus_2` from:

```rust
use anyinput::anyinput;

#[anyinput]
fn len_plus_2(s: AnyString) -> usize {
    s.len()+2
}
```

into

```rust
fn len_plus_2<AnyString0: AsRef<str>>(s: AnyString0) -> usize {
    let s = s.as_ref();
    s.len() + 2
}
```

Here `AnyString0` is the generic type. The line `let s = s.as_ref()` converts from generic type `AnyString0` to concrete type `&str`.

As with all Rust generics, the compiler creates a separate function for each combination of concrete types used by the calling code.

Project Links
-----

- [**Installation**](https://crates.io/crates/anyinput)
- [**Documentation**](https://docs.rs/anyinput/)
- [**Source code**](https://github.com/CarlKCarlK/anyinput)
- [**Discussion**](https://github.com/CarlKCarlK/anyinput/discussions/)
- [**Bug Reports and Feature Requests**](https://github.com/CarlKCarlK/anyinput/issues)
