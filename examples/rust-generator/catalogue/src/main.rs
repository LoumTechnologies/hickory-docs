//! Run the generated code, so the document can pin what it produced.

fn main() {
    println!("{}", catalogue::Order::describe());
    println!("{}", catalogue::Customer::describe());
}
