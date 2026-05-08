//! Beginner-friendly educational exhibits for the ChronoOS shell.

const USAGE: &str = "Usage: museum boot|kernel|memory|interrupts|keyboard|serial|era";
const INNER_WIDTH: usize = 68;

struct Exhibit {
    key: &'static str,
    title: &'static str,
    lines: &'static [&'static str],
}

type FrameStyle = crate::theme::TextFrameProfile;

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
        "memory" => Some(Exhibit {
            key: "memory",
            title: "MEMORY",
            lines: &[
                "RAM is the fast workspace the CPU can read and write.",
                "ChronoOS reads the memory map to find usable frames.",
                "Paging maps virtual addresses to physical memory.",
                "The heap is a managed area for growing data structures.",
                "Together, they let the kernel use memory safely.",
            ],
        }),
        "interrupts" => Some(Exhibit {
            key: "interrupts",
            title: "INTERRUPTS",
            lines: &[
                "An interrupt is a signal that asks the CPU for attention.",
                "The IDT is a table of handlers for those signals.",
                "ChronoOS loads the IDT so exceptions and devices have paths.",
                "Timers, faults, and hardware events can all enter this way.",
                "It is how the machine says: something happened.",
            ],
        }),
        "keyboard" => Some(Exhibit {
            key: "keyboard",
            title: "KEYBOARD",
            lines: &[
                "The PS/2 keyboard sends small numbers called scancodes.",
                "ChronoOS turns those scancodes into keys and characters.",
                "Polling means the shell asks for keys in a loop.",
                "Interrupts mean the keyboard can call the CPU when ready.",
                "Both are ways to turn key presses into commands.",
            ],
        }),
        "serial" => Some(Exhibit {
            key: "serial",
            title: "SERIAL",
            lines: &[
                "COM1 is an old but reliable serial port.",
                "It sends text one byte at a time outside the main screen.",
                "Developers use it because it works even when graphics fail.",
                "ChronoOS logs boot steps, commands, and device events there.",
                "Serial output is the kernel leaving a trail of breadcrumbs.",
            ],
        }),
        "era" => Some(Exhibit {
            key: "era",
            title: "ERA",
            lines: &[
                "The era system changes how ChronoOS presents itself.",
                "The same kernel can feel like 1984, 1995, 2007, or 2040.",
                "Prompts, colors, top bars, and windows all follow the era.",
                "The machine underneath stays the same.",
                "Only its personality changes.",
            ],
        }),
        _ => None,
    }
}

fn print_exhibit(exhibit: Exhibit) {
    let style = frame_style();
    let title = exhibit.title;

    print_border(style.top_left, style.top_fill, style.top_right);
    print_frame_line("", style);
    crate::println!(
        "{} {:<width$} {}",
        style.side_left,
        MuseumTitle(title),
        style.side_right,
        width = INNER_WIDTH
    );
    print_frame_line("", style);

    for line in exhibit.lines {
        print_frame_line(line, style);
    }

    print_frame_line("", style);
    print_border(style.bottom_left, style.bottom_fill, style.bottom_right);
}

fn frame_style() -> FrameStyle {
    crate::theme::active_profile().text_frame
}

fn print_border(left: &str, fill: &str, right: &str) {
    crate::print!("{}", left);

    let fill_width = if left.is_empty() && right.is_empty() {
        INNER_WIDTH + 4
    } else {
        INNER_WIDTH + 2
    };

    for _ in 0..fill_width {
        crate::print!("{}", fill);
    }

    crate::println!("{}", right);
}

fn print_frame_line(line: &str, style: FrameStyle) {
    crate::println!(
        "{} {:<width$} {}",
        style.side_left,
        line,
        style.side_right,
        width = INNER_WIDTH
    );
}

struct MuseumTitle(&'static str);

impl core::fmt::Display for MuseumTitle {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "MUSEUM: {}", self.0)
    }
}
