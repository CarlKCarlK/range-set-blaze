macro_rules! print_target_feature_status {
    ($target_feature:tt, $bit_width:expr) => {
        println!(
            "{:<10}\t{:<18}\t{}\t\t{}",
            $target_feature,
            format!("{}-bit/{}-bytes", $bit_width, $bit_width / 8),
            std::is_x86_feature_detected!($target_feature),
            cfg!(target_feature = $target_feature)
        );
    };
}

fn main() {
    println!("feature   \twidth           \tavailable\tenabled");
    print_target_feature_status!("sse2", 128);
    print_target_feature_status!("avx2", 256);
    print_target_feature_status!("avx512f", 512);
}
