use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use walkdir::WalkDir;

fn main() {
    println!("Group\tId\tMean(ns)\tStdErr(ns)");

    // cmk this ../../.. is hard to explain
    for entry in WalkDir::new("../../../target/criterion")
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        // println!("path: {}", path.display());
        if path.is_file()
            && path.file_name() == Some(OsStr::new("estimates.json"))
            && path
                .ancestors()
                .any(|parent| parent.file_name() == Some(OsStr::new("new")))
        {
            // println!(">> path: {}", path.display());
            match fs::read_to_string(path) {
                Ok(contents) => {
                    // println!(">>> path: {}", path.display());
                    let json: Value = serde_json::from_str(&contents).unwrap_or_else(|_| {
                        panic!("Invalid JSON format in file: {}", path.display())
                    });
                    if let Some(mean) = json["mean"]["point_estimate"].as_f64() {
                        let mean = format!("{:.4e}", mean).parse::<f64>().unwrap(); // slow, but fine
                        let standard_error =
                            json["mean"]["standard_error"].as_f64().unwrap_or(f64::NAN);
                        let standard_error =
                            format!("{:.4e}", standard_error).parse::<f64>().unwrap(); // slow, but fine
                        let components: Vec<_> = path
                            .components()
                            .map(|c| c.as_os_str().to_str().unwrap_or(""))
                            .collect();
                        let benchmark_group = components[components.len() - 4];
                        let function = components[components.len() - 3];
                        println!(
                            "{}\t{}\t{}\t{}",
                            benchmark_group, function, mean, standard_error
                        );
                    } else {
                        println!("{}\tmissing", path.display());
                    }
                }
                Err(_) => println!("Error reading file: {}", path.display()),
            }
        }
    }
}
