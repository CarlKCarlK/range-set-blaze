// use std::path::Path;

// use range_set_blaze::RangeSetBlaze;
//use tests_common::read_roaring_data;

fn main() -> std::io::Result<()> {
    // let top = Path::new(r"M:\projects\roaring_data");
    // let name_and_vec_vec_list = read_roaring_data(top)?;

    // println!("name, value_count, unique_count, range_count");
    // for (name, vec_vec) in name_and_vec_vec_list.iter() {
    //     let vec = vec_vec
    //         .iter()
    //         .flat_map(|v| v.iter().cloned())
    //         .collect::<Vec<u32>>();
    //     let value_count = vec.len();
    //     let range_set_blaze = vec.iter().collect::<RangeSetBlaze<_>>();
    //     let unique_count = range_set_blaze.len();
    //     let range_count = range_set_blaze.ranges_len();
    //     let min = range_set_blaze.first().unwrap();
    //     let max = range_set_blaze.last().unwrap();
    //     println!("{name}, {value_count}, {unique_count}, {range_count}, {min}, {max}");
    //     // if range_count < 5 {
    //     //     println!("    {:?}", range_set_blaze);
    //     // }
    // }

    // println!("name, value_count, unique_count, range_count");
    // for (name, vec_vec) in name_and_vec_vec_list.iter() {
    //     if name != "census1881_srt" {
    //         continue;
    //     }
    //     for vec in vec_vec {
    //         let value_count = vec.len();
    //         let range_set_blaze = vec.iter().collect::<RangeSetBlaze<_>>();
    //         let unique_count = range_set_blaze.len();
    //         let range_count = range_set_blaze.ranges_len();
    //         let min = range_set_blaze.first().unwrap();
    //         let max = range_set_blaze.last().unwrap();
    //         println!("{name}, {value_count}, {unique_count}, {range_count}, {min}, {max}");
    //     }
    // }

    Ok(())
}
