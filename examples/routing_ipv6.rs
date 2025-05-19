//! Examples with `Ipv6Addr` and `RangeMapBlaze` in the context of routing.
// cmk000

use std::fs::File;
use std::io::{self, BufRead};
use std::net::Ipv6Addr;
use std::ops::RangeInclusive;

use range_set_blaze::RangeMapBlaze;

#[allow(clippy::type_complexity)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pair_vec: Vec<(RangeInclusive<Ipv6Addr>, (Ipv6Addr, String, u32, u32))> = Vec::new();

    for line in BufRead::lines(io::BufReader::new(File::open(
        "./examples/routing_ipv6.tsv",
    )?))
    .skip(1)
    {
        let line = line?;
        let fields: Vec<&str> = line.split(&['\t', '/']).collect();
        if fields.len() != 5 {
            return Err("Expected 5 fields".into());
        }

        let destination: Ipv6Addr = fields[0].parse()?;
        let prefix_len: u32 = fields[1].parse()?;
        let next_hop: Ipv6Addr = fields[2].parse()?;
        let interface: &str = fields[3];
        let metric: u32 = fields[4].parse()?;

        // Calculate the range start and end for the given destination and prefix length
        let mask = u128::MAX >> prefix_len;
        let range_start = Ipv6Addr::from(u128::from(destination) & !mask);
        let range_end = Ipv6Addr::from(u128::from(destination) | mask);
        let range = range_start..=range_end;

        // Add the calculated range and associated next hop and interface to the vector
        pair_vec.push((range, (next_hop, interface.to_string(), prefix_len, metric)));
    }

    // Sort by prefix length then by metric (longer prefix & lower metric last)
    pair_vec.sort_by(
        |(_, (_, _, prefix_len_a, metric_a)), (_, (_, _, prefix_len_b, metric_b))| {
            prefix_len_a.cmp(prefix_len_b).then(metric_a.cmp(metric_b))
        },
    );

    // Create a RangeMapBlaze from the vector
    let range_map = RangeMapBlaze::from_iter(pair_vec);
    for (range, (next_hop, interface, _, _)) in range_map.range_values() {
        println!("{range:?} → ({next_hop}, {interface})");
    }

    Ok(())
}
