macro_rules! print_feature_status {
    ($feature:tt, $bit_width:expr) => {
        println!(
            "{:<10}\t{:<18}\t{}\t\t{}",
            $feature,
            format!("{}-bit/{}-bytes", $bit_width, $bit_width / 8),
            std::is_x86_feature_detected!($feature),
            cfg!(target_feature = $feature),
        );
    };
}

fn main() {
    println!("feature   \twidth           \tavailable\tenabled");
    print_feature_status!("sse2", 128);
    print_feature_status!("avx2", 256);
    print_feature_status!("avx512f", 512);
}
