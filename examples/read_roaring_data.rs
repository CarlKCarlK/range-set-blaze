use std::{
    collections::HashMap,
    fs::{self},
    path::Path,
};

fn main() -> std::io::Result<()> {
    let top = Path::new(r"M:\projects\roaring_data");
    let name_to_vec_vec = read_roaring_data(top)?;

    println!(
        "name_to_vec_vec: {:?}",
        name_to_vec_vec.keys().collect::<Vec<_>>()
    );

    Ok(())
}

fn read_roaring_data(top: &Path) -> Result<HashMap<String, Vec<Vec<u32>>>, std::io::Error> {
    let subfolders: Vec<_> = top.read_dir()?.map(|entry| entry.unwrap().path()).collect();
    let mut name_to_vec_vec: HashMap<String, Vec<Vec<u32>>> = HashMap::new();
    for subfolder in subfolders {
        let subfolder_name = subfolder.file_name().unwrap().to_string_lossy().to_string();
        let mut data: Vec<Vec<u32>> = Vec::new();
        for file in subfolder.read_dir()? {
            let file = file.unwrap().path();
            let contents = fs::read_to_string(&file)?;
            let contents = contents.trim_end_matches('\n');
            let nums: Vec<u32> = contents.split(',').map(|s| s.parse().unwrap()).collect();
            data.push(nums);
        }
        name_to_vec_vec.insert(subfolder_name, data);
    }
    Ok(name_to_vec_vec)
}
