// Array summation benchmark - Rust version
fn array_sum() -> i32 {
    let mut sum = 0;
    for i in 0..1000000 {
        sum += i;
    }
    sum
}

fn main() {
    let result = array_sum();
    println!("Array sum result: {}", result);
}