//! Retro terminal RPG quests derived from compiled ChronoOS capabilities.

const QUEST_USAGE: &str = "Usage: quest list|status";

#[derive(Clone, Copy)]
struct Quest {
    title: &'static str,
    summary: &'static str,
    flavor: &'static str,
    inventory: Option<&'static str>,
    next_step: &'static str,
    complete: bool,
}

const QUESTS: &[Quest] = &[
    Quest {
        title: "The Boot",
        summary: "Kernel reached main()",
        flavor: "The first spark catches. ChronoOS wakes.",
        inventory: None,
        next_step: "",
        complete: true,
    },
    Quest {
        title: "Voice of God",
        summary: "Serial logging online",
        flavor: "A debug voice echoes through COM1.",
        inventory: Some("Serial Logging"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "First Words",
        summary: "Framebuffer output working",
        flavor: "Pixels become letters. The void gets subtitles.",
        inventory: Some("Framebuffer Console"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "Ears Open",
        summary: "Keyboard input working",
        flavor: "The machine listens for tiny scancode spells.",
        inventory: Some("Keyboard"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "The Shell",
        summary: "Commands accepted",
        flavor: "A prompt appears, and the kernel answers back.",
        inventory: Some("Shell"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "Time Traveler",
        summary: "Era switching live",
        flavor: "One kernel, four costumes, zero paradoxes.",
        inventory: Some("Era Engine"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "Gatekeeper",
        summary: "IDT and exceptions loaded",
        flavor: "The CPU now knows where to knock.",
        inventory: Some("IDT"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "The Watchmaker",
        summary: "Timer interrupt ticking",
        flavor: "The PIT starts counting heartbeats.",
        inventory: Some("PIT Timer"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "Mind Palace",
        summary: "Memory and heap online",
        flavor: "Pages align. The heap opens its first room.",
        inventory: Some("Heap"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "Pack Rat",
        summary: "In-memory filesystem online",
        flavor: "Tiny files find a temporary home.",
        inventory: Some("Filesystem"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "Tiny Guild",
        summary: "Built-in apps available",
        flavor: "Notes, math, and sysinfo join the party.",
        inventory: Some("Apps"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "Silver Pointer",
        summary: "Mouse input online",
        flavor: "The cursor learns to wander.",
        inventory: Some("Mouse"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "Glass Panes",
        summary: "Window manager online",
        flavor: "Little rooms appear inside the screen.",
        inventory: Some("Windows"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "Many Hands",
        summary: "Cooperative multitasking online",
        flavor: "Tasks take turns like polite adventurers.",
        inventory: Some("Multitasking"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "Museum Curator",
        summary: "Museum mode unlocked",
        flavor: "The kernel starts explaining itself.",
        inventory: Some("Museum Mode"),
        next_step: "",
        complete: true,
    },
    Quest {
        title: "Swift Fingers",
        summary: "Interrupt-driven keyboard input",
        flavor: "",
        inventory: None,
        next_step: "Replace keyboard polling with IRQ-driven input.",
        complete: false,
    },
    Quest {
        title: "Reclaimer",
        summary: "Reusable heap allocator",
        flavor: "",
        inventory: None,
        next_step: "Upgrade the bump heap so freed memory can be reused.",
        complete: false,
    },
    Quest {
        title: "Stone Archive",
        summary: "Persistent disk storage",
        flavor: "",
        inventory: None,
        next_step: "Add disk-backed storage so files survive reboot.",
        complete: false,
    },
];

pub fn run(command: &str) -> bool {
    let command = command.trim();

    match command {
        "stats" => {
            crate::serial_println!("[CHRONO] quest: stats");
            print_stats();
            return true;
        }
        "inventory" => {
            crate::serial_println!("[CHRONO] quest: inventory");
            print_inventory();
            return true;
        }
        _ => {}
    }

    if command != "quest" && !command.starts_with("quest ") {
        return false;
    }

    run_quest_command(command);
    true
}

fn run_quest_command(command: &str) {
    let mut parts = command.split_ascii_whitespace();
    let _command_name = parts.next();

    let Some(subcommand) = parts.next() else {
        print_usage();
        return;
    };

    if parts.next().is_some() {
        print_usage();
        return;
    }

    match subcommand {
        "list" => {
            crate::serial_println!("[CHRONO] quest: list");
            print_quest_list();
        }
        "status" => {
            crate::serial_println!("[CHRONO] quest: status");
            print_quest_status();
        }
        _ => print_usage(),
    }
}

fn print_usage() {
    crate::println!("{}", QUEST_USAGE);
}

fn print_quest_list() {
    crate::println!("QUEST LIST");

    for quest in QUESTS {
        let marker = if quest.complete { "[x]" } else { "[ ]" };
        crate::println!("{} {:<16} - {}", marker, quest.title, quest.summary);

        if quest.complete {
            crate::println!("    {}", quest.flavor);
        }
    }
}

fn print_quest_status() {
    crate::println!("QUEST STATUS");

    match QUESTS.iter().find(|quest| !quest.complete) {
        Some(quest) => {
            crate::println!("Current: {}", quest.title);
            crate::println!("{}", quest.summary);
            crate::println!("Next: {}", quest.next_step);
        }
        None => {
            crate::println!("All current quests complete.");
        }
    }
}

fn print_stats() {
    let completed = completed_count();
    let total = QUESTS.len();
    let locked = total.saturating_sub(completed);
    let era = crate::theme::active_profile().name;

    crate::println!("PLAYER STATS");
    crate::println!("Systems Online: {}/{}", completed, total);
    crate::println!("Artifacts Found: {}", completed);
    crate::println!("Locked Quests: {}", locked);
    crate::println!("Era: {}", era);
}

fn print_inventory() {
    crate::println!("INVENTORY");

    for quest in QUESTS {
        if quest.complete {
            if let Some(item) = quest.inventory {
                crate::println!("- {}", item);
            }
        }
    }
}

fn completed_count() -> usize {
    QUESTS.iter().filter(|quest| quest.complete).count()
}
