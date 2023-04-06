use csv::ReaderBuilder;
use glob::glob;
use range_set_blaze::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut all_exps = RangeSetBlaze::from_iter([0..=99_999_999]);

    for path in glob("examples/cluster_file.*.tsv")? {
        let exp_nums: RangeSetBlaze<_> = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(path?)?
            .into_records()
            .filter_map(|line| line.ok()?.get(0)?.parse::<u32>().ok())
            .collect();
        all_exps = all_exps - exp_nums;
    }
    println!("{all_exps}");
    Ok(())
}
