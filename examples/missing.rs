use glob::glob;
use range_set_blaze::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn main() {
    let mut all_exps = RangeSetBlaze::from_iter([0..=99_999_999]);

    // TODO: Is there a nicer Rust way to turn the first column of a set of files into an iterator?
    for path in glob("examples/cluster_file.*.tsv").unwrap() {
        let file = File::open(path.unwrap()).unwrap();

        let exp_nums: RangeSetBlaze<_> = BufReader::new(file)
            .lines()
            .skip(1)
            .map(|line| {
                line.unwrap()
                    .split('\t')
                    .next()
                    .unwrap()
                    .parse::<u32>()
                    .unwrap()
            })
            .collect();

        all_exps = all_exps - exp_nums;
    }
    println!("{all_exps}");
}
