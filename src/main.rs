
fn main() {
    let message = "Hello from hick!";
    println!("{}", message);

    let numbers: Vec<i32> = (1..=10).collect();
    for n in &numbers {
        if n % 2 == 0 {
            println!("{} is even", n);
        }
    }
}
