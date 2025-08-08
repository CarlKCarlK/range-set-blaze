//! Example of working with `char` and `RangeMapBlaze` in the context of fonts

use anyhow::Error;
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

fn example_font_fallback() -> Result<(), anyhow::Error> {
    // Font Selection System Using RangeMapBlaze
    // Defining overlapping font ranges with higher priority last.
    let overlapping_font_table = [
        ('\u{0000}'..='\u{10FFFF}', "Default Font"), // covers all
        ('\u{0000}'..='\u{007F}', "Basic Latin"),
        ('\u{1F600}'..='\u{1F64F}', "Emoji Font"),
        ('\u{4E00}'..='\u{9FFF}', "Chinese Font"),
        ('\u{4E00}'..='\u{4FFF}', "Japanese Font"), // overwrites some Chinese
        ('\u{30A0}'..='\u{30FF}', "Japanese Font"),
        ('\u{3040}'..='\u{309F}', "Japanese Font"), // overwrites some previous
    ];
    let disjoint_font_table = RangeMapBlaze::from_iter(overlapping_font_table);

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

    // Check for any gaps.
    if !&disjoint_font_table.is_universal() {
        return Err(Error::msg("Font table contains gaps."));
    }

    println!("\n=== Font Selection for Sample Text ===");
    let text = "Hello, こんにちは, ∑, 😊";
    println!("Text: {text}");
    for c in text.chars() {
        let font = disjoint_font_table[c];
        println!("{c} → {font}");
    }
    Ok(())
}

fn main() {
    example_font_fallback().expect("Failed to run font example");
}
