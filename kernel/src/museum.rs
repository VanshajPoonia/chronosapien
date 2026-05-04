//! Beginner-friendly educational exhibits for the ChronoOS shell.

const USAGE: &str = "Usage: museum boot|kernel|memory|interrupts|keyboard|serial|era";

struct Exhibit {
    key: &'static str,
    title: &'static str,
    lines: &'static [&'static str],
}

pub fn run(command: &str) -> bool {
    let command = command.trim();

    if command != "museum" && !command.starts_with("museum ") {
        return false;
    }

    let mut parts = command.split_ascii_whitespace();
    let _command_name = parts.next();

    let Some(topic) = parts.next() else {
        print_usage();
        return true;
    };

    if parts.next().is_some() {
        print_usage();
        return true;
    }

    let Some(exhibit) = find_exhibit(topic) else {
        print_usage();
        return true;
    };

    crate::serial_println!("[CHRONO] museum: {}", exhibit.key);
    print_exhibit(exhibit);
    true
}

fn print_usage() {
    crate::println!("{}", USAGE);
}

fn find_exhibit(topic: &str) -> Option<Exhibit> {
    match topic {
        "boot" => Some(Exhibit {
            key: "boot",
            title: "BOOT",
            lines: &[
                "When you press power, the firmware runs first.",
                "It finds the bootloader on disk and gives it control.",
                "The bootloader prepares memory and enters long mode.",
                "Then it jumps into the ChronoOS kernel.",
                "Everything after that is our code waking up.",
            ],
        }),
        "kernel" => Some(Exhibit {
            key: "kernel",
            title: "KERNEL",
            lines: &[
                "A kernel is the core program of an operating system.",
                "It talks to hardware and keeps the machine organized.",
                "ChronoOS sets up memory, interrupts, devices, and the shell.",
                "It also draws the screen and tracks small tasks.",
                "User commands are the visible tip of that work.",
            ],
        }),
        _ => None,
    }
}

fn print_exhibit(exhibit: Exhibit) {
    crate::println!("MUSEUM: {}", exhibit.title);

    for line in exhibit.lines {
        crate::println!("{}", line);
    }
}
