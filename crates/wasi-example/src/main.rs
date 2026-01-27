use std::io;

fn main() {
    let mut line = String::new();
    io::stdin().read_line(&mut line).expect("failed to read stdin");

    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.len() != 3 {
        eprintln!("expected: <number> <op> <number>");
        std::process::exit(1);
    }

    let lhs: i64 = parts[0].parse().expect("invalid left operand");
    let op = parts[1];
    let rhs: i64 = parts[2].parse().expect("invalid right operand");

    let result = match op {
        "+" => lhs + rhs,
        "-" => lhs - rhs,
        "*" => lhs * rhs,
        "/" => lhs / rhs,
        _ => {
            eprintln!("unsupported operator: {op}");
            std::process::exit(1);
        }
    };

    println!("{result}");
}
