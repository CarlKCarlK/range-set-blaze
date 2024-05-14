use std::fs::File;
use std::io::{self, BufRead};
use std::net::Ipv4Addr;
use std::ops::RangeInclusive;

use range_set_blaze::RangeMapBlaze;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Until Rust get's 'yield' keyword, to keep things simple, we'll use a Vec
    let mut pair_vec: Vec<(RangeInclusive<u32>, (u32, String))> = Vec::new();

    let mut prev_prefix_len: Option<u32> = None;
    let mut prev_metric: Option<u32> = None;
    for line in BufRead::lines(io::BufReader::new(File::open(
        "./examples/routing_ip4.tsv",
    )?))
    .skip(1)
    {
        let line = line?;
        println!("{}", line);
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 5, "Expected 5 fields");

        // Until Rust gets the Step trait, we'll use u32 for Ipv4Addr
        let destination: u32 = fields[0].parse::<Ipv4Addr>()?.into();
        let prefix_len: u32 = fields[1].parse()?;
        let next_hop: u32 = fields[2].parse::<Ipv4Addr>()?.into();
        let interface: &str = fields[3];
        let metric: u32 = fields[4].parse()?;

        if let Some(prev_prefix_len) = prev_prefix_len.replace(prefix_len) {
            if prev_prefix_len != prefix_len {
                assert!(prev_prefix_len > prefix_len, "Sort by prefix length (desc)");
                prev_metric = None;
            }
        }

        if let Some(prev_metric) = prev_metric.replace(metric) {
            assert!(prev_metric <= metric, "Sort by prefix len & metric (asc)");
        }

        let range_start = destination & !(u32::MAX >> prefix_len);
        let range_end = range_start | (u32::MAX >> prefix_len);
        let range = range_start..=range_end;

        pair_vec.push((range, (next_hop, interface.to_string())));
    }

    let range_map = RangeMapBlaze::from_iter(pair_vec);
    for (range, (next_hop, interface)) in range_map.range_values() {
        let (start, end) = range.into_inner();
        println!(
            "{:?}..={:?} -> ({}, {})",
            Ipv4Addr::from(start),
            Ipv4Addr::from(end),
            Ipv4Addr::from(*next_hop),
            interface
        );
    }

    Ok(())
}
