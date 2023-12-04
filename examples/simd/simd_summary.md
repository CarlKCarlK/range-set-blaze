# Selected Rust `Simd` structs, Methods, etc

## Stucts

- [`Simd`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html) - a special, aligned, fixed-length array of [`SimdElement`](https://doc.rust-lang.org/std/simd/trait.SimdElement.html). We refer to a position in the array and the element stored at that position as a "lane".
- [`Mask`](https://doc.rust-lang.org/nightly/core/simd/struct.Mask.html) - a special boolean array showing inclusion/exclusion on a per-lane basis.

## SimdElements

- Floating-Point Types: `f32`, `f64`
- Integer Types: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `isize`, `usize`
- -- [*but not `i128`, `u128`*](https://medium.com/r/?url=https%3A%2F%2Fgithub.com%2Frust-lang%2Fportable-simd%2Fissues%2F108)

## `Simd` constructors

- [`Simd::from_array`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.from_array) - creates a `Simd` struct by copying a fixed-length array. By default, we copy `Simd` structs rather than reference them.
- [`Simd::from_slice`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.from_slice) - creates a `Simd` struct by copying the first LANE elements of a slice..
- [`Simd::splat`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.splat) - replicates a single value across all lanes of a `Simd` struct.
- [`slice::as_simd`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.to_simd) - without copying, converts a regular slice into an aligned slice of `Simd` (plus unaligned leftovers).

## `Simd` conversion

- [`Simd::as_array`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.as_array) - without copying, converts an `Simd` struct into a regular array reference.

## `Simd` methods and operators

- [`simd[i]`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.index) - extract a value from a lane of a `Simd`.
- [`simd + simd`](https://doc.rust-lang.org/core/simd/struct.Simd.html#impl-Add%3C%26'rhs+Simd%3CT,+LANES%3E%3E-for-%26'lhs+Simd%3CT,+LANES%3E) - performs element-wise addition of two `Simd` structs. Also, supported `-`, `*`, `/`, `%`, remainder, bitwise-and, -or, -xor, -not, -shift.
- [`simd += simd`](https://doc.rust-lang.org/core/simd/struct.Simd.html#impl-AddAssign%3CU%3E-for-Simd%3CT,+LANES%3E) - adds another `Simd` struct to the current one, in place. Other operators supported, too.
- [`Simd::simd_gt`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.simd_gt) - compares two `Simd` structs, returning a `Mask` indicating if elements of the first are greater than those of the second. Also, supported `simd_lt`, `simd_le`, `simd_ge`, `simd_lt`, `simd_eq`, `simd_ne`.
- [`Simd::rotate_lanes_left`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.rotate_lanes_left) - rotates the lanes of a `Simd` struct to the left by a specified amount. Also, `rotate_lanes_right`.
- [`simd_swizzle!(simd, indexes)`](https://doc.rust-lang.org/std/simd/prelude/macro.simd_swizzle.html) - rearranges the elements of a `Simd` struct based on specified indices.
- [`simd == simd`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#impl-Eq-for-Simd%3CT,+N%3E) - checks for equality between two `Simd` structs, returning a regular `bool` result.
- [`Simd::reduce_and`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.reduce_and) - performs a bitwise AND reduction across all lanes of a `Simd` struct. Also, supported `reduce_or`, `reduce_xor`, `reduce_max`, `reduce_min`, `reduce_sum`.

## `Mask` methods and operators

- [`Mask::select`](https://doc.rust-lang.org/nightly/core/simd/struct.Mask.html#method.select) - selects elements from two `Simd` structs based on a mask.
- [`Mask::all`](https://doc.rust-lang.org/nightly/core/simd/struct.Mask.html#method.all) - tells if the mask is all `true`.
- [`Mask::any`](https://doc.rust-lang.org/nightly/core/simd/struct.Mask.html#method.all) - tells if the mask contains any true.

## All about lanes

- [`Simd::LANES`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#associatedconstant.LANES) - a constant indicating the number of elements (lanes) in a `Simd` struct.
- [`SupportedLaneCount`](https://doc.rust-lang.org/nightly/core/simd/trait.SupportedLaneCount.html) - tells the allowed values of LANES. Use by generics.
- [`simd.lanes`](https://doc.rust-lang.org/core/simd/struct.Simd.html#method.lanes) - const method that tells a Simd struct's number of lanes.

## Low-Level Alignment, Offsets etc

*When possible, use [`to_simd`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.to_simd) instead.*

- [`mem::size_of`](https://doc.rust-lang.org/std/mem/fn.size_of.html), [`mem::align_of`](https://doc.rust-lang.org/std/mem/fn.align_of.html), [`mem::align_to`](https://doc.rust-lang.org/std/mem/fn.align_to.html), [`intrinsics::offset`](https://doc.rust-lang.org/std/intrinsics/fn.offset.html),
[`pointer::read_unaligned`](https://doc.rust-lang.org/std/primitive.pointer.html#method.read_unaligned) (unsafe),
[`pointer::write_unaligned`](https://doc.rust-lang.org/std/primitive.pointer.html#method.write_unaligned) (unsafe), [`mem::transmute`](https://doc.rust-lang.org/std/mem/fn.transmute.html) (unsafe, const)

## More, perhaps of interest

- [`deinterleave`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.deinterleave),
[`gather_or`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.gather_or),
[`reverse`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.reverse),
[`scatter`](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html#method.scatter)
