#[cfg(not(target_arch = "wasm32"))]
mod native {
    use glob::glob;
    use range_set_blaze::prelude::*;
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    pub fn inner() -> Result<(), Box<dyn Error>> {
        let mut all_exps = RangeSetBlaze::from_iter([0..=99_999_999]);

        for path in glob("examples/cluster_file.*.tsv")? {
            let exp_nums: RangeSetBlaze<_> = BufReader::new(File::open(path?)?)
                .lines()
                .filter_map(|line| line.ok()?.split_once('\t')?.0.parse::<u32>().ok())
                .collect();
            all_exps = all_exps - exp_nums;
        }
        println!("{all_exps}");
        Ok(())
    }
}
#[cfg(target_arch = "wasm32")]
mod wasm {
    // Code here will only be compiled when the target architecture is WASM.
    pub fn inner() {}
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    native::inner().unwrap();
    #[cfg(target_arch = "wasm32")]
    wasm::inner();
}
