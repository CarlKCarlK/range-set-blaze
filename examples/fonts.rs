//! Example of working with `char` and `RangeMapBlaze` in the context of fonts

use range_set_blaze::Integer;
use range_set_blaze::prelude::*;

fn sample2() {
    let text = include_str!("linear_algebra.jp.txt");
    let chars_used = text.chars().collect::<RangeSetBlaze<char>>();
    for range in chars_used.ranges() {
        let (start, end) = range.clone().into_inner();
        println!(
            "U+{:X}[{}]..=U+{:X}[{}], len {}",
            u32::from(start),
            start,
            u32::from(end),
            end,
            char::safe_len(&range)
        );
    }
}

#[allow(deprecated)]
fn sample1() {
    let overlapping_font_table = [
        ('\u{3040}'..='\u{309F}', "Japanese Font"),
        ('\u{30A0}'..='\u{30FF}', "Japanese Font"), // adjacent with prev
        ('\u{4E00}'..='\u{4FFF}', "Japanese Font"), // overlaps with Chinese
        ('\u{4E00}'..='\u{9FFF}', "Chinese Font"),
        ('\u{1F600}'..='\u{1F64F}', "Emoji Font"),
        ('\u{0000}'..='\u{007F}', "Basic Latin"),
        ('\u{0000}'..='\u{10FFFF}', "Default Font"), // covers all
    ];

    let disjoint_font_table = RangeMapBlaze::from_iter(overlapping_font_table);
    for (range, font) in disjoint_font_table.range_values() {
        let (start, end) = range.into_inner();
        println!(
            "U+{:X}[{}]..=U+{:X}[{}] -> {}",
            u32::from(start),
            start,
            u32::from(end),
            end,
            font
        );
    }
    println!("-----");

    let text = "Hello, こんにちは, ∑, 😊";
    for c in text.chars() {
        let font = disjoint_font_table.get(c).unwrap_or(&"**MISSING**");
        println!("{c} -> {font}");
    }
}

fn main() {
    sample1();
    sample2();
}
