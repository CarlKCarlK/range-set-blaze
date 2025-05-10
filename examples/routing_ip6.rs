//! cmk000

use std::fs::File;
use std::io::{self, BufRead};
use std::net::Ipv6Addr;
use std::ops::RangeInclusive;

use range_set_blaze::RangeMapBlaze;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Until Rust gets the 'yield' keyword, to keep things simple, we'll use a Vec
    let mut pair_vec: Vec<(RangeInclusive<Ipv6Addr>, (Ipv6Addr, String))> = Vec::new();

    let mut prev_prefix_len: Option<u32> = None;
    let mut prev_metric: Option<u32> = None;
    for line in BufRead::lines(io::BufReader::new(File::open(
        "./examples/routing_ip6.tsv",
    )?))
    .skip(1)
    {
        let line = line?;
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err("Expected 5 fields".into());
        }

        let destination: Ipv6Addr = fields[0].parse()?;
        let prefix_len: u32 = fields[1].parse()?;
        let next_hop: Ipv6Addr = fields[2].parse()?;
        let interface: &str = fields[3];
        let metric: u32 = fields[4].parse()?;

        // Ensure the entries are sorted by prefix length (descending) and metric (ascending)
        if let Some(prev_prefix_len) = prev_prefix_len.replace(prefix_len) {
            if prev_prefix_len != prefix_len {
                if prev_prefix_len <= prefix_len {
                    return Err("Sort by prefix length (desc)".into());
                }
                prev_metric = None;
            }
        }

        if let Some(prev_metric) = prev_metric.replace(metric) {
            if prev_metric > metric {
                return Err("Sort by prefix len & metric (asc)".into());
            }
        }

        // Calculate the range start and end for the given destination and prefix length
        let mask = u128::MAX >> prefix_len;
        let range_start = Ipv6Addr::from(u128::from(destination) & !mask);
        let range_end = Ipv6Addr::from(u128::from(destination) | mask);
        let range = range_start..=range_end;

        // Add the calculated range and associated next hop and interface to the vector
        pair_vec.push((range, (next_hop, interface.to_string())));
    }

    // Create a RangeMapBlaze from the vector
    let range_map = RangeMapBlaze::from_iter(pair_vec);
    for (range, (next_hop, interface)) in range_map.range_values() {
        println!("{range:?} -> ({next_hop}, {interface})");
    }

    Ok(())
}
