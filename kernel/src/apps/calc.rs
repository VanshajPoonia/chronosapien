//! Tiny integer calculator app.

pub fn run(args: &str) {
    let mut parts = args.split_ascii_whitespace();

    let Some(left) = parts.next().and_then(parse_i64) else {
        print_usage();
        return;
    };
    let Some(operator) = parts.next() else {
        print_usage();
        return;
    };
    let Some(right) = parts.next().and_then(parse_i64) else {
        print_usage();
        return;
    };

    if parts.next().is_some() {
        print_usage();
        return;
    }

    match operator {
        "+" => crate::println!("{}", left + right),
        "-" => crate::println!("{}", left - right),
        "*" => crate::println!("{}", left * right),
        "/" if right != 0 => crate::println!("{}", left / right),
        "/" => crate::println!("divide by zero"),
        _ => print_usage(),
    }
}

fn parse_i64(text: &str) -> Option<i64> {
    text.parse().ok()
}

fn print_usage() {
    crate::println!("Usage: calc <int> +|-|*|/ <int>");
}
