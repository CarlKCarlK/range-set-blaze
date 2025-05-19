//! Examples with `Ipv4Addr` and `RangeMapBlaze` in the context of routing.
// cmk000

use range_set_blaze::prelude::*;
use std::net::Ipv4Addr;
use std::ops::RangeInclusive;

#[allow(clippy::type_complexity)]
fn sample1() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;

    let mut pair_vec: Vec<(RangeInclusive<Ipv4Addr>, (Ipv4Addr, String, u32, u32))> = Vec::new();
    for line in std::io::BufRead::lines(std::io::BufReader::new(File::open(
        "./examples/routing_ipv4.tsv",
    )?))
    .skip(1)
    {
        let line = line?;
        // println!("{}", line);
        let fields: Vec<&str> = line.split(&['\t', '/']).collect();
        if fields.len() != 5 {
            return Err("Expected 5 fields".into());
        }

        let destination: Ipv4Addr = fields[0].parse()?;
        let prefix_len: u32 = fields[1].parse()?;
        let next_hop: Ipv4Addr = fields[2].parse()?;
        let interface: &str = fields[3];
        let metric: u32 = fields[4].parse()?;

        let mask = u32::MAX >> prefix_len;
        let range_start = Ipv4Addr::from(u32::from(destination) & !mask);
        let range_end = Ipv4Addr::from(u32::from(destination) | mask);
        let range = range_start..=range_end;

        pair_vec.push((range, (next_hop, interface.to_string(), prefix_len, metric)));
    }

    // Sort by prefix length then by metric (longer prefix & lower metric last)
    pair_vec.sort_by(
        |(_, (_, _, prefix_len_a, metric_a)), (_, (_, _, prefix_len_b, metric_b))| {
            prefix_len_a.cmp(prefix_len_b).then(metric_a.cmp(metric_b))
        },
    );

    let range_map = RangeMapBlaze::from_iter(pair_vec);
    for (range, (next_hop, interface, _, _)) in range_map.range_values() {
        println!("{range:?} → ({next_hop}, {interface})");
    }

    Ok(())
}

#[allow(clippy::unwrap_used)]
fn sample2() {
    use range_set_blaze::prelude::*;
    use std::net::Ipv4Addr;

    // A routing table, sorted by prefix length (so, highest priority last)
    let routing = [
        // destination, prefix, next hop, interface
        ("0.0.0.0", 0, "152.10.0.0", "eth0"),
        ("10.0.0.0", 8, "10.3.4.2", "eth1"),
        ("10.0.1.12", 30, "10.1.1.0", "eth2"),
        ("10.0.1.8", 30, "10.1.1.0", "eth2"),
        ("10.0.1.7", 32, "10.1.1.0", "eth2"),
    ];

    // Create a RangeMapBlaze from the routing table
    let range_map = routing
        .iter()
        .map(|(dest, prefix_len, next_hop, interface)| {
            let dest: Ipv4Addr = dest.parse().unwrap();
            let next_hop: Ipv4Addr = next_hop.parse().unwrap();
            let mask = u32::MAX.checked_shr(*prefix_len).unwrap_or(0);
            let range_start = Ipv4Addr::from(u32::from(dest) & !mask);
            let range_end = Ipv4Addr::from(u32::from(dest) | mask);
            (range_start..=range_end, (next_hop, interface))
        })
        .collect::<RangeMapBlaze<_, _>>();

    // Print the now disjoint, sorted ranges and their associated values
    for (range, (next_hop, interface)) in range_map.range_values() {
        println!("{range:?} → ({next_hop}, {interface})");
    }

    // Look up an address
    assert_eq!(
        range_map.get(Ipv4Addr::new(10, 0, 1, 6)),
        Some(&(Ipv4Addr::new(10, 3, 4, 2), &"eth1"))
    );
}

fn main() {
    println!("Sample 1");
    sample1().expect("Failed to run sample1");
    println!("Sample 2");
    sample2();
}
