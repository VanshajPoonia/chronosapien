//! Retro terminal RPG quests derived from compiled ChronoOS capabilities.

use crate::theme::Era;

const QUEST_USAGE: &str = "Usage: quest list|status";
const INNER_WIDTH: usize = 72;

#[derive(Clone, Copy)]
struct Quest {
    title: &'static str,
    summary: &'static str,
    flavor: &'static str,
    inventory: Option<&'static str>,
    next_step: &'static str,
    state: QuestState,
}

#[derive(Clone, Copy)]
enum QuestState {
    Complete,
    Locked,
}

impl QuestState {
    const fn is_complete(self) -> bool {
        match self {
            Self::Complete => true,
            Self::Locked => false,
        }
    }

    const fn marker(self) -> &'static str {
        match self {
            Self::Complete => "[x]",
            Self::Locked => "[ ]",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Locked => "locked",
        }
    }
}

#[derive(Clone, Copy)]
struct FrameStyle {
    top_left: &'static str,
    top_fill: &'static str,
    top_right: &'static str,
    side_left: &'static str,
    side_right: &'static str,
    bottom_left: &'static str,
    bottom_fill: &'static str,
    bottom_right: &'static str,
}

const QUESTS: &[Quest] = &[
    Quest {
        title: "The Boot",
        summary: "Kernel reached main()",
        flavor: "The first spark catches. ChronoOS wakes.",
        inventory: None,
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "Voice of God",
        summary: "Serial logging online",
        flavor: "A debug voice echoes through COM1.",
        inventory: Some("Serial Logging"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "First Words",
        summary: "Framebuffer output working",
        flavor: "Pixels become letters. The void gets subtitles.",
        inventory: Some("Framebuffer Console"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "Ears Open",
        summary: "Keyboard input working",
        flavor: "The machine listens for tiny scancode spells.",
        inventory: Some("Keyboard"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "The Shell",
        summary: "Commands accepted",
        flavor: "A prompt appears, and the kernel answers back.",
        inventory: Some("Shell"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "Time Traveler",
        summary: "Era switching live",
        flavor: "One kernel, four costumes, zero paradoxes.",
        inventory: Some("Era Engine"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "Gatekeeper",
        summary: "IDT and exceptions loaded",
        flavor: "The CPU now knows where to knock.",
        inventory: Some("IDT"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "The Watchmaker",
        summary: "Timer interrupt ticking",
        flavor: "The PIT starts counting heartbeats.",
        inventory: Some("PIT Timer"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "Mind Palace",
        summary: "Memory and heap online",
        flavor: "Pages align. The heap opens its first room.",
        inventory: Some("Heap"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "Pack Rat",
        summary: "In-memory filesystem online",
        flavor: "Tiny files find a temporary home.",
        inventory: Some("Filesystem"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "Tiny Guild",
        summary: "Built-in apps available",
        flavor: "Notes, math, and sysinfo join the party.",
        inventory: Some("Apps"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "Silver Pointer",
        summary: "Mouse input online",
        flavor: "The cursor learns to wander.",
        inventory: Some("Mouse"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "Glass Panes",
        summary: "Window manager online",
        flavor: "Little rooms appear inside the screen.",
        inventory: Some("Windows"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "Many Hands",
        summary: "Cooperative multitasking online",
        flavor: "Tasks take turns like polite adventurers.",
        inventory: Some("Multitasking"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "Museum Curator",
        summary: "Museum mode unlocked",
        flavor: "The kernel starts explaining itself.",
        inventory: Some("Museum Mode"),
        next_step: "",
        state: QuestState::Complete,
    },
    Quest {
        title: "Swift Fingers",
        summary: "Interrupt-driven keyboard input",
        flavor: "",
        inventory: None,
        next_step: "Replace keyboard polling with IRQ-driven input.",
        state: QuestState::Locked,
    },
    Quest {
        title: "Reclaimer",
        summary: "Reusable heap allocator",
        flavor: "",
        inventory: None,
        next_step: "Upgrade the bump heap so freed memory can be reused.",
        state: QuestState::Locked,
    },
    Quest {
        title: "Stone Archive",
        summary: "Persistent disk storage",
        flavor: "",
        inventory: None,
        next_step: "Add disk-backed storage so files survive reboot.",
        state: QuestState::Locked,
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
    let style = frame_style();
    let progress = progress();

    print_header("QUEST LIST", style);
    print_frame_line(ProgressLine(progress.completed, progress.total), style);
    print_frame_line("", style);

    for quest in QUESTS {
        print_frame_line(
            QuestLine {
                marker: quest.state.marker(),
                title: quest.title,
                summary: quest.summary,
                status: quest.state.label(),
            },
            style,
        );

        if quest.state.is_complete() {
            print_frame_line(FlavorLine(quest.flavor), style);
        }
    }

    print_footer(style);
}

fn print_quest_status() {
    let style = frame_style();
    let progress = progress();

    print_header("QUEST STATUS", style);
    print_frame_line(ProgressLine(progress.completed, progress.total), style);
    print_frame_line(CountLine("Locked Quests", progress.locked), style);
    print_frame_line("", style);

    match QUESTS.iter().find(|quest| !quest.state.is_complete()) {
        Some(quest) => {
            print_frame_line("Active Quest", style);
            print_frame_line(QuestTitle(quest.title), style);
            print_frame_line(quest.summary, style);
            print_frame_line(NextStep(quest.next_step), style);
        }
        None => {
            print_frame_line("All current quests complete.", style);
        }
    }

    print_footer(style);
}

fn print_stats() {
    let style = frame_style();
    let progress = progress();
    let era = crate::theme::active_profile().name;

    print_header("PLAYER STATS", style);
    print_frame_line(StatLine("Systems Online", progress.completed, progress.total), style);
    print_frame_line(CountLine("Artifacts Found", progress.completed), style);
    print_frame_line(CountLine("Locked Quests", progress.locked), style);
    print_frame_line(RankLine(rank_for(progress.completed, progress.total)), style);
    print_frame_line(EraLine(era), style);
    match active_quest() {
        Some(quest) => print_frame_line(QuestTitle(quest.title), style),
        None => print_frame_line("Current: All quests complete", style),
    }
    print_footer(style);
}

fn print_inventory() {
    let style = frame_style();

    print_header("INVENTORY", style);

    for quest in QUESTS {
        if quest.state.is_complete() {
            if let Some(item) = quest.inventory {
                print_frame_line(InventoryLine(item), style);
            }
        }
    }

    print_footer(style);
}

struct QuestProgress {
    completed: usize,
    locked: usize,
    total: usize,
}

fn progress() -> QuestProgress {
    let completed = QUESTS
        .iter()
        .filter(|quest| quest.state.is_complete())
        .count();
    let total = QUESTS.len();

    QuestProgress {
        completed,
        locked: total.saturating_sub(completed),
        total,
    }
}

fn active_quest() -> Option<&'static Quest> {
    QUESTS.iter().find(|quest| !quest.state.is_complete())
}

fn rank_for(completed: usize, total: usize) -> &'static str {
    if completed == total {
        "Chronomancer"
    } else if completed * 4 >= total * 3 {
        "Kernel Knight"
    } else if completed * 2 >= total {
        "Boot Squire"
    } else {
        "Novice Operator"
    }
}

fn frame_style() -> FrameStyle {
    match crate::theme::active_era() {
        Era::Eighties => FrameStyle {
            top_left: "+",
            top_fill: "=",
            top_right: "+",
            side_left: "|",
            side_right: "|",
            bottom_left: "+",
            bottom_fill: "=",
            bottom_right: "+",
        },
        Era::Nineties => FrameStyle {
            top_left: "+",
            top_fill: "-",
            top_right: "+",
            side_left: "|",
            side_right: "|",
            bottom_left: "+",
            bottom_fill: "-",
            bottom_right: "+",
        },
        Era::TwoThousands => FrameStyle {
            top_left: "[",
            top_fill: "-",
            top_right: "]",
            side_left: "|",
            side_right: "|",
            bottom_left: "[",
            bottom_fill: "-",
            bottom_right: "]",
        },
        Era::Future => FrameStyle {
            top_left: "",
            top_fill: "-",
            top_right: "",
            side_left: "|",
            side_right: "|",
            bottom_left: "",
            bottom_fill: "-",
            bottom_right: "",
        },
    }
}

fn print_header(title: &str, style: FrameStyle) {
    print_border(style.top_left, style.top_fill, style.top_right);
    print_frame_line("", style);
    print_frame_line(title, style);
    print_frame_line("", style);
}

fn print_footer(style: FrameStyle) {
    print_frame_line("", style);
    print_border(style.bottom_left, style.bottom_fill, style.bottom_right);
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

fn print_frame_line(text: impl core::fmt::Display, style: FrameStyle) {
    crate::println!(
        "{} {:<width$} {}",
        style.side_left,
        text,
        style.side_right,
        width = INNER_WIDTH
    );
}

struct QuestLine {
    marker: &'static str,
    title: &'static str,
    summary: &'static str,
    status: &'static str,
}

impl core::fmt::Display for QuestLine {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "{} {:<16} - {} [{}]",
            self.marker, self.title, self.summary, self.status
        )
    }
}

struct FlavorLine(&'static str);

impl core::fmt::Display for FlavorLine {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "    {}", self.0)
    }
}

struct QuestTitle(&'static str);

impl core::fmt::Display for QuestTitle {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Current: {}", self.0)
    }
}

struct NextStep(&'static str);

impl core::fmt::Display for NextStep {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Next: {}", self.0)
    }
}

struct StatLine(&'static str, usize, usize);

impl core::fmt::Display for StatLine {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}: {}/{}", self.0, self.1, self.2)
    }
}

struct ProgressLine(usize, usize);

impl core::fmt::Display for ProgressLine {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Quest Progress: {}/{}", self.0, self.1)
    }
}

struct CountLine(&'static str, usize);

impl core::fmt::Display for CountLine {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}: {}", self.0, self.1)
    }
}

struct RankLine(&'static str);

impl core::fmt::Display for RankLine {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Rank: {}", self.0)
    }
}

struct EraLine(&'static str);

impl core::fmt::Display for EraLine {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Era: {}", self.0)
    }
}

struct InventoryLine(&'static str);

impl core::fmt::Display for InventoryLine {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "- {}", self.0)
    }
}
