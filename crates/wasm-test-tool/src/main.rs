/// Minimal WASI binary for testing hick-shell WASM dispatch.
///
/// Prints command-line arguments (excluding argv[0]) separated by spaces,
/// followed by a newline. Exits with code 0.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("{}", args.join(" "));
}
