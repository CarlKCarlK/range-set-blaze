//! Example of working with `char` and `RangeMapBlaze` in the context of fonts
// cmk000

use range_set_blaze::Integer;
use range_set_blaze::prelude::*;

#[allow(unused)]
fn unicode_analysis() {
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

fn sample1() {
    // Font Selection System Using RangeMapBlaze
    // Defining overlapping font ranges with priority order:");

    let overlapping_font_table = [
        ('\u{3040}'..='\u{309F}', "Japanese Font"),
        ('\u{30A0}'..='\u{30FF}', "Japanese Font"), // adjacent with prev
        ('\u{4E00}'..='\u{4FFF}', "Japanese Font"), // overlaps with Chinese
        ('\u{4E00}'..='\u{9FFF}', "Chinese Font"),
        ('\u{1F600}'..='\u{1F64F}', "Emoji Font"),
        ('\u{0000}'..='\u{007F}', "Basic Latin"),
        ('\u{0000}'..='\u{10FFFF}', "Default Font"), // covers all
    ]; // Reverse the order so that higher-priority ranges come last,
    // allowing them to override lower-priority ones in case of overlap.
    let disjoint_font_table = overlapping_font_table
        .into_iter()
        .rev() // Put higher priority ranges LAST
        .collect::<RangeMapBlaze<_, _>>();

    println!("\n=== Optimized Font Table (after merging and prioritizing) ===");
    for (range, font) in disjoint_font_table.range_values() {
        let (start, end) = range.into_inner();
        println!(
            "U+{:X}[{}]..=U+{:X}[{}] → {}",
            u32::from(start),
            start,
            u32::from(end),
            end,
            font
        );
    }

    println!("\n=== Font Selection for Sample Text ===");
    let text = "Hello, こんにちは, ∑, 😊";
    println!("Text: {text}");
    for c in text.chars() {
        let font = disjoint_font_table.get(c).unwrap_or(&"**MISSING**");
        println!("{c} → {font}");
    }
}

fn main() {
    sample1();
}
