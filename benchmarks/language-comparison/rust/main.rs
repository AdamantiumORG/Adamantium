fn main() {
    let mut total: i32 = 0;
    for index in 0..1_000_000_i32 {
        total = (total + index) % 1_000_003;
    }
    println!("{total}");
}
