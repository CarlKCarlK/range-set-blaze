use std::env;

fn main() {
    if let Ok(features) = env::var("BUILDFEATURES") {
        for feature in features.split(',') {
            println!("cargo:rustc-cfg=feature=\"{}\"", feature.trim());
            println!("cargo:rerun-if-env-changed=BUILDFEATURES");
        }
    }
}
