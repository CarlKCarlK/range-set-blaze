// use range_set_blaze::prelude::*;
// use std::net::Ipv4Addr;

// fn sample1() -> Result<(), Box<dyn std::error::Error>> {
//     use std::fs::File;
//     use std::io::BufReader;

//     // Until Rust get's 'yield' keyword, to keep things simple, we'll use a Vec
//     let mut pair_vec: Vec<(RangeInclusive<Ipv4Addr>, (Ipv4Addr, String))> = Vec::new();

//     let mut prev_prefix_len: Option<u32> = None;
//     let mut prev_metric: Option<u32> = None;
//     for line in BufRead::lines(io::BufReader::new(File::open(
//         "./examples/routing_ip4.tsv",
//     )?))
//     .skip(1)
//     {
//         let line = line?;
//         // println!("{}", line);
//         let fields: Vec<&str> = line.split('\t').collect();
//         assert_eq!(fields.len(), 5, "Expected 5 fields");

//         let destination: Ipv4Addr = fields[0].parse()?;
//         let prefix_len: u32 = fields[1].parse()?;
//         let next_hop: Ipv4Addr = fields[2].parse()?;
//         let interface: &str = fields[3];
//         let metric: u32 = fields[4].parse()?;

//         if let Some(prev_prefix_len) = prev_prefix_len.replace(prefix_len) {
//             if prev_prefix_len != prefix_len {
//                 assert!(prev_prefix_len > prefix_len, "Sort by prefix length (desc)");
//                 prev_metric = None;
//             }
//         }

//         if let Some(prev_metric) = prev_metric.replace(metric) {
//             assert!(prev_metric <= metric, "Sort by prefix len & metric (asc)");
//         }

//         let mask = u32::MAX >> prefix_len;
//         let range_start = Ipv4Addr::from(u32::from(destination) & !mask);
//         let range_end = Ipv4Addr::from(u32::from(destination) | mask);
//         let range = range_start..=range_end;

//         pair_vec.push((range, (next_hop, interface.to_string())));
//     }

//     let range_map = RangeMapBlaze::from_iter(pair_vec);
//     for (range, (next_hop, interface)) in range_map.range_values() {
//         println!("{range:?} -> ({next_hop}, {interface})");
//     }

//     Ok(())
// }

fn sample2() {
    use range_set_blaze::prelude::*;
    use std::net::Ipv4Addr;

    // A routing table, sorted by priority
    let routing = [
        ("10.0.1.8", 30, "10.1.1.0", "eth2"),
        ("10.0.1.12", 30, "10.1.1.0", "eth2"),
        ("10.0.1.7", 32, "10.1.1.0", "eth2"),
        ("10.0.0.0", 8, "10.3.4.2", "eth1"),
        ("0.0.0.0", 0, "152.10.0.0", "eth0"),
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

    // Print the disjoint, sorted ranges and their associated values
    for (range, (next_hop, interface)) in range_map.range_values() {
        println!("{range:?} -> ({next_hop}, {interface})");
    }

    // Look up an address
    assert_eq!(
        range_map.get(Ipv4Addr::new(10, 0, 1, 6)),
        Some(&(Ipv4Addr::new(10, 3, 4, 2), &"eth1"))
    );
}

fn main() {
    sample2();
}
