//! Beginner-friendly educational exhibits for the ChronoOS shell.

const USAGE: &str = "Usage: museum boot|kernel|memory|interrupts|keyboard|serial|era";

pub fn run(command: &str) -> bool {
    let command = command.trim();

    if command != "museum" && !command.starts_with("museum ") {
        return false;
    }

    print_usage();
    true
}

fn print_usage() {
    crate::println!("{}", USAGE);
}
