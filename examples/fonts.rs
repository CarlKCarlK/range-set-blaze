use range_set_blaze::prelude::*;
use std::collections::BTreeSet;

fn main() {
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
            "U+{:X}{}..=U+{:X}{} -> {}",
            start as u32, start, end as u32, end, font
        );
    }

    let text = "Hello, こんにちは, ∑, 😊";
    let chars_used = text.chars().collect::<RangeSetBlaze<char>>();
    println!("Chars used: {:?}", chars_used);
    let fonts_used = disjoint_font_table
        .intersection_with_set(&chars_used)
        .range_values()
        .map(|(_, font)| *font)
        .collect::<BTreeSet<&str>>();
    println!("Fonts used: {:?}", fonts_used);
}
