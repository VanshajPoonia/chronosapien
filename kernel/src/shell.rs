//! Tiny line-based shell for the first interactive milestone.

use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};

use crate::apps;
use crate::console;
use crate::fs::{self, FsError};
use crate::keyboard::{self, KeyEvent};
use crate::memory;
use crate::mouse;
use crate::museum;
use crate::net;
use crate::process;
use crate::quest;
use crate::sched;
use crate::theme::{self, Era};
use crate::timer;
use crate::wm;
use crate::{print, println, serial_println};

const COMMAND_BUFFER_CAPACITY: usize = 80;
const RECENT_COMMAND_COUNT: usize = 6;
const RECENT_COMMAND_CAPACITY: usize = 64;
const RESET_COMMAND_PORT: u16 = 0x64;
const CPU_RESET_COMMAND: u8 = 0xFE;
const RELIABILITY_MODE_DEMO: u8 = 1;

static RELIABILITY_MODE: AtomicU8 = AtomicU8::new(RELIABILITY_MODE_DEMO);
static RECENT_COMMANDS: GlobalRecentCommands =
    GlobalRecentCommands(UnsafeCell::new(RecentCommands::new()));
static RECENT_APPS: GlobalRecentCommands =
    GlobalRecentCommands(UnsafeCell::new(RecentCommands::new()));

#[derive(Clone, Copy, Eq, PartialEq)]
enum ReliabilityMode {
    Safe,
    Demo,
    Experimental,
}

impl ReliabilityMode {
    const fn from_byte(value: u8) -> Self {
        match value {
            0 => Self::Safe,
            2 => Self::Experimental,
            _ => Self::Demo,
        }
    }

    const fn as_byte(self) -> u8 {
        match self {
            Self::Safe => 0,
            Self::Demo => 1,
            Self::Experimental => 2,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Demo => "demo",
            Self::Experimental => "experimental",
        }
    }
}

pub fn run() -> ! {
    let mut buffer = CommandBuffer::new();
    let mut cursor_visible = true;
    let mut idle_ticks = 0;
    let mut top_bar_second = timer::uptime_seconds();
    // Throttle cooperative yields to once per timer tick (100 Hz) so we do not
    // flood the serial log or waste cycles context-switching in a tight loop.
    let mut last_yield_tick = 0u64;

    print_prompt();
    draw_cursor();

    loop {
        let mouse_activity = process_mouse_events();

        // The shell polls one key at a time. Printable keys edit the fixed
        // buffer and framebuffer line; Enter turns that buffer into a command,
        // runs it, clears the buffer, and redraws the prompt.
        match keyboard::read_key() {
            Some(KeyEvent::Char(byte)) => {
                hide_cursor(&mut cursor_visible);

                if buffer.push(byte) {
                    print!("{}", byte as char);
                } else {
                    serial_println!("[CHRONO] input buffer full");
                }

                show_cursor(&mut cursor_visible);
                wm::redraw_if_open();
                idle_ticks = 0;
            }
            Some(KeyEvent::Backspace) => {
                hide_cursor(&mut cursor_visible);

                if buffer.pop().is_some() {
                    console::backspace();
                }

                show_cursor(&mut cursor_visible);
                wm::redraw_if_open();
                idle_ticks = 0;
            }
            Some(KeyEvent::Enter) => {
                hide_cursor(&mut cursor_visible);
                println!();

                record_recent_command(buffer.as_str());
                execute_command(buffer.as_str());
                buffer.clear();
                print_prompt();
                show_cursor(&mut cursor_visible);
                wm::redraw_if_open();
                idle_ticks = 0;
            }
            None => {
                if mouse_activity {
                    idle_ticks = 0;
                } else {
                    idle_ticks += 1;
                }

                if idle_ticks >= theme::active_profile().cursor_blink_ticks {
                    toggle_cursor(&mut cursor_visible);
                    wm::redraw_if_open();
                    idle_ticks = 0;
                }

                let uptime = timer::uptime_seconds();
                if uptime != top_bar_second {
                    top_bar_second = uptime;
                    console::refresh_top_bar();
                    wm::redraw_if_open();
                }

                // Yield to other tasks once per timer tick.
                let now = timer::ticks();
                if now != last_yield_tick {
                    last_yield_tick = now;
                    net::poll();
                    sched::yield_now();
                }

                cpu_relax();
            }
        }
    }
}

fn print_prompt() {
    let profile = theme::active_profile();

    print!("{} ", profile.screen_prompt);
}

fn process_mouse_events() -> bool {
    let mut handled = false;

    while let Some(event) = mouse::take_event() {
        wm::handle_mouse_event(event);
        handled = true;
    }

    handled
}

struct CommandBuffer {
    bytes: [u8; COMMAND_BUFFER_CAPACITY],
    len: usize,
}

impl CommandBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; COMMAND_BUFFER_CAPACITY],
            len: 0,
        }
    }

    fn push(&mut self, byte: u8) -> bool {
        if self.len >= self.bytes.len() {
            return false;
        }

        self.bytes[self.len] = byte;
        self.len += 1;
        true
    }

    fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        Some(self.bytes[self.len])
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn as_str(&self) -> &str {
        // SAFETY: The keyboard decoder only returns printable ASCII bytes, and
        // ASCII is always valid UTF-8.
        unsafe { core::str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }
}

struct GlobalRecentCommands(UnsafeCell<RecentCommands>);

unsafe impl Sync for GlobalRecentCommands {}

struct RecentCommands {
    entries: [[u8; RECENT_COMMAND_CAPACITY]; RECENT_COMMAND_COUNT],
    lengths: [usize; RECENT_COMMAND_COUNT],
    next: usize,
    count: usize,
}

impl RecentCommands {
    const fn new() -> Self {
        Self {
            entries: [[0; RECENT_COMMAND_CAPACITY]; RECENT_COMMAND_COUNT],
            lengths: [0; RECENT_COMMAND_COUNT],
            next: 0,
            count: 0,
        }
    }

    fn record(&mut self, command: &str) {
        let command = command.trim();
        if command.is_empty() || command == "recent" {
            return;
        }

        let bytes = command.as_bytes();
        let mut len = bytes.len();
        if len > RECENT_COMMAND_CAPACITY {
            len = RECENT_COMMAND_CAPACITY;
        }

        let slot = self.next;
        let mut index = 0;
        while index < len {
            self.entries[slot][index] = bytes[index];
            index += 1;
        }
        while index < RECENT_COMMAND_CAPACITY {
            self.entries[slot][index] = 0;
            index += 1;
        }

        self.lengths[slot] = len;
        self.next = (self.next + 1) % RECENT_COMMAND_COUNT;
        if self.count < RECENT_COMMAND_COUNT {
            self.count += 1;
        }
    }
}

fn record_recent_command(command: &str) {
    // SAFETY: The shell is the only writer. Commands can call other commands
    // through `execute_command`, but only the interactive Enter path records.
    unsafe {
        (*RECENT_COMMANDS.0.get()).record(command);
    }
}

fn print_recent_commands() {
    // SAFETY: Read-only snapshot from the single shell command path.
    let recent = unsafe { &*RECENT_COMMANDS.0.get() };

    println!("Recent commands");
    println!("Scope: in-memory commands typed since boot; no arrow-key recall.");

    if recent.count == 0 {
        println!("No recent commands yet.");
        println!("Try: workspace, shortcuts, help, apps, fs status");
        return;
    }

    let mut shown = 0;
    while shown < recent.count {
        let index = (recent.next + RECENT_COMMAND_COUNT - 1 - shown) % RECENT_COMMAND_COUNT;
        let len = recent.lengths[index];
        let text = unsafe { core::str::from_utf8_unchecked(&recent.entries[index][..len]) };
        println!("{}. {}", shown + 1, text);
        shown += 1;
    }
}

fn record_recent_app(name: &str) {
    // SAFETY: App launcher routes run on the same single shell command path as
    // command history. This stores names only and has fixed capacity.
    unsafe {
        (*RECENT_APPS.0.get()).record(name);
    }
}

fn print_recent_apps() {
    // SAFETY: Read-only snapshot from the single shell command path.
    let recent = unsafe { &*RECENT_APPS.0.get() };

    println!("Recent apps");
    println!("Scope: in-memory app launcher routes since boot.");

    if recent.count == 0 {
        println!("No app launcher entries yet.");
        println!("Try: apps featured");
        return;
    }

    let mut shown = 0;
    while shown < recent.count {
        let index = (recent.next + RECENT_COMMAND_COUNT - 1 - shown) % RECENT_COMMAND_COUNT;
        let len = recent.lengths[index];
        let text = unsafe { core::str::from_utf8_unchecked(&recent.entries[index][..len]) };
        println!("{}. {}", shown + 1, text);
        shown += 1;
    }
}

fn execute_command(command: &str) {
    let command = command.trim();

    if !command.is_empty() {
        serial_println!("[CHRONO] cmd: {}", command);
    }

    match command {
        "" => {}
        command if command == "help" || command.starts_with("help ") => run_help(command),
        "workspace" | "status" => print_workspace_dashboard(),
        "shortcuts" => print_shortcuts(),
        "whereami" => print_whereami(),
        "verify" => print_verify_summary(),
        "recent" => print_recent_commands(),
        command if command == "files" || command.starts_with("files ") => run_files_command(command),
        "theme" => print_theme_workspace(),
        command if command == "mode" || command.starts_with("mode ") => run_mode_command(command),
        command if command == "safe" || command.starts_with("safe ") => run_safe_command(command),
        "start" | "welcome" => print_welcome(),
        command if command == "guide" || command.starts_with("guide ") => run_guide(command),
        command if command == "learn" || command.starts_with("learn ") => run_learn(command),
        command if command == "explain" || command.starts_with("explain ") => {
            run_explain_command(command)
        }
        command if command == "demo" || command.starts_with("demo ") => run_demo(command),
        command if command == "tour" || command.starts_with("tour ") => run_tour(command),
        command if command == "capsule" || command.starts_with("capsule ") => run_capsule(command),
        command if command == "doctor" || command.starts_with("doctor ") => run_doctor(command),
        command if command == "notes" || command.starts_with("notes ") => run_notes_app(command),
        command if command == "apps" || command.starts_with("apps ") => run_apps_launcher(command),
        command if command == "travel" || command.starts_with("travel ") => run_travel(command),
        command if command == "poster" || command.starts_with("poster ") => run_poster(command),
        "museum disk" => print_museum_disk_page(),
        "museum filesystem" => print_museum_filesystem_page(),
        "museum userspace" => print_museum_userspace_page(),
        "museum syscalls" => print_museum_syscalls_page(),
        "museum elf" => print_museum_elf_page(),
        "museum networking" => print_museum_networking_page(),
        "museum smp" => print_museum_smp_page(),
        "museum scheduler" => print_museum_scheduler_page(),
        "clear" => console::clear(),
        "about" => print_about(),
        "reboot" => reboot(),
        "uptime" => print_uptime(),
        "clock" => print_clock(),
        "mem" => print_memory(),
        "cores" => print_cores(),
        command if command == "beep" || command.starts_with("beep ") => beep(command),
        command if command == "userspace" || command.starts_with("userspace ") => {
            run_userspace_command(command)
        }
        "ring3" => run_ring3_with_warning(),
        "syshello" => run_syshello_with_warning(),
        "ls" => list_files(),
        command if command == "cat" || command.starts_with("cat ") => cat_file(command),
        command if command == "write" || command.starts_with("write ") => write_file(command),
        command if command == "exec" || command.starts_with("exec ") => {
            exec_file_with_warning(command)
        }
        command if command == "rm" || command.starts_with("rm ") => remove_file(command),
        command if command == "fs" || command.starts_with("fs ") => run_fs_command(command),
        command if command == "fsck" || command.starts_with("fsck ") => run_fsck(command),
        command if command == "journal" || command.starts_with("journal ") => run_journal(command),
        command if command == "era" || command.starts_with("era ") => run_era_command(command),
        command if command == "windows" || command.starts_with("windows ") => {
            run_windows_command(command)
        }
        command if command == "open" || command.starts_with("open ") => open_window(command),
        "tasks" => list_tasks(),
        command if command == "kill" || command.starts_with("kill ") => kill_task(command),
        command if net::run(command) => {}
        command if museum::run(command) => {}
        command if quest::run(command) => {}
        command if apps::run(command) => {}
        _ => print_unknown_command(command),
    }
}

fn run_help(command: &str) {
    let topic = command.strip_prefix("help").unwrap_or("").trim();

    match topic {
        "" => print_help(),
        "search" => print_help_search(topic),
        topic if topic.starts_with("search ") => print_help_search(topic),
        "workspace" | "dashboard" | "shortcuts" | "whereami" => print_help_workspace(),
        "start" | "guide" => print_help_start(),
        "learn" | "learning" => print_help_learn(),
        "mode" | "safe" | "reliability" => print_help_mode(),
        "apps" | "app" => print_help_apps(),
        "fs" | "files" | "filesystem" => print_help_fs(),
        "theme" | "themes" => print_help_theme(),
        "system" | "status" | "verify" => print_help_system(),
        "network" | "net" => print_help_network(),
        "userspace" | "user" | "elf" => print_help_userspace(),
        "labs" | "lab" | "debug" => print_help_labs(),
        "roadmap" | "future" | "next" => print_help_roadmap(),
        _ => print_unknown_help_topic(topic),
    }
}

fn print_help() {
    println!("ChronoOS help");
    println!("Workspace       : workspace, shortcuts, whereami, recent, status, verify");
    println!("Getting started : start, welcome, guide, learn, demo, tour");
    println!("Learning paths  : learn map|progress|beginner|advanced|next, explain <term>");
    println!("Reliability     : mode, mode safe|demo|experimental, safe on|off");
    println!("Eras and themes : era, travel <year>, poster eras, apps theme");
    println!("Apps            : apps, notes, calc, sysinfo, open notes, open sysinfo");
    println!("Windows/tasks   : windows, open notes, open sysinfo, tasks, kill <id>");
    println!("Filesystem      : files, fs, ls, cat, write, rm, fsck, journal");
    println!("Museum/quests   : museum index, quest list/status/dependencies/badges");
    println!("System status   : doctor, sysinfo, mem, cores, uptime, clock, poster system");
    println!("Userspace       : ring3, syshello, exec <name> (needs verification)");
    println!("Networking      : net status, net config, net arp, net udp, net log");
    println!("Debug/lab       : beep <hz>, reboot, fsck repair, risky demos");
    println!("Roadmap/future  : capsule next, poster roadmap, tour future");
    println!();
    println!("More help: help workspace|start|learn|mode|apps|fs|system|network|userspace");
    println!("Search: help search <term>");
}

fn print_help_workspace() {
    println!("Help: workspace");
    println!("- workspace : compact dashboard for the current shell");
    println!("- status    : alias for workspace");
    println!("- verify    : read-only verification boundary summary");
    println!("- whereami  : current era, mode, UI context, and next action");
    println!("- shortcuts : best demo and utility commands");
    println!("- recent    : commands typed since boot, no persistent history");
    println!();
    println!("These commands are read-only and do not certify runtime behavior.");
    println!("Next: workspace");
}

fn print_help_start() {
    println!("Help: getting started");
    println!("- start / welcome : polished first-run ChronoOS screen");
    println!("- guide           : topic menu for the guided shell path");
    println!("- guide quick     : short first-demo route");
    println!("- guide full      : longer route across product surfaces");
    println!("- learn           : structured curriculum paths by subsystem");
    println!("- demo            : read-only preview of current features");
    println!("- tour            : educational explanations by subsystem");
    println!();
    println!("Difference: guide orients you, demo previews, tour teaches.");
    println!("Next: guide quick or learn");
}

fn print_help_learn() {
    println!("Help: learning paths");
    println!("- learn              : curriculum overview");
    println!("- learn boot         : firmware, bootloader, kernel handoff");
    println!("- learn memory       : memory map, paging, heap");
    println!("- learn interrupts   : IDT, timer, keyboard, mouse/input");
    println!("- learn filesystem   : ChronoFS, fsck, journal");
    println!("- learn gui          : apps, windows, shell-first UI");
    println!("- learn userspace    : Ring 3, syscalls, static ELF");
    println!("- learn networking   : RTL8139, static IPv4, ARP, UDP");
    println!("- learn scheduler    : tasks, scheduler, SMP/AP boundary");
    println!("- learn eras         : themes, travel, era identity");
    println!("- learn roadmap      : future systems");
    println!("- learn map          : learning area status map");
    println!("- learn progress     : static progress and badge summary");
    println!("- learn beginner     : safe first curriculum route");
    println!("- learn advanced     : intentional verification route");
    println!("- learn next         : safe next curriculum step");
    println!("- explain <term>     : short OS glossary entry");
    println!("Each path is read-only and does not run probes.");
}

fn print_help_mode() {
    println!("Help: reliability mode");
    println!("- mode / mode status       : show current warning mode");
    println!("- mode safe                : prefer read-only demo paths");
    println!("- mode demo                : default portfolio/demo mode");
    println!("- mode experimental        : intentional lab/verification mode");
    println!("- safe / safe status       : alias for mode status");
    println!("- safe on                  : alias for mode safe");
    println!("- safe off                 : return to mode demo");
    println!();
    println!("This is warning-only; it does not block commands or provide security.");
}

fn print_help_apps() {
    println!("Help: apps");
    println!("- apps / apps list         : static app registry");
    println!("- apps featured            : best demo-ready app surfaces");
    println!("- apps recent              : app launcher routes since boot");
    println!("- apps category <name>     : browse Core, Files, Learning, System, Networking, Visual, Debug/Lab, Roadmap/Future");
    println!("- apps info <name>         : app manifest details");
    println!("- apps help <name>         : app-specific help and related commands");
    println!("- apps demo <name>         : safe demo script preview");
    println!("- apps launch <name>       : run launch command if available");
    println!("- apps verified            : app entries with partial QEMU evidence");
    println!("- apps roadmap             : future app ideas");
    println!("- apps notes|calc|sysinfo  : legacy direct routes");
    println!("- notes           : notes home screen");
    println!("- calc 6 * 7      : integer calculator");
    println!("- sysinfo         : compact status view");
    println!("- open notes      : small notes window path");
    println!("- open sysinfo    : small sysinfo window path");
    println!("- windows         : list/status/focus/close small windows");
    println!();
    println!("Note: apps are shell-first; open uses partially implemented windows.");
    println!("Next: apps");
}

fn print_help_fs() {
    println!("Help: filesystem");
    println!("- files                    : ChronoFS usability overview");
    println!("- files list               : visible files with byte sizes");
    println!("- files info <name>        : file size, storage mode, safety note");
    println!("- files search <term>      : search names and UTF-8 contents");
    println!("- files sample             : read-only sample file suggestions");
    println!("- files demo               : guided demo script, no automatic writes");
    println!("- files copy <src> <dst>   : copy only when destination is absent");
    println!("- files rename <old> <new> : deferred safe-rename explanation");
    println!("- fs                       : ChronoFS status summary");
    println!("- fs info                  : layout, limits, and journal reservation");
    println!("- fs check                 : read-only fsck summary");
    println!("- fs journal               : journal status");
    println!("- ls                       : list ChronoFS files");
    println!("- cat <name>               : print a text file");
    println!("- write <name> <content>   : create or overwrite a file");
    println!("- rm <name>                : remove a file");
    println!("- fsck                     : read-only ChronoFS metadata check");
    println!("- fsck repair              : conservative metadata repair");
    println!("- journal                  : ChronoFS journal status");
    println!();
    println!("Warning: fsck repair mutates metadata; files rename is intentionally deferred.");
    println!("Aliases/cards: apps files");
    println!("Next: files");
}

fn print_help_theme() {
    println!("Help: themes");
    println!("- theme      : current era/theme card");
    println!("- era        : list available eras");
    println!("- era 1984   : monochrome terminal era");
    println!("- era 1995   : chunky desktop era");
    println!("- era 2007   : glossy app-era lens");
    println!("- era 2040   : future lab console");
    println!("- travel <year> : map a year to an era");
    println!("- poster eras   : screenshot-friendly era card");
    println!();
    println!("Theme changes presentation only, not kernel behavior.");
}

fn print_help_system() {
    println!("Help: system status");
    println!("- workspace       : compact shell dashboard");
    println!("- status          : alias for workspace");
    println!("- verify          : read-only verification summary");
    println!("- whereami        : current shell context");
    println!("- doctor          : conservative subsystem report");
    println!("- poster system   : screenshot-friendly status card");
    println!("- capsule current : current milestone snapshot");
    println!("- quest status    : quest/progress status");
    println!("- sysinfo         : era, uptime, memory summary");
    println!("- mem             : heap and memory numbers");
    println!("- cores           : online core/task counts");
    println!("- uptime / clock  : timer-derived counters");
    println!();
    println!("These are status surfaces, not runtime certification.");
    println!("Next: workspace");
}

fn print_help_network() {
    println!("Help: networking");
    println!("- net / net status            : RTL8139/static IPv4 status");
    println!("- net config                  : static IP, gateway, and limits");
    println!("- net arp                     : explain and send gateway ARP");
    println!("- net udp                     : explain UDP send boundary");
    println!("- net send                    : send default UDP payload");
    println!("- net send <ip> <port> <text> : send custom UDP payload");
    println!("- net log                     : counters and last event/error");
    println!("- net demo                    : read-only walkthrough");
    println!("- net roadmap                 : future protocol boundaries");
    println!();
    println!("Boundary: static IPv4 ARP/UDP only; no TCP, DHCP, DNS, or sockets.");
    println!("Status: partially implemented, needs runtime verification.");
    println!("Next: net status");
}

fn print_help_userspace() {
    println!("Help: userspace");
    println!("- userspace status   : current user-space boundary");
    println!("- userspace syscalls : tiny syscall ABI table");
    println!("- userspace elf      : static ELF support boundary");
    println!("- userspace roadmap  : future process work");
    println!("- ring3       : opt-in ring 3 teaching demo");
    println!("- syshello    : syscall-style hello demo");
    println!("- exec <name> : run a static ELF64 file from ChronoFS");
    println!();
    println!("Boundary: no general userland, dynamic linker, argv/env, libc, or package model.");
    println!("Status: partially implemented, needs runtime verification.");
    println!("Next: userspace status");
}

fn print_help_labs() {
    println!("Help: debug/lab");
    println!("- mode status : show safe/demo/experimental command categories");
    println!("- beep <hz>   : PC speaker tone path");
    println!("- reboot      : immediate PS/2-controller reset request");
    println!("- fsck repair : intentional ChronoFS repair verification");
    println!("- ring3/syshello/exec <name> : userspace verification paths");
    println!("- net arp / net send         : ARP/UDP verification paths");
    println!();
    println!("Future: crash lab is roadmap/design-only, not a current command.");
    println!("Safe mode is warning-only; it does not block commands.");
    println!("Next: guide next");
}

fn print_help_roadmap() {
    println!("Help: roadmap/future");
    println!("- capsule next   : next recommended milestones");
    println!("- poster roadmap : screenshot-friendly roadmap card");
    println!("- tour future    : beginner-friendly future-work explanation");
    println!();
    println!("Roadmap/design-only: TCP, DHCP, DNS, USB, dynamic linker,");
    println!("package manager, full compositor, and preemptive scheduler.");
    println!("Next: capsule next or mode status");
}

struct HelpSearchEntry {
    command: &'static str,
    terms: &'static str,
    summary: &'static str,
}

const HELP_SEARCH_INDEX: &[HelpSearchEntry] = &[
    HelpSearchEntry {
        command: "workspace",
        terms: "workspace dashboard status verify shell whereami shortcuts recent",
        summary: "Compact shell dashboard and product orientation.",
    },
    HelpSearchEntry {
        command: "shortcuts",
        terms: "shortcuts quick commands demo useful start guide apps files theme",
        summary: "Best demo and utility command list.",
    },
    HelpSearchEntry {
        command: "whereami",
        terms: "whereami context current era mode ui verification",
        summary: "Explain current era, reliability mode, and UI context.",
    },
    HelpSearchEntry {
        command: "status",
        terms: "status workspace dashboard verification doctor",
        summary: "Alias for the workspace dashboard.",
    },
    HelpSearchEntry {
        command: "verify",
        terms: "verify verification evidence qemu hardware blocked unverified",
        summary: "Read-only verification boundary summary.",
    },
    HelpSearchEntry {
        command: "files",
        terms: "files fs filesystem chronofs list info search sample demo copy rename ls cat write rm fsck journal",
        summary: "Shell-first ChronoFS usability commands.",
    },
    HelpSearchEntry {
        command: "theme",
        terms: "theme era themes travel poster eras 1984 1995 2007 2040",
        summary: "Era/theme command card.",
    },
    HelpSearchEntry {
        command: "apps",
        terms: "apps launcher featured recent category info help demo notes calc sysinfo files theme tasks verification risk roadmap",
        summary: "Static app registry and shell-first app launcher.",
    },
    HelpSearchEntry {
        command: "learn",
        terms: "learn learning curriculum map progress beginner advanced boot memory filesystem userspace networking explain glossary",
        summary: "Structured learning paths and progress map.",
    },
    HelpSearchEntry {
        command: "explain",
        terms: "explain glossary term kernel bootloader framebuffer interrupt heap filesystem journal syscall userspace elf arp udp scheduler smp",
        summary: "Short OS glossary explanations.",
    },
    HelpSearchEntry {
        command: "museum index",
        terms: "museum index exhibits boot kernel memory filesystem userspace syscalls elf networking smp scheduler",
        summary: "Index of museum learning pages.",
    },
    HelpSearchEntry {
        command: "quest badges",
        terms: "quest badges dependencies progress learning achievements inventory",
        summary: "Static quest badges and dependency route.",
    },
    HelpSearchEntry {
        command: "safe status",
        terms: "safe mode reliability warning demo experimental",
        summary: "Current warning-only reliability mode.",
    },
];

fn print_help_search(topic: &str) {
    let term = topic.strip_prefix("search").unwrap_or("").trim();
    if term.is_empty() {
        println!("Usage: help search <term>");
        println!("Try: help search fs, help search app, help search verify");
        return;
    }

    println!("Help search: {}", term);
    let mut matches = 0;
    for entry in HELP_SEARCH_INDEX {
        if contains_ignore_ascii_case(entry.terms, term)
            || contains_ignore_ascii_case(entry.command, term)
            || contains_ignore_ascii_case(entry.summary, term)
        {
            println!("- {}: {}", entry.command, entry.summary);
            matches += 1;
        }
    }

    if matches == 0 {
        println!("No direct matches.");
        println!("Try: help workspace, help apps, help fs, help system");
    }
}

fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() {
        return true;
    }
    if n.len() > h.len() {
        return false;
    }

    let mut start = 0;
    while start + n.len() <= h.len() {
        let mut offset = 0;
        while offset < n.len() && ascii_eq_ignore_case(h[start + offset], n[offset]) {
            offset += 1;
        }
        if offset == n.len() {
            return true;
        }
        start += 1;
    }

    false
}

fn ascii_eq_ignore_case(left: u8, right: u8) -> bool {
    ascii_lower(left) == ascii_lower(right)
}

fn ascii_lower(byte: u8) -> u8 {
    if byte >= b'A' && byte <= b'Z' {
        byte + 32
    } else {
        byte
    }
}

fn print_unknown_help_topic(topic: &str) {
    println!("Unknown help topic: {}", topic);
    if let Some(suggestion) = suggest_command(topic) {
        println!("Did you mean: help {}?", suggestion);
    }
    println!("Try: help workspace|start|learn|mode|apps|fs|system|network|userspace");
    println!("Search: help search <term>");
}

fn print_unknown_command(command: &str) {
    println!("unknown command: {}", command);

    if let Some(suggestion) = suggest_command(command) {
        println!("Did you mean: {}?", suggestion);
    } else if command.starts_with("status") || command.starts_with("verify") {
        println!("Hint: use workspace, verify, doctor, or help system.");
    } else if command.starts_with("file") || command.starts_with("files") {
        println!("Hint: use files or help fs for ls, cat, write, rm, fsck, and journal.");
    } else if command.starts_with("app") {
        println!("Hint: use apps or help apps.");
    } else if command.starts_with("net") {
        println!("Hint: use net or help network.");
    } else if command.starts_with("guide") {
        println!("Hint: use guide or help start.");
    } else if command.starts_with("learn") {
        println!("Hint: use learn or help learn.");
    }

    println!("Try: help, shortcuts, or workspace");
    println!("Search: help search <term>");
}

struct CommandSuggestion {
    typo: &'static str,
    suggestion: &'static str,
}

const COMMAND_TYPO_SUGGESTIONS: &[CommandSuggestion] = &[
    CommandSuggestion {
        typo: "hlep",
        suggestion: "help",
    },
    CommandSuggestion {
        typo: "apss",
        suggestion: "apps",
    },
    CommandSuggestion {
        typo: "verfy",
        suggestion: "verify",
    },
    CommandSuggestion {
        typo: "lern",
        suggestion: "learn",
    },
    CommandSuggestion {
        typo: "workspce",
        suggestion: "workspace",
    },
    CommandSuggestion {
        typo: "whreami",
        suggestion: "whereami",
    },
];

const COMMON_COMMANDS: &[&str] = &[
    "help",
    "workspace",
    "shortcuts",
    "whereami",
    "status",
    "verify",
    "recent",
    "apps",
    "learn",
    "guide",
    "files",
    "theme",
    "safe",
    "doctor",
    "fs",
    "journal",
    "windows",
];

fn suggest_command(command: &str) -> Option<&'static str> {
    let command = command.trim();

    for suggestion in COMMAND_TYPO_SUGGESTIONS {
        if command == suggestion.typo {
            return Some(suggestion.suggestion);
        }
    }

    for candidate in COMMON_COMMANDS {
        if is_near_command(command, candidate) {
            return Some(candidate);
        }
    }

    None
}

fn is_near_command(input: &str, candidate: &str) -> bool {
    let input = input.as_bytes();
    let candidate = candidate.as_bytes();

    if input.len() == candidate.len() {
        let mut mismatches = 0;
        let mut index = 0;
        while index < input.len() {
            if !ascii_eq_ignore_case(input[index], candidate[index]) {
                mismatches += 1;
            }
            index += 1;
        }
        return mismatches <= 1 || is_single_transposition(input, candidate);
    }

    if input.len() + 1 == candidate.len() {
        return is_one_missing_byte(input, candidate);
    }

    if candidate.len() + 1 == input.len() {
        return is_one_missing_byte(candidate, input);
    }

    false
}

fn is_single_transposition(input: &[u8], candidate: &[u8]) -> bool {
    if input.len() != candidate.len() || input.len() < 2 {
        return false;
    }

    let mut index = 0;
    while index + 1 < input.len() {
        if !ascii_eq_ignore_case(input[index], candidate[index]) {
            return ascii_eq_ignore_case(input[index], candidate[index + 1])
                && ascii_eq_ignore_case(input[index + 1], candidate[index])
                && ranges_equal_ignore_case(input, candidate, index + 2);
        }
        index += 1;
    }

    false
}

fn is_one_missing_byte(shorter: &[u8], longer: &[u8]) -> bool {
    let mut short_index = 0;
    let mut long_index = 0;
    let mut skipped = false;

    while short_index < shorter.len() && long_index < longer.len() {
        if ascii_eq_ignore_case(shorter[short_index], longer[long_index]) {
            short_index += 1;
            long_index += 1;
        } else if skipped {
            return false;
        } else {
            skipped = true;
            long_index += 1;
        }
    }

    true
}

fn ranges_equal_ignore_case(left: &[u8], right: &[u8], start: usize) -> bool {
    let mut index = start;
    while index < left.len() {
        if !ascii_eq_ignore_case(left[index], right[index]) {
            return false;
        }
        index += 1;
    }
    true
}

fn run_mode_command(command: &str) {
    let mode = command.strip_prefix("mode").unwrap_or("").trim();

    match mode {
        "" | "status" => print_reliability_status(),
        "safe" => set_reliability_mode(ReliabilityMode::Safe),
        "demo" => set_reliability_mode(ReliabilityMode::Demo),
        "experimental" => set_reliability_mode(ReliabilityMode::Experimental),
        "help" => print_mode_usage(),
        _ => print_mode_usage(),
    }
}

fn run_safe_command(command: &str) {
    let mode = command.strip_prefix("safe").unwrap_or("").trim();

    match mode {
        "" | "status" => print_reliability_status(),
        "on" => set_reliability_mode(ReliabilityMode::Safe),
        "off" => set_reliability_mode(ReliabilityMode::Demo),
        "help" => print_safe_usage(),
        _ => print_safe_usage(),
    }
}

fn current_reliability_mode() -> ReliabilityMode {
    ReliabilityMode::from_byte(RELIABILITY_MODE.load(Ordering::Relaxed))
}

fn set_reliability_mode(mode: ReliabilityMode) {
    RELIABILITY_MODE.store(mode.as_byte(), Ordering::Relaxed);
    println!("Reliability mode: {}", mode.label());
    match mode {
        ReliabilityMode::Safe => {
            println!("Safe mode prefers read-only demo paths and stronger warnings.");
            println!("It does not block commands and is not a security sandbox.");
        }
        ReliabilityMode::Demo => {
            println!("Demo mode is the default portfolio path.");
            println!("Risky commands still warn before running.");
        }
        ReliabilityMode::Experimental => {
            println!("Experimental mode is for intentional verification/lab paths.");
            println!("Runtime evidence is still required before claiming success.");
        }
    }
}

fn print_reliability_status() {
    let mode = current_reliability_mode();

    println!("ChronoOS reliability mode");
    println!("Current mode: {}", mode.label());
    println!("Persistence: in-memory only; resets on reboot.");
    println!("Policy: warning-only, no command blocking, not a sandbox.");
    println!();
    println!("Demo-safe/read-only:");
    println!("- help, start, guide, learn, demo, tour, capsule, poster, doctor");
    println!("- apps list/info, fs status/info/check/journal, journal");
    println!("- net status/config/log/demo/roadmap, userspace status/syscalls/elf/roadmap");
    println!("- windows status/list");
    println!();
    println!("Verification/controlled mutation:");
    println!("- write, rm, notes write/clear, fsck repair");
    println!("- net arp, net send, open notes/sysinfo, windows focus/close, kill");
    println!();
    println!("Experimental/risky:");
    println!("- ring3, syshello, exec <name>, reboot, SMP/AP, UEFI/custom BIOS");
    println!("- crash/fault paths and hardware tests");
    println!();
    println!("Next: guide next or learn roadmap");
}

fn print_mode_usage() {
    println!("Usage: mode [status|safe|demo|experimental|help]");
    println!("Aliases: safe, safe status, safe on, safe off");
}

fn print_safe_usage() {
    println!("Usage: safe [status|on|off|help]");
    println!("safe on -> mode safe; safe off -> mode demo");
}

fn print_workspace_dashboard() {
    let profile = theme::active_profile();
    let mode = current_reliability_mode();
    let status = fs::status();

    println!("ChronoOS workspace");
    println!("Era/theme: {}", profile.name);
    println!("Prompt: {}", profile.screen_prompt);
    println!("Mode: {} (warning-only)", mode.label());
    println!();
    println!("Verified core:");
    println!("- single-core BIOS boot, serial logs, framebuffer shell");
    println!("- onboarding/status/app registry screenshots");
    println!("- ChronoFS CRUD/check/journal on disposable QEMU image");
    println!();
    println!("Apps:");
    println!("- apps, apps list, notes, calc, sysinfo");
    println!("- open notes/open sysinfo are partial window paths");
    println!();
    println!("ChronoFS:");
    println!(
        "- disk: {}",
        if status.disk_available { "available" } else { "not available" }
    );
    println!(
        "- mode: {}",
        if status.persistent { "persistent ATA disk" } else { "memory fallback" }
    );
    println!(
        "- files: visible={} slots={}/{} journal={}",
        status.visible_file_count,
        status.used_file_slots,
        status.max_files,
        status.journal.state
    );
    println!();
    println!("Suggested next command: shortcuts");
    println!("Learning suggestion: learn next");
    println!("Verification note: workspace is read-only; it does not run live probes.");
}

fn print_shortcuts() {
    println!("ChronoOS shortcuts");
    println!("Start and help:");
    println!("- start        : first-run welcome");
    println!("- guide        : guided shell map");
    println!("- help         : categorized command list");
    println!("- shortcuts    : this command list");
    println!();
    println!("Product workspace:");
    println!("- workspace    : compact dashboard");
    println!("- whereami     : current shell context");
    println!("- recent       : commands typed since boot");
    println!("- apps         : text app launcher");
    println!("- files        : ChronoFS command card");
    println!("- theme        : era/theme command card");
    println!();
    println!("Learning and status:");
    println!("- learn        : curriculum paths");
    println!("- demo         : read-only product preview");
    println!("- status       : alias for workspace");
    println!("- verify       : verification boundary summary");
    println!("- safe status  : warning-only reliability mode");
}

fn print_whereami() {
    let profile = theme::active_profile();
    let mode = current_reliability_mode();

    println!("Where am I?");
    println!("Mode: {}", mode.label());
    println!("Era: {}", profile.name);
    println!("Prompt: {}", profile.screen_prompt);
    println!("UI context: framebuffer shell with tiny fixed-capacity windows.");
    println!("Safe status: reliability mode is warning-only, not a sandbox.");
    println!("Verification status: BIOS shell is the safest verified demo base.");
    println!("Unverified nearby: manual typing, Backspace/Shift, mouse drag/close, tasks/kill.");
    println!("Suggested next action: workspace or shortcuts");
}

fn print_verify_summary() {
    println!("ChronoOS verification summary");
    println!("This is a read-only summary, not a live certification run.");
    println!();
    println!("Verified in QEMU:");
    println!("- single-core BIOS boot, serial output, framebuffer shell");
    println!("- start, guide quick, mode status, safe on, doctor, apps list");
    println!("- disposable ChronoFS CRUD/check/journal/delete-persistence path");
    println!("- userspace status/syscalls screens and fixed ring3 teaching demo");
    println!();
    println!("Partially verified or blocked:");
    println!("- window open/list/focus and serial-backed close need lifecycle follow-up");
    println!("- syshello and static ELF exec remain unverified/tooling-blocked");
    println!("- UEFI loader starts under OVMF but fails with Out of Resources");
    println!("- no hardware proof yet");
    println!();
    println!("Next verification target: input/window lifecycle cleanup.");
}

fn print_files_workspace() {
    println!("ChronoFS workspace");
    println!("Use these commands to inspect files without guessing the raw fs syntax.");
    println!();
    println!("- files list          : visible files with byte sizes");
    println!("- files info <name>   : size, storage mode, and safety note");
    println!("- files search <term> : match filenames and UTF-8 file contents");
    println!("- files sample        : read-only sample file ideas");
    println!("- files demo          : guided demo script, no automatic writes");
    println!("- files copy <s> <d>  : non-overwriting copy through ChronoFS");
    println!("- files rename <o> <n>: deferred safe-rename explanation");
    println!();
    println!("Compatibility: ls, cat, write, rm, fs check, and journal still work.");
    println!("Verification: files namespace is code-present; QEMU proof is still needed.");
    println!("Safety: repair/recovery and independent write persistence remain unverified.");
    println!();
    print_files_app_card();
    println!("Quick check: files list or fs status");
}

fn print_theme_workspace() {
    println!("Workspace alias: theme");
    print_theme_app_card();
    println!("Quick switch: era 1995 or travel 2004");
}

pub fn print_reliability_warning(area: &str) {
    match current_reliability_mode() {
        ReliabilityMode::Safe => {
            println!(
                "Reliability mode: safe - {} is intentional verification, not the safe demo path.",
                area
            );
        }
        ReliabilityMode::Demo => {
            println!(
                "Reliability mode: demo - {} is outside the default demo-safe path.",
                area
            );
        }
        ReliabilityMode::Experimental => {
            println!(
                "Reliability mode: experimental - {} may run, but still needs evidence.",
                area
            );
        }
    }
}

fn print_userspace_warning() {
    print_reliability_warning("userspace demos");
    println!("Warning: userspace demos are partially implemented and need runtime verification.");
    println!("For the current boundary, run: userspace status");
}

fn run_userspace_command(command: &str) {
    let mode = command.strip_prefix("userspace").unwrap_or("").trim();

    match mode {
        "" | "status" => print_userspace_status(),
        "syscalls" => print_userspace_syscalls(),
        "elf" => print_userspace_elf(),
        "roadmap" => print_userspace_roadmap(),
        "help" => print_userspace_namespace_help(),
        _ => {
            println!("Usage: userspace [status|syscalls|elf|roadmap|help]");
            println!("Risky demos: ring3, syshello, exec <name>");
        }
    }
}

fn print_userspace_status() {
    println!("ChronoOS userspace status");
    println!("Ring 3 demo: implemented in code, not verified");
    println!("Syscalls: write/read/exit/uptime, not verified");
    println!("Static ELF exec: foreground only, not verified");
    println!(
        "Active ELF process: {}",
        if process::is_active() { "yes" } else { "no" }
    );
    println!("Scheduler boundary: cooperative kernel/app tasks, not preemptive user processes");
    println!("Not supported: fork, argv/env, dynamic linker, package manager, libc");
    println!("Next: userspace syscalls");
}

fn print_userspace_syscalls() {
    println!("ChronoOS syscall table");
    println!("No  Name       Inputs                 Outputs              Status");
    println!("1   write      fd, buffer, len        bytes or error       code-present");
    println!("2   read       fd, buffer, len        bytes or error       code-present");
    println!("3   exit       code                   returns/parks        code-present");
    println!("4   uptime     none                   PIT ticks            code-present");
    println!("Verification: implemented in code, not verified in recorded QEMU passes.");
    println!("ABI: rax=number, rdi/rsi/rdx=args, rax=return.");
}

fn print_userspace_elf() {
    println!("Static ELF support");
    println!("Supported: ELF64 little-endian ET_EXEC for x86_64 with PT_LOAD segments.");
    println!("Memory: user ELF window starts at 0x0000008000000000.");
    println!("Stack: a small mapped user stack is created for the foreground program.");
    println!("Command: exec <name> reads bytes from ChronoFS and enters the ELF entry.");
    println!("Not supported: dynamic linking, relocations, argv/env, libc, packages.");
    println!("Status: implemented in code, not verified.");
}

fn print_userspace_roadmap() {
    println!("Userspace roadmap");
    println!("- Verify ring3, syshello, and exec hello.elf one at a time");
    println!("- Add clearer process status only after runtime evidence");
    println!("- Future: argv/env, process table, app loading, safer lifecycle");
    println!("- Roadmap/design-only: dynamic linker, package manager, preemptive scheduler");
}

fn print_userspace_namespace_help() {
    println!("Userspace inspection commands");
    println!("- userspace / userspace status : current boundary and active state");
    println!("- userspace syscalls           : syscall ABI table");
    println!("- userspace elf                : static ELF support boundary");
    println!("- userspace roadmap            : future process work");
    println!("- ring3, syshello, exec <name> : risky verification demos");
    println!("The userspace namespace is read-only.");
}

fn run_ring3_with_warning() {
    print_userspace_warning();
    crate::ring3::run_demo();
}

fn run_syshello_with_warning() {
    print_userspace_warning();
    crate::ring3::run_syshello();
}

fn exec_file_with_warning(command: &str) {
    if !command.strip_prefix("exec").unwrap_or("").trim().is_empty() {
        print_userspace_warning();
    }
    exec_file(command);
}

fn print_welcome() {
    let profile = theme::active_profile();

    println!("ChronoOS first-run guide");
    println!("Era lens: {}", profile.name);
    println!("Prompt: {}", profile.screen_prompt);
    println!();
    println!("Welcome to a Rust no_std x86_64 teaching OS with eras,");
    println!("museum pages, quests, tiny apps, ChronoFS, and honest status labels.");
    println!();
    println!("Start here:");
    println!("- guide quick      : five safe commands for a first screenshot");
    println!("- guide full       : the complete shell-first tour map");
    println!("- mode status      : safe/demo/experimental path guide");
    println!("- apps             : text launcher for notes, calc, sysinfo, files");
    println!("- museum boot      : learn how the machine wakes up");
    println!("- doctor           : conservative subsystem status");
    println!("- capsule current  : current build-in-public snapshot");
    println!();
    println!("Verification note: this screen is read-only and does not certify runtime behavior.");
}

fn run_guide(command: &str) {
    let topic = command.strip_prefix("guide").unwrap_or("").trim();

    match topic {
        "" => guide_overview(),
        "quick" => guide_quick(),
        "full" => guide_full(),
        "eras" => guide_eras(),
        "apps" => guide_apps(),
        "systems" => guide_systems(),
        "status" => guide_status(),
        "next" => guide_next(),
        _ => print_guide_usage(),
    }
}

fn print_guide_usage() {
    println!("Usage: guide [quick|full|eras|apps|systems|status|next]");
}

fn guide_header(title: &str) {
    let profile = theme::active_profile();

    println!("ChronoOS guide: {}", title);
    println!("Era lens: {}", profile.name);
    println!();
}

fn guide_overview() {
    guide_header("welcome map");
    println!("This guide is read-only. It points to existing commands.");
    println!();
    println!("Topics:");
    println!("- guide quick   : first five commands");
    println!("- guide full    : full demo route");
    println!("- guide eras    : time-travel themes");
    println!("- guide apps    : launcher and tiny apps");
    println!("- guide systems : museum and OS concepts");
    println!("- guide status  : conservative verification/status surfaces");
    println!("- guide next    : safe next steps and risky commands");
}

fn guide_quick() {
    guide_header("quick start");
    println!("Try this first:");
    println!("1. about        - identify ChronoOS");
    println!("2. era          - see available eras");
    println!("3. apps         - open the text app launcher");
    println!("4. museum boot  - learn the boot story");
    println!("5. doctor       - read conservative subsystem status");
    println!();
    println!("Then try: guide full");
}

fn guide_full() {
    guide_header("full route");
    println!("Follow this shell-first path:");
    println!("1. demo              - safe high-level preview");
    println!("2. tour              - choose boot, memory, files, apps, userspace");
    println!("3. capsule           - build-in-public timeline");
    println!("4. poster            - screenshot-friendly cards");
    println!("5. apps              - notes, calc, sysinfo, files, museum, theme");
    println!("6. museum filesystem - deeper OS explanation");
    println!("7. quest status      - RPG-style progress");
    println!("8. fsck              - read-only filesystem check");
    println!("9. journal           - ChronoFS journal status");
    println!("10. learn next       - structured next curriculum step");
}

fn guide_eras() {
    guide_header("eras");
    println!("ChronoOS can shift presentation across computing eras.");
    println!("Commands:");
    println!("- era 1984 | era 1995 | era 2007 | era 2040");
    println!("- travel 1987 | travel 1998 | travel 2004 | travel 2049");
    println!("- poster eras");
    println!();
    println!("Era switching changes style and mood, not the underlying kernel.");
}

fn guide_apps() {
    guide_header("apps");
    println!("Apps are small shell-first workflows, not a full desktop.");
    println!("Commands:");
    println!("- apps");
    println!("- notes | notes write <text> | notes read");
    println!("- calc 6 * 7");
    println!("- sysinfo");
    println!("- open notes | open sysinfo");
    println!();
    println!("Window paths are partially implemented and need runtime verification.");
}

fn guide_systems() {
    guide_header("systems");
    println!("Use museum and tour commands to learn what each subsystem means.");
    println!("Museum:");
    println!("- museum boot|kernel|memory|interrupts|filesystem");
    println!("- museum userspace|networking|scheduler");
    println!("Tours:");
    println!("- tour boot");
    println!("- tour files");
    println!("- tour userspace");
    println!("Curriculum:");
    println!("- learn boot|memory|filesystem|userspace|networking");
}

fn guide_status() {
    guide_header("status");
    println!("These commands show conservative status, not full certification:");
    println!("- doctor          : subsystem report without live probes");
    println!("- poster system   : screenshot-friendly status card");
    println!("- capsule current : current milestone snapshot");
    println!("- quest status    : next verification quest");
    println!("- fsck            : read-only ChronoFS check");
    println!("- journal         : ChronoFS journal status");
    println!();
    println!("Only QEMU or hardware evidence upgrades a feature to verified.");
}

fn guide_next() {
    guide_header("next steps");
    println!("Safe demo commands:");
    println!("- guide quick | demo | tour | capsule | poster | apps");
    println!("- museum boot | museum filesystem | quest list");
    println!("- ls | fsck | journal");
    println!();
    println!("Intentional verification only:");
    println!("- ring3 | syshello | exec <name>");
    println!("- net arp | net send");
    println!("- fsck repair");
    println!("- SMP/multicore, UEFI, custom BIOS, crash/fault paths");
    println!();
    println!("Use mode status to separate safe, verification, and experimental paths.");
}

struct LearningPath {
    key: &'static str,
    title: &'static str,
    summary: &'static str,
    status: &'static str,
    verification: &'static str,
    commands: &'static [&'static str],
    next: &'static str,
}

struct LearningMapArea {
    area: &'static str,
    status: &'static str,
    verification: &'static str,
    suggested: &'static str,
    related: &'static str,
}

const LEARNING_MAP: &[LearningMapArea] = &[
    LearningMapArea {
        area: "Boot",
        status: "implemented in code",
        verification: "verified in QEMU for single-core BIOS path",
        suggested: "learn boot",
        related: "museum boot",
    },
    LearningMapArea {
        area: "Memory",
        status: "implemented in code",
        verification: "heap reuse still needs runtime verification",
        suggested: "learn memory",
        related: "mem",
    },
    LearningMapArea {
        area: "Interrupts/Input",
        status: "implemented in code",
        verification: "keyboard partial; mouse/window input needs proof",
        suggested: "learn interrupts",
        related: "doctor",
    },
    LearningMapArea {
        area: "Filesystem",
        status: "implemented in code",
        verification: "ChronoFS CRUD partial; repair/recovery unverified",
        suggested: "learn filesystem",
        related: "fs status",
    },
    LearningMapArea {
        area: "Apps/UI",
        status: "partially implemented",
        verification: "app launcher partial; window lifecycle needs proof",
        suggested: "learn gui",
        related: "apps featured",
    },
    LearningMapArea {
        area: "Userspace",
        status: "partially implemented",
        verification: "ring3 teaching demo verified; exec blocked",
        suggested: "learn userspace",
        related: "userspace status",
    },
    LearningMapArea {
        area: "Networking",
        status: "partially implemented",
        verification: "RTL8139 init/MAC partial; ARP/UDP unverified",
        suggested: "learn networking",
        related: "net status",
    },
    LearningMapArea {
        area: "Scheduler/SMP",
        status: "partially implemented",
        verification: "BSP-only SMP evidence; AP/tasks need proof",
        suggested: "learn scheduler",
        related: "cores",
    },
    LearningMapArea {
        area: "Roadmap/Future",
        status: "roadmap/design-only",
        verification: "not runtime behavior",
        suggested: "learn roadmap",
        related: "capsule next",
    },
];

const LEARNING_PATHS: &[LearningPath] = &[
    LearningPath {
        key: "boot",
        title: "Boot Path",
        summary: "Learn how firmware, the bootloader, and the kernel hand control forward.",
        status: "implemented in code",
        verification: "single-core BIOS boot has QEMU evidence; UEFI/custom BIOS still need verification",
        commands: &["museum boot", "museum kernel", "poster boot", "capsule current"],
        next: "museum boot",
    },
    LearningPath {
        key: "memory",
        title: "Memory And Heap",
        summary: "Learn how ChronoOS talks about RAM, pages, and the reusable heap.",
        status: "implemented in code",
        verification: "memory reporting builds; heap reuse still needs runtime verification",
        commands: &["museum memory", "tour memory", "mem", "doctor"],
        next: "mem",
    },
    LearningPath {
        key: "interrupts",
        title: "Interrupts And Input",
        summary: "Learn how CPU events, timers, keyboard input, and mouse packets reach the kernel.",
        status: "implemented in code",
        verification: "narrow keyboard input has QEMU evidence; polling fallback and mouse/window behavior need more checks",
        commands: &["museum interrupts", "museum keyboard", "help system", "doctor"],
        next: "museum interrupts",
    },
    LearningPath {
        key: "filesystem",
        title: "Filesystem And ChronoFS",
        summary: "Learn how tiny named files, fsck, repair boundaries, and the journal fit together.",
        status: "implemented in code",
        verification: "implemented in code, not runtime-verified for shell workflows",
        commands: &["tour files", "museum filesystem", "fs status", "fs check", "journal"],
        next: "fs status",
    },
    LearningPath {
        key: "gui",
        title: "Apps And Windowing",
        summary: "Learn how shell apps, the static app registry, and tiny windows form a small UI layer.",
        status: "partially implemented",
        verification: "apps are partially verified; window lifecycle commands still need runtime verification",
        commands: &["apps", "apps list", "open notes", "windows status", "tour apps"],
        next: "apps",
    },
    LearningPath {
        key: "userspace",
        title: "Userspace, Syscalls, And ELF",
        summary: "Learn the boundary between Ring 3 demos, tiny syscalls, and static ELF execution.",
        status: "partially implemented",
        verification: "implemented in code, not runtime-verified in recorded QEMU passes",
        commands: &["userspace status", "userspace syscalls", "museum userspace", "tour userspace"],
        next: "userspace status",
    },
    LearningPath {
        key: "networking",
        title: "Networking",
        summary: "Learn the static IPv4, ARP, UDP, and RTL8139 teaching stack before future protocols.",
        status: "partially implemented",
        verification: "RTL8139 init/MAC has QEMU evidence; ARP/UDP behavior needs runtime verification",
        commands: &["help network", "net status", "net config", "net log", "museum networking"],
        next: "net status",
    },
    LearningPath {
        key: "scheduler",
        title: "Scheduler And SMP",
        summary: "Learn how cooperative tasks differ from AP startup and future scheduling work.",
        status: "partially implemented",
        verification: "BSP-only SMP evidence exists; AP startup and task lifecycle still need checks",
        commands: &["cores", "tasks", "museum scheduler", "museum smp", "capsule current"],
        next: "cores",
    },
    LearningPath {
        key: "eras",
        title: "Themes And Eras",
        summary: "Learn how ChronoOS changes presentation across computing eras without changing the kernel.",
        status: "implemented in code",
        verification: "era commands exist; full visual theme walkthrough still needs runtime verification",
        commands: &["era", "travel <year>", "poster eras", "apps theme"],
        next: "era",
    },
    LearningPath {
        key: "roadmap",
        title: "Roadmap And Future Systems",
        summary: "Learn what is intentionally future work so ChronoOS stays honest and understandable.",
        status: "roadmap/design-only",
        verification: "not applicable; roadmap screens do not prove runtime behavior",
        commands: &["mode status", "capsule next", "poster roadmap", "tour future", "apps roadmap", "net roadmap"],
        next: "capsule next",
    },
];

fn run_learn(command: &str) {
    let topic = command.strip_prefix("learn").unwrap_or("").trim();

    if topic.is_empty() {
        print_learn_overview();
        return;
    }

    if topic == "next" {
        print_learn_next();
        return;
    }

    match topic {
        "map" => {
            print_learn_map();
            return;
        }
        "progress" => {
            print_learn_progress();
            return;
        }
        "beginner" => {
            print_learn_beginner();
            return;
        }
        "advanced" => {
            print_learn_advanced();
            return;
        }
        _ => {}
    }

    let canonical = match topic {
        "fs" | "files" => "filesystem",
        "apps" | "windows" | "windowing" => "gui",
        "network" | "net" => "networking",
        "smp" | "tasks" => "scheduler",
        "theme" | "themes" | "era" => "eras",
        "future" => "roadmap",
        other => other,
    };

    match LEARNING_PATHS.iter().find(|path| path.key == canonical) {
        Some(path) => print_learning_path(path),
        None => print_learn_usage(),
    }
}

fn print_learn_overview() {
    println!("ChronoOS learning paths");
    println!("Pick a subsystem and follow the suggested commands.");
    println!("These paths are read-only; they do not certify runtime behavior.");
    println!();
    for path in LEARNING_PATHS {
        println!("- learn {:<11} : {}", path.key, path.title);
    }
    println!("- learn map         : status and verification map");
    println!("- learn progress    : static learning progress summary");
    println!("- learn beginner    : safe first route");
    println!("- learn advanced    : intentional verification route");
    println!("- learn next        : safest next path");
}

fn print_learning_path(path: &LearningPath) {
    println!("Learn: {}", path.title);
    println!("{}", path.summary);
    println!("Status: {}", path.status);
    println!("Verification: {}", path.verification);
    println!();
    println!("Try:");
    for command in path.commands {
        println!("- {}", command);
    }
    println!();
    println!("Safe next command: {}", path.next);
}

fn print_learn_next() {
    println!("ChronoOS learning path: next");
    println!("Start with the map, then follow the read-only beginner route.");
    println!("1. learn map        - see areas, status, and proof boundaries");
    println!("2. learn beginner   - safe first curriculum");
    println!("3. workspace        - current mode and verification summary");
    println!("4. quest status     - static next quest");
    println!();
    println!("Suggested next safe command: learn map");
    println!("This is not live progress tracking or certification.");
}

fn print_learn_usage() {
    println!("Usage: learn [map|progress|beginner|advanced|next]");
    println!("       learn [boot|memory|interrupts|filesystem|gui|userspace|networking|scheduler|eras|roadmap]");
}

fn print_learn_map() {
    println!("ChronoOS learning map");
    println!("Static educational map; not a runtime certification system.");
    println!();

    for area in LEARNING_MAP {
        println!("{}", area.area);
        println!("  Status: {}", area.status);
        println!("  Verification: {}", area.verification);
        println!("  Try: {}", area.suggested);
        println!("  Related: {}", area.related);
    }
}

fn print_learn_progress() {
    let total = LEARNING_MAP.len();
    let roadmap = LEARNING_MAP
        .iter()
        .filter(|area| area.status == "roadmap/design-only")
        .count();
    let active = total.saturating_sub(roadmap);

    println!("ChronoOS learning progress");
    println!("Mode: static/sessionless educational map");
    println!("Available areas: {}", total);
    println!("Implemented or partial areas: {}", active);
    println!("Roadmap/future areas: {}", roadmap);
    println!("Badges: see quest badges");
    println!("Dependencies: see quest dependencies");
    println!();
    println!("Verification boundary:");
    println!("- QEMU evidence exists for narrow BIOS/app/ChronoFS/userspace paths.");
    println!("- Code-present systems still need focused runtime proof.");
    println!("- Roadmap/future systems are not implemented behavior.");
    println!();
    println!("Next safe route: learn beginner");
}

fn print_learn_beginner() {
    println!("ChronoOS beginner route");
    println!("Read-only first pass; no probes or risky commands.");
    println!("1. learn boot       - startup story");
    println!("2. learn memory     - RAM, paging, heap");
    println!("3. learn filesystem - ChronoFS status and journal");
    println!("4. learn gui        - apps and tiny windows");
    println!("5. workspace        - current shell dashboard");
    println!("6. verify           - evidence boundary summary");
    println!();
    println!("Suggested next safe command: learn boot");
}

fn print_learn_advanced() {
    println!("ChronoOS advanced route");
    println!("Intentional verification map; commands may need controlled QEMU evidence.");
    println!("1. learn userspace  - Ring 3, syscalls, static ELF boundary");
    println!("2. userspace status - read-only userspace summary");
    println!("3. learn networking - RTL8139, ARP, UDP boundaries");
    println!("4. learn scheduler  - cooperative tasks and SMP/AP boundary");
    println!("5. fs check         - read-only ChronoFS check");
    println!("6. journal          - one-record journal status");
    println!("7. capsule next     - UEFI/custom boot and future tracks");
    println!();
    println!("Avoid in casual demos: fsck repair, net send, syshello, exec <name>.");
}

struct Explanation {
    term: &'static str,
    lines: &'static [&'static str],
    command: &'static str,
}

const EXPLANATIONS: &[Explanation] = &[
    Explanation {
        term: "kernel",
        lines: &[
            "The kernel is the core OS program.",
            "It manages hardware, memory, interrupts, files, and the shell.",
        ],
        command: "museum kernel",
    },
    Explanation {
        term: "bootloader",
        lines: &[
            "A bootloader is the tiny program firmware starts first.",
            "It prepares the machine and jumps into the kernel.",
        ],
        command: "learn boot",
    },
    Explanation {
        term: "framebuffer",
        lines: &[
            "A framebuffer is memory that represents pixels on screen.",
            "ChronoOS writes text UI into it after boot.",
        ],
        command: "poster boot",
    },
    Explanation {
        term: "interrupt",
        lines: &[
            "An interrupt is a signal asking the CPU for attention.",
            "Timers, keyboard input, mouse packets, and faults use this idea.",
        ],
        command: "learn interrupts",
    },
    Explanation {
        term: "heap",
        lines: &[
            "The heap is kernel-managed memory for growing data.",
            "ChronoOS has a reusable heap, but reuse still needs stress proof.",
        ],
        command: "mem",
    },
    Explanation {
        term: "filesystem",
        lines: &[
            "A filesystem gives names and structure to bytes on disk.",
            "ChronoFS is ChronoOS's tiny educational file format.",
        ],
        command: "learn filesystem",
    },
    Explanation {
        term: "journal",
        lines: &[
            "A journal records an intended metadata change.",
            "ChronoFS uses a tiny one-record journal for write/remove safety.",
        ],
        command: "journal",
    },
    Explanation {
        term: "syscall",
        lines: &[
            "A syscall is a controlled entry from userspace into the kernel.",
            "ChronoOS has a small teaching syscall boundary.",
        ],
        command: "userspace syscalls",
    },
    Explanation {
        term: "userspace",
        lines: &[
            "Userspace is where less-privileged programs run.",
            "ChronoOS has teaching demos, not a full process platform.",
        ],
        command: "userspace status",
    },
    Explanation {
        term: "elf",
        lines: &[
            "ELF is an executable file format.",
            "ChronoOS has a static ELF path, but execution still needs proof.",
        ],
        command: "userspace elf",
    },
    Explanation {
        term: "arp",
        lines: &[
            "ARP maps an IPv4 address to an Ethernet MAC address.",
            "ChronoOS exposes ARP as a teaching/status path.",
        ],
        command: "net arp",
    },
    Explanation {
        term: "udp",
        lines: &[
            "UDP sends small datagrams without a connection.",
            "ChronoOS has ARP/UDP code paths, but behavior needs proof.",
        ],
        command: "net udp",
    },
    Explanation {
        term: "scheduler",
        lines: &[
            "A scheduler chooses which task runs next.",
            "ChronoOS has cooperative task teaching paths, not preemption.",
        ],
        command: "learn scheduler",
    },
    Explanation {
        term: "smp",
        lines: &[
            "SMP means multiple CPU cores can participate.",
            "ChronoOS has BSP-only evidence; AP startup still needs proof.",
        ],
        command: "museum smp",
    },
];

fn run_explain_command(command: &str) {
    let term = command.strip_prefix("explain").unwrap_or("").trim();
    if term.is_empty() {
        print_explain_usage();
        return;
    }

    let Some(explanation) = EXPLANATIONS
        .iter()
        .find(|entry| entry.term.eq_ignore_ascii_case(term))
    else {
        println!("No short explanation for '{}'.", term);
        print_explain_usage();
        return;
    };

    println!("Explain: {}", explanation.term);
    for line in explanation.lines {
        println!("{}", line);
    }
    println!("Try: {}", explanation.command);
}

fn print_explain_usage() {
    println!("Usage: explain <term>");
    println!("Terms: kernel, bootloader, framebuffer, interrupt, heap");
    println!("       filesystem, journal, syscall, userspace, ELF");
    println!("       ARP, UDP, scheduler, SMP");
}

fn run_demo(command: &str) {
    let rest = command.strip_prefix("demo").unwrap_or("").trim();
    if !rest.is_empty() {
        println!("Usage: demo");
        return;
    }

    let profile = theme::active_profile();

    println!("ChronoOS demo");
    println!("This guide is text-only. It does not change system state.");
    println!();

    println!("[1] Current era");
    println!("Active era: {}", profile.name);
    println!("Prompt: {}", profile.screen_prompt);
    println!();

    println!("[2] Era switching preview");
    println!("Try later: era 1984 | era 1995 | era 2007 | era 2040");
    println!("The demo only previews these commands; it does not switch era.");
    println!();

    println!("[3] Museum mode preview");
    println!("Explore: museum boot|kernel|memory|interrupts|keyboard|serial|era");
    println!("Deep dives: museum disk|filesystem|userspace|syscalls|elf|networking|smp|scheduler");
    println!("These pages explain the OS pieces in small, readable steps.");
    println!();

    println!("[4] Filesystem preview");
    println!("Read-only tour commands: ls, cat <name>, fsck, journal");
    println!("Changing commands: write <name> <content>, rm <name>, fsck repair");
    print_demo_files();
    println!();

    println!("[5] Sysinfo preview");
    println!("Use sysinfo for a compact status view.");
    println!("Use open sysinfo to see it in a small window.");
    println!();

    println!("[6] Apps preview");
    println!("Apps: notes, calc, sysinfo");
    println!("These are shell apps, not new kernel subsystems.");
    println!();

    println!("[7] Advanced preview");
    println!("Windows: open notes, open sysinfo");
    println!("Tasks: tasks, kill <id>");
    println!("User-space demos: ring3, syshello, exec <name>");
    println!("This guide does not spawn tasks, open windows, or execute programs.");
}

fn print_demo_files() {
    let mut printed_header = false;
    let has_files = fs::list(|name| {
        if !printed_header {
            println!("Current files:");
            printed_header = true;
        }
        println!("- {}", name);
    });

    if !has_files {
        println!("Current files: (none)");
    }
}

fn run_tour(command: &str) {
    let topic = command.strip_prefix("tour").unwrap_or("").trim();

    match topic {
        "" => tour_overview(),
        "boot" => tour_boot(),
        "memory" => tour_memory(),
        "files" => tour_files(),
        "apps" => tour_apps(),
        "userspace" => tour_userspace(),
        "future" => tour_future(),
        _ => print_tour_usage(),
    }
}

fn print_tour_usage() {
    println!("Usage: tour [boot|memory|files|apps|userspace|future]");
}

fn tour_overview() {
    let profile = theme::active_profile();

    println!("ChronoOS tour");
    println!("Active era: {}", profile.name);
    println!("Prompt style: {}", profile.screen_prompt);
    println!();
    println!("This tour explains what is already implemented in code inside the OS.");
    println!("It only reads state and prints text; it does not change files, eras, tasks, or apps.");
    println!();
    println!("Tour topics:");
    println!("- tour boot       : how the OS starts");
    println!("- tour memory     : how memory is organized");
    println!("- tour files      : how ChronoFS stores small files");
    println!("- tour apps       : shell apps and windows");
    println!("- tour userspace  : ring 3 and user programs");
    println!("- tour future     : ideas that are not finished systems yet");
}

fn tour_boot() {
    let profile = theme::active_profile();

    println!("Tour: boot");
    println!("Era lens: {}", profile.name);
    println!();
    println!("ChronoOS begins with the bootloader placing the kernel in memory.");
    println!("The kernel sets up the CPU basics, interrupts, memory services, and device input.");
    println!("After that, the shell becomes the friendly front desk for exploring the system.");
    println!();
    println!("Related commands:");
    println!("- museum boot");
    println!("- museum kernel");
    println!("- museum interrupts");
    println!("- sysinfo");
}

fn tour_memory() {
    let profile = theme::active_profile();

    println!("Tour: memory");
    println!("Era lens: {}", profile.name);
    println!();
    println!("Memory is the workspace the kernel uses while the machine is running.");
    println!("ChronoOS has implemented-in-code pieces for tracking memory, using a heap,");
    println!("and explaining frames/pages in beginner-friendly museum pages.");
    println!();
    println!("Useful commands:");
    println!("- mem");
    println!("- museum memory");
    println!("- sysinfo");
}

fn tour_files() {
    let profile = theme::active_profile();

    println!("Tour: files");
    println!("Era lens: {}", profile.name);
    println!();
    println!("ChronoFS is the small educational filesystem for ChronoOS.");
    println!("It stores named files, tracks file extents, checks consistency with fsck,");
    println!("and keeps a tiny journal for safer write/remove metadata operations.");
    println!();
    println!("Read-only commands:");
    println!("- ls");
    println!("- cat <name>");
    println!("- fsck");
    println!("- journal");
    println!();
    println!("Changing commands:");
    println!("- write <name> <content>");
    println!("- rm <name>");
    println!("- fsck repair");
    print_tour_files();
}

fn print_tour_files() {
    let mut printed_header = false;
    let has_files = fs::list(|name| {
        if !printed_header {
            println!();
            println!("Current files:");
            printed_header = true;
        }
        println!("- {}", name);
    });

    if !has_files {
        println!();
        println!("Current files: (none)");
    }
}

fn tour_apps() {
    let profile = theme::active_profile();

    println!("Tour: apps");
    println!("Era lens: {}", profile.name);
    println!();
    println!("ChronoOS has small shell apps and window previews that help show");
    println!("what an operating system can do without hiding the learning steps.");
    println!();
    println!("Code-present app commands:");
    println!("- notes");
    println!("- calc");
    println!("- sysinfo");
    println!("- open notes");
    println!("- open sysinfo");
    println!();
    println!("This tour does not open apps or windows; it only points to them.");
}

fn tour_userspace() {
    let profile = theme::active_profile();

    println!("Tour: userspace");
    println!("Era lens: {}", profile.name);
    println!();
    println!("User-space is where programs run with fewer privileges than the kernel.");
    println!("ChronoOS has implemented-in-code demos for entering ring 3, making a simple");
    println!("syscall-style hello, and executing a stored program by name.");
    println!();
    println!("Related commands:");
    println!("- ring3");
    println!("- syshello");
    println!("- exec <name>");
    println!();
    println!("This tour does not execute user programs.");
}

fn tour_future() {
    let profile = theme::active_profile();

    println!("Tour: future");
    println!("Era lens: {}", profile.name);
    println!();
    println!("These are roadmap-style ideas, not runtime-verified promises:");
    println!("- richer guided lessons");
    println!("- stronger filesystem recovery");
    println!("- clearer user-space examples");
    println!("- more museum pages that explain each subsystem");
    println!();
    println!("ChronoOS should keep growing in small, understandable steps.");
}

fn run_capsule(command: &str) {
    let topic = command.strip_prefix("capsule").unwrap_or("").trim();

    match topic {
        "" => capsule_overview(),
        "milestones" => capsule_milestones(),
        "current" => capsule_current(),
        "next" => capsule_next(),
        _ => print_capsule_usage(),
    }
}

fn print_capsule_usage() {
    println!("Usage: capsule [milestones|current|next]");
}

fn print_capsule_header(title: &str) {
    let profile = theme::active_profile();

    println!("{}", title);
    println!("Era lens: {}", profile.name);
    println!("Prompt style: {}", profile.screen_prompt);
    println!();
}

fn capsule_overview() {
    print_capsule_header("Capsule timeline");

    println!("A build-in-public map of ChronoOS.");
    println!("This command is read-only: it does not change eras, files, apps, tasks, or userspace.");
    println!();
    println!("Legend:");
    println!("- implemented in code: present in the source tree");
    println!("- partially implemented: present, but still limited or educational");
    println!("- roadmap/design-only: next ideas, not finished systems");
    println!("- needs runtime verification: still needs build or OS-shell checks");
    println!();
    println!("Open a capsule:");
    println!("- capsule milestones");
    println!("- capsule current");
    println!("- capsule next");
}

fn capsule_milestones() {
    print_capsule_header("Capsule milestones");

    println!("[implemented in code]");
    println!("- Shell command center with help, eras, museum pages, quests, and guides");
    println!("- Era themes for 1984, 1995, 2007, and 2040");
    println!("- Museum explanations for boot, kernel, memory, interrupts, keyboard, serial, and eras");
    println!("- ChronoFS basics: ls, cat, write, rm, exec, fsck, fsck repair, and journal status");
    println!("- Tiny journal and conservative fsck/repair paths for ChronoFS metadata");
    println!("- Guided learning commands: demo, tour, and capsule");
    println!("- Small app commands and previews: notes, calc, sysinfo, open notes, open sysinfo");
    println!("- User-space demos where available: ring3, syshello, and exec <name>");
    println!();

    println!("[partially implemented]");
    println!("- Window and task features are useful previews, not a complete desktop environment");
    println!("- User-space execution exists as an educational path, not a full process platform");
    println!("- ChronoFS recovery is conservative and refuses ambiguous repairs");
    println!("- Apps are intentionally small and shell-first");
    println!("- Guides explain current behavior, but still need runtime walkthrough checks");
    println!();

    println!("[roadmap/design-only]");
    println!("- Stronger build and shell verification loops");
    println!("- Richer guided lessons that connect quests, museum pages, and commands");
    println!("- Deeper app workflows while staying beginner-friendly");
    println!("- Safer filesystem recovery examples without guessing at data");
    println!("- Clearer user-space examples and documentation");
    println!();

    println!("[needs runtime verification]");
    println!("- Build sanity for the current source tree");
    println!("- Shell checks for capsule, tour, demo, and invalid command forms");
    println!("- ChronoFS checks for ls, cat, write, rm, fsck, fsck repair, and journal");
    println!("- Journal recovery scenarios on real or emulated disk state");
    println!("- App/window/task/user-space demos in the OS runtime");
}

fn capsule_current() {
    print_capsule_header("Capsule current");

    println!("[implemented in code]");
    println!("- ChronoOS can present itself through help, demo, tour, museum, and capsule text");
    println!("- ChronoFS has basic file commands, fsck/repair, and journal status support");
    println!("- Era identity and educational shell commands are part of the current experience");
    println!();

    println!("[partially implemented]");
    println!("- Some systems are intentionally small teaching versions");
    println!("- Recovery, user-space, windows, tasks, and apps still need more runtime confidence");
    println!();

    println!("[needs runtime verification]");
    println!("- Build sanity");
    println!("- OS-shell checks for capsule milestones, capsule current, capsule next, and bad input");
}

fn capsule_next() {
    print_capsule_header("Capsule next");

    println!("[recommended next steps]");
    println!("- Run a build-only check when toolchain use is available");
    println!("- Verify capsule, tour, and demo from inside the OS shell");
    println!("- Exercise ChronoFS fsck and journal status before adding broader filesystem work");
    println!("- Keep new lessons text-first until the runtime behavior is verified");
    println!();
    println!("[roadmap/design-only, not implemented here]");
    println!("- Better guided learning paths");
    println!("- Stronger recovery documentation and examples");
    println!("- More polished app and user-space demos");
}

fn run_doctor(command: &str) {
    let rest = command.strip_prefix("doctor").unwrap_or("").trim();
    if !rest.is_empty() {
        print_doctor_usage();
        return;
    }

    print_doctor_report();
}

fn print_doctor_usage() {
    println!("Usage: doctor");
}

fn print_doctor_line(name: &str, status: &str, note: &str) {
    println!("- {}: {} - {}", name, status, note);
}

fn print_doctor_report() {
    let profile = theme::active_profile();

    println!("ChronoOS doctor");
    println!("Era lens: {}", profile.name);
    println!("Prompt style: {}", profile.screen_prompt);
    println!();
    println!("Legend:");
    println!("- ok: checked safely inside this command");
    println!("- implemented in code: source paths or shell commands exist, but no live probe was run");
    println!("- partially implemented: useful teaching version with known limits");
    println!("- needs runtime verification: build or OS-shell testing is still needed");
    println!();
    println!("Subsystem health:");
    print_doctor_line(
        "serial",
        "implemented in code",
        "serial logging paths exist; doctor does not perform a port loopback test",
    );
    print_doctor_line(
        "framebuffer",
        "implemented in code",
        "shell text uses the display path; doctor does not probe graphics hardware",
    );
    print_doctor_line(
        "timer",
        "implemented in code",
        "uptime and clock commands are available; interrupt timing is not certified here",
    );
    print_doctor_line(
        "keyboard",
        "implemented in code",
        "the shell has an input path; doctor does not run an interactive key test",
    );
    print_doctor_line(
        "mouse",
        "implemented in code",
        "PS/2 mouse and window paths exist; doctor does not run a live mouse probe",
    );
    print_doctor_line(
        "filesystem",
        "implemented in code",
        "ChronoFS, fsck, repair, and journal commands exist; doctor does not run fsck",
    );
    print_doctor_line(
        "heap",
        "implemented in code",
        "memory reporting exists through mem; doctor does not allocate test blocks",
    );
    print_doctor_line(
        "network",
        "partially implemented",
        "ARP/UDP paths and net commands exist; doctor does not query NIC state",
    );
    print_doctor_line(
        "SMP/core count",
        "partially implemented",
        "core reporting and AP startup paths exist; doctor does not validate SMP",
    );
    print_doctor_line(
        "scheduler",
        "partially implemented",
        "task commands are present as teaching tools; this is not a full scheduler audit",
    );
    print_doctor_line(
        "userspace/ELF support",
        "partially implemented",
        "ring3, syshello, and exec paths exist; doctor does not execute user programs",
    );
    println!();
    println!("Doctor note: this is a conservative read-only report, not full runtime certification.");
    println!("Use build checks and OS-shell testing before treating a subsystem as verified.");
}

const NOTES_FILE_NAME: &str = "notes";
const NOTES_WRITE_COMMAND_MAX: usize = 768;

fn run_notes_app(command: &str) {
    let rest = command.strip_prefix("notes").unwrap_or("").trim();

    if rest.is_empty() {
        print_notes_home();
    } else if rest == "read" {
        notes_read();
    } else if rest == "clear" {
        notes_clear();
    } else if rest == "save" {
        notes_save();
    } else if rest == "open" {
        notes_open();
    } else if let Some(text) = rest.strip_prefix("write ") {
        notes_write(text.trim());
    } else {
        print_notes_usage();
    }
}

fn print_notes_usage() {
    println!("Usage: notes [read|write <text>|clear|save|open]");
}

fn print_notes_home() {
    let profile = theme::active_profile();

    println!("ChronoOS Notes");
    println!("Era lens: {}", profile.name);
    println!("Storage: ChronoFS file '{}'", NOTES_FILE_NAME);
    println!();
    println!("Commands:");
    println!("- notes read");
    println!("- notes write <text>");
    println!("- notes clear");
    println!("- notes save");
    println!("- notes open");
    println!();
    println!("Small note: writes are saved immediately; this is not a full editor yet.");
}

fn notes_read() {
    println!("Reading notes from '{}':", NOTES_FILE_NAME);
    execute_command("cat notes");
}

fn notes_write(text: &str) {
    if text.is_empty() {
        print_notes_usage();
        return;
    }

    let prefix = b"write notes ";
    let text_bytes = text.as_bytes();
    let needed = prefix.len() + text_bytes.len();

    if needed > NOTES_WRITE_COMMAND_MAX {
        println!("Note is too long for the small text launcher.");
        println!("Limit: {} bytes of command text.", NOTES_WRITE_COMMAND_MAX - prefix.len());
        return;
    }

    let mut command = [0u8; NOTES_WRITE_COMMAND_MAX];
    let mut index = 0;

    while index < prefix.len() {
        command[index] = prefix[index];
        index += 1;
    }

    let mut text_index = 0;
    while text_index < text_bytes.len() {
        command[index + text_index] = text_bytes[text_index];
        text_index += 1;
    }

    let total = index + text_bytes.len();
    match core::str::from_utf8(&command[..total]) {
        Ok(write_command) => {
            println!("Saving note to '{}'.", NOTES_FILE_NAME);
            execute_command(write_command);
        }
        Err(_) => {
            println!("Could not save note text.");
        }
    }
}

fn notes_clear() {
    println!("Clearing notes by removing '{}'.", NOTES_FILE_NAME);
    execute_command("rm notes");
}

fn notes_save() {
    println!("Notes are saved immediately when you run: notes write <text>");
    println!("Current storage file: '{}'", NOTES_FILE_NAME);
    println!("No unsaved draft buffer exists in this small text app yet.");
}

fn notes_open() {
    println!("Opening notes with the existing window command, if available.");
    execute_command("open notes");
}

fn run_apps_launcher(command: &str) {
    let app = command.strip_prefix("apps").unwrap_or("").trim();

    match app {
        "" | "list" => print_apps_launcher(),
        "featured" => print_apps_featured(),
        "recent" => print_recent_apps(),
        "verified" => print_apps_verified(),
        "roadmap" => print_apps_roadmap(),
        "help" => print_apps_usage(),
        app if app.starts_with("help ") => apps_help(app),
        app if app.starts_with("category ") => apps_category(app),
        app if app.starts_with("info ") => apps_info(app),
        app if app.starts_with("demo ") => apps_demo(app),
        app if app.starts_with("launch ") => apps_launch(app),
        "notes" => launch_existing_app("notes"),
        "calc" => launch_existing_app("calc"),
        "sysinfo" => launch_existing_app("sysinfo"),
        "clock" => launch_existing_app("clock"),
        "tasks" => launch_existing_app("tasks"),
        "files" => {
            record_recent_app("files");
            print_files_app_card();
        }
        "museum" => {
            record_recent_app("museum");
            print_museum_app_card();
        }
        "theme" => {
            record_recent_app("theme");
            print_theme_app_card();
        }
        _ => print_apps_usage(),
    }
}

fn print_apps_usage() {
    println!("Usage: apps [list|featured|recent|category <name>]");
    println!("       apps [info|help|demo|launch] <name>");
    println!("       apps [verified|roadmap]");
    println!("Aliases: apps notes|calc|sysinfo|files|clock|museum|theme|tasks");
    println!("Categories: Core, Files, Learning, System, Networking, Visual, Debug/Lab, Roadmap/Future");
}

fn apps_style_for_era(name: &str) -> (&'static str, &'static str, &'static str) {
    if name.contains("1984") {
        ("Desk Accessories", "[ ]", "Monochrome shelf")
    } else if name.contains("1995") {
        ("Program Manager", "[*]", "Start-menu friendly")
    } else if name.contains("2007") {
        ("Dock", "[>]", "Glossy quick launch")
    } else if name.contains("2040") {
        ("Capsule Grid", "[+]", "Future lab console")
    } else {
        ("App Launcher", "[-]", "ChronoOS shelf")
    }
}

fn print_apps_launcher() {
    let profile = theme::active_profile();
    let (title, marker, subtitle) = apps_style_for_era(profile.name);

    println!("{} - {}", title, subtitle);
    println!("Era lens: {}", profile.name);
    println!("Prompt style: {}", profile.screen_prompt);
    println!();
    println!("Launch with: apps launch <name> or apps <name>");
    println!("Inspect with: apps info <name>");
    println!("Browse with: apps featured, apps category <name>, apps recent");
    println!();

    for app in apps::registry() {
        print_app_entry(marker, app);
    }

    println!();
    println!("This is a static registry, not a package manager or dynamic loader.");
    println!("Roadmap entries are labels, not installed apps.");
}

fn print_app_entry(marker: &str, app: &apps::AppManifest) {
    println!(
        "{} {:9} - {} [{} | {} | {}]",
        marker,
        app.name,
        app.description,
        app.status.label(),
        app.verification.badge(),
        app.risk.badge()
    );
}

fn launch_existing_app(command: &str) {
    record_recent_app(command);
    println!("Launching existing command: {}", command);
    execute_command(command);
}

fn print_apps_featured() {
    println!("Featured apps");
    println!("Best shell-first demo surfaces; demo commands are not run automatically.");

    for app in apps::registry() {
        if app.featured {
            println!(
                "- {} ({}) [{} | {}]",
                app.name,
                app.category,
                app.verification.badge(),
                app.risk.badge()
            );
            println!("  Launch: {}", app.launch_command);
        }
    }

    println!("Try: apps demo notes, apps demo files, apps info status");
}

fn apps_category(command: &str) {
    let category = command.strip_prefix("category").unwrap_or("").trim();
    if category.is_empty() {
        println!("Usage: apps category <name>");
        println!("Categories: Core, Files, Learning, System, Networking, Visual, Debug/Lab, Roadmap/Future");
        return;
    }

    println!("Apps category: {}", category);
    let mut matches = 0;
    for app in apps::registry() {
        if app_category_matches(app.category, category) {
            print_app_entry("-", app);
            matches += 1;
        }
    }

    if matches == 0 {
        println!("No apps in that category.");
        println!("Try: apps category Core or apps category System");
    }
}

fn apps_info(command: &str) {
    let name = command.strip_prefix("info").unwrap_or("").trim();
    if name.is_empty() {
        println!("Usage: apps info <name>");
        return;
    }

    let Some(app) = apps::find_manifest(name) else {
        println!("No app manifest named '{}'.", name);
        println!("Try: apps list");
        return;
    };

    println!("App: {}", app.name);
    println!("Category: {}", app.category);
    println!("Description: {}", app.description);
    println!("Launch command: {}", if app.launch_command.is_empty() { "(none)" } else { app.launch_command });
    println!("Status: {}", app.status.label());
    println!("Verification: {}", app.verification.label());
    println!("Risk: {}", app.risk.label());
    print_app_command_list("Related commands", app.related_commands);
}

fn apps_help(command: &str) {
    let name = command.strip_prefix("help").unwrap_or("").trim();
    if name.is_empty() {
        println!("Usage: apps help <name>");
        return;
    }

    let Some(app) = apps::find_manifest(name) else {
        println!("No app manifest named '{}'.", name);
        println!("Try: apps list");
        return;
    };

    println!("App help: {}", app.name);
    println!("{}", app.help);
    print_app_command_list("Related commands", app.related_commands);
}

fn apps_demo(command: &str) {
    let name = command.strip_prefix("demo").unwrap_or("").trim();
    if name.is_empty() {
        println!("Usage: apps demo <name>");
        return;
    }

    let Some(app) = apps::find_manifest(name) else {
        println!("No app manifest named '{}'.", name);
        println!("Try: apps featured");
        return;
    };

    println!("App demo: {}", app.name);
    println!("This is a safe script preview; commands are not run automatically.");
    println!("Verification: {}", app.verification.label());
    println!("Risk: {}", app.risk.label());
    print_app_command_list("Demo path", app.demo_commands);
}

fn apps_launch(command: &str) {
    let name = command.strip_prefix("launch").unwrap_or("").trim();
    if name.is_empty() {
        println!("Usage: apps launch <name>");
        return;
    }

    let Some(app) = apps::find_manifest(name) else {
        println!("No app manifest named '{}'.", name);
        println!("Try: apps list");
        return;
    };

    if app.status == apps::AppStatus::Roadmap || app.launch_command.is_empty() {
        println!("{} is {}.", app.name, app.status.label());
        println!("No runtime launch command is available yet.");
        return;
    }

    if !app.launch_command.starts_with("apps ") {
        record_recent_app(app.name);
    }
    println!("Launching from registry: {}", app.launch_command);
    execute_command(app.launch_command);
}

fn print_apps_verified() {
    println!("Apps with recorded QEMU evidence");
    for app in apps::registry() {
        if app.verification == apps::VerificationStatus::PartiallyVerifiedQemu {
            println!("- {}: {}", app.name, app.verification.label());
        }
    }
    println!("Note: partial QEMU evidence is not full app verification.");
}

fn print_apps_roadmap() {
    println!("Roadmap/design-only app ideas");
    for app in apps::registry() {
        if app.status == apps::AppStatus::Roadmap {
            println!("- {}: {}", app.name, app.description);
        }
    }
    println!("No package manager, dynamic linker, or dynamic app loading exists.");
}

fn print_app_command_list(title: &str, commands: &[&str]) {
    println!("{}:", title);
    if commands.is_empty() {
        println!("- (none)");
        return;
    }

    for command in commands {
        println!("- {}", command);
    }
}

fn app_category_matches(category: &str, query: &str) -> bool {
    if contains_ignore_ascii_case(category, query) {
        return true;
    }

    match ascii_lower_str(query) {
        "core" => category == "Core",
        "files" | "file" | "storage" => category == "Files",
        "learning" | "learn" | "education" => category == "Learning",
        "system" | "status" | "systems" => category == "System",
        "networking" | "network" | "net" => category == "Networking",
        "visual" | "theme" | "themes" => category == "Visual",
        "debug" | "lab" | "debug/lab" => category == "Debug/Lab",
        "roadmap" | "future" | "roadmap/future" => category == "Roadmap/Future",
        _ => false,
    }
}

fn ascii_lower_str(input: &str) -> &'static str {
    if input.eq_ignore_ascii_case("Core") {
        "core"
    } else if input.eq_ignore_ascii_case("Files") || input.eq_ignore_ascii_case("File") {
        "files"
    } else if input.eq_ignore_ascii_case("Storage") {
        "storage"
    } else if input.eq_ignore_ascii_case("Learning") || input.eq_ignore_ascii_case("Learn") {
        "learning"
    } else if input.eq_ignore_ascii_case("Education") {
        "education"
    } else if input.eq_ignore_ascii_case("System") || input.eq_ignore_ascii_case("Systems") {
        "system"
    } else if input.eq_ignore_ascii_case("Status") {
        "status"
    } else if input.eq_ignore_ascii_case("Networking") || input.eq_ignore_ascii_case("Network") {
        "networking"
    } else if input.eq_ignore_ascii_case("Net") {
        "net"
    } else if input.eq_ignore_ascii_case("Visual") {
        "visual"
    } else if input.eq_ignore_ascii_case("Theme") || input.eq_ignore_ascii_case("Themes") {
        "theme"
    } else if input.eq_ignore_ascii_case("Debug") {
        "debug"
    } else if input.eq_ignore_ascii_case("Lab") {
        "lab"
    } else if input.eq_ignore_ascii_case("Debug/Lab") {
        "debug/lab"
    } else if input.eq_ignore_ascii_case("Roadmap") {
        "roadmap"
    } else if input.eq_ignore_ascii_case("Future") {
        "future"
    } else if input.eq_ignore_ascii_case("Roadmap/Future") {
        "roadmap/future"
    } else {
        ""
    }
}

fn print_files_app_card() {
    let profile = theme::active_profile();
    let (_, marker, _) = apps_style_for_era(profile.name);

    println!("Files app");
    println!("Era lens: {}", profile.name);
    println!();
    println!("{} Read files: ls, cat <name>", marker);
    println!("{} Change files: write <name> <content>, rm <name>", marker);
    println!("{} Check disk: fsck, fsck repair, journal", marker);
    println!();
    println!("The launcher does not edit files by itself.");
}

fn print_museum_app_card() {
    let profile = theme::active_profile();
    let (_, marker, _) = apps_style_for_era(profile.name);

    println!("Museum app");
    println!("Era lens: {}", profile.name);
    println!();
    println!("{} Open exhibits with:", marker);
    println!("museum boot|kernel|memory|interrupts|keyboard|serial|era");
    println!("museum disk|filesystem|userspace|syscalls|elf|networking|smp|scheduler");
    println!();
    println!("The launcher points to exhibits without opening a new desktop window.");
}

fn print_theme_app_card() {
    let profile = theme::active_profile();
    let (_, marker, _) = apps_style_for_era(profile.name);

    println!("Theme app");
    println!("Current era: {}", profile.name);
    println!("Prompt style: {}", profile.screen_prompt);
    println!();
    println!("{} Switch eras with:", marker);
    println!("era 1984 | era 1995 | era 2007 | era 2040");
    println!();
    println!("The launcher does not switch eras by itself.");
}

fn run_travel(command: &str) {
    let rest = command.strip_prefix("travel").unwrap_or("").trim();
    if rest.is_empty() || rest.contains(' ') {
        print_travel_usage();
        return;
    }

    let year = match rest.parse::<u32>() {
        Ok(year) => year,
        Err(_) => {
            print_travel_usage();
            return;
        }
    };

    let (era, equivalent, explanation) = match map_travel_year(year) {
        Some(mapping) => mapping,
        None => {
            println!("Unsupported travel year: {}", year);
            println!("ChronoOS currently maps years from 1980 onward.");
            print_travel_usage();
            return;
        }
    };

    println!("Travel request: {}", year);
    println!("Mapped era: {} mode", era);
    println!("{}", explanation);
    println!("Equivalent: {}", equivalent);
    println!();

    execute_command(equivalent);
}

fn print_travel_usage() {
    println!("Usage: travel <year>");
}

fn map_travel_year(year: u32) -> Option<(&'static str, &'static str, &'static str)> {
    if year >= 1980 && year <= 1989 {
        Some((
            "1984",
            "era 1984",
            "Why: 1980s computing is represented by the monochrome early personal computer era.",
        ))
    } else if year >= 1990 && year <= 1999 {
        Some((
            "1995",
            "era 1995",
            "Why: 1990s computing is represented by the classic desktop GUI era.",
        ))
    } else if year >= 2000 && year <= 2009 {
        Some((
            "2007",
            "era 2007",
            "Why: 2000s computing is represented by the glossy mobile-web transition era.",
        ))
    } else if year >= 2010 {
        Some((
            "2040",
            "era 2040",
            "Why: future years use the speculative future-lab era.",
        ))
    } else {
        None
    }
}

fn print_deep_museum_page(
    title: &str,
    meaning: &str,
    current: &str,
    real_os: &str,
    missing: &str,
) {
    let profile = theme::active_profile();

    println!("Museum: {}", title);
    println!("Era lens: {}", profile.name);
    println!();
    println!("What it means");
    println!("{}", meaning);
    println!();
    println!("What ChronoOS currently does");
    println!("{}", current);
    println!();
    println!("What real operating systems do");
    println!("{}", real_os);
    println!();
    println!("What is still missing");
    println!("{}", missing);
}

fn print_museum_disk_page() {
    print_deep_museum_page(
        "disk",
        "A disk is persistent storage split into addressable blocks or sectors. The kernel asks it to read or write bytes that should survive reboot.",
        "ChronoOS uses disk-backed paths for ChronoFS when available, and falls back conservatively when persistent disk state is not available.",
        "Real operating systems have storage drivers, request queues, caches, partitions, error handling, and careful ordering so data survives crashes.",
        "Robust driver coverage, broad device support, deep caching, full crash-safety, and runtime-verified recovery are still missing.",
    );
}

fn print_museum_filesystem_page() {
    print_deep_museum_page(
        "filesystem",
        "A filesystem gives names and metadata to bytes on disk so people can work with files instead of raw sectors.",
        "ChronoOS has ChronoFS with simple named files, contiguous extents, ls/cat/write/rm, fsck/repair, and a tiny journal for metadata safety.",
        "Real operating systems add directories, permissions, caching, journaling or copy-on-write, quotas, timestamps, and many compatibility rules.",
        "ChronoFS is still intentionally small: no full POSIX model, no directories, limited recovery, and no broad real-world filesystem compatibility.",
    );
}

fn print_museum_userspace_page() {
    print_deep_museum_page(
        "userspace",
        "Userspace is where regular programs run with less privilege than the kernel, so mistakes are easier to contain.",
        "ChronoOS has educational paths like ring3, syshello, and exec <name> that show the idea without pretending to be a full process platform.",
        "Real operating systems isolate processes with virtual memory, permissions, handles, signals, scheduling, and carefully validated kernel entry points.",
        "A mature process model, strong isolation, broad runtime verification, and production-grade program lifecycle management are still missing.",
    );
}

fn print_museum_syscalls_page() {
    print_deep_museum_page(
        "syscalls",
        "A syscall is a controlled doorway from userspace into the kernel for services like output, files, memory, and time.",
        "ChronoOS includes simple syscall-style teaching demos through userspace examples such as syshello.",
        "Real operating systems define stable syscall ABIs, validate every pointer and argument, enforce permissions, and keep compatibility for years.",
        "A broad syscall table, hardened validation, permissions, tracing, and complete ABI documentation are still missing.",
    );
}

fn print_museum_elf_page() {
    print_deep_museum_page(
        "ELF",
        "ELF is a common executable file format that describes program code, data, entry points, and loading rules.",
        "ChronoOS exposes exec <name> as an implemented-in-code educational path for loading stored programs where supported.",
        "Real operating systems parse many ELF sections and segments, map pages, handle relocations, link libraries, and prepare process state.",
        "Broad ELF feature support, dynamic linking, relocation coverage, safety hardening, and runtime verification are still missing.",
    );
}

fn print_museum_networking_page() {
    print_deep_museum_page(
        "networking",
        "Networking moves packets between machines through devices, protocols, addresses, and routing rules.",
        "ChronoOS treats networking conservatively in the shell reports; no full verified network stack is claimed here.",
        "Real operating systems drive network cards, handle interrupts, implement protocols like Ethernet/IP/TCP/UDP, and expose sockets to programs.",
        "A verified driver path, protocol stack, socket API, packet tools, and runtime network tests are still missing.",
    );
}

fn print_museum_smp_page() {
    print_deep_museum_page(
        "SMP",
        "SMP means symmetric multiprocessing: more than one CPU core can run kernel or program work at the same time.",
        "ChronoOS has implemented-in-code core-count and multicore teaching concepts, surfaced through commands like cores.",
        "Real operating systems bring up application processors, send inter-processor interrupts, balance work, and protect shared data with locks.",
        "Deep multicore verification, mature synchronization, cross-core scheduling confidence, and stress testing are still missing.",
    );
}

fn print_museum_scheduler_page() {
    print_deep_museum_page(
        "scheduler",
        "A scheduler chooses which task gets CPU time next, making one processor feel like it can do many things.",
        "ChronoOS has task and scheduler teaching paths that are useful previews, not a production scheduler audit.",
        "Real operating systems handle priorities, blocking, wakeups, fairness, preemption, CPU affinity, and many edge cases under load.",
        "Full production behavior, fairness tuning, blocking I/O integration, multicore stress confidence, and runtime verification are still missing.",
    );
}

fn run_poster(command: &str) {
    let topic = command.strip_prefix("poster").unwrap_or("").trim();

    match topic {
        "" => poster_overview(),
        "boot" => poster_boot(),
        "system" => poster_system(),
        "roadmap" => poster_roadmap(),
        "eras" => poster_eras(),
        _ => print_poster_usage(),
    }
}

fn print_poster_usage() {
    println!("Usage: poster [boot|system|roadmap|eras]");
}

fn print_poster_header(title: &str) {
    let profile = theme::active_profile();

    println!("============================================================");
    println!("TIME CAPSULE OS :: {}", title);
    println!("Era: {}", profile.name);
    println!("Prompt: {}", profile.screen_prompt);
    println!("============================================================");
    println!();
}

fn print_poster_row(label: &str, value: &str) {
    println!("{:18} {}", label, value);
}

fn poster_overview() {
    print_poster_header("POSTER MODE");
    print_poster_row("Identity", "educational x86_64 hobby OS");
    print_poster_row("Mood", "retro, readable, build-in-public");
    print_poster_row("Display", "framebuffer text UI");
    print_poster_row("Safety", "presentation only; no checks or mutations");
    println!();
    println!("Screens");
    print_poster_row("poster boot", "boot and kernel learning flow");
    print_poster_row("poster system", "compact subsystem status card");
    print_poster_row("poster roadmap", "implemented, partial, roadmap");
    print_poster_row("poster eras", "1984, 1995, 2007, 2040 showcase");
}

fn poster_boot() {
    print_poster_header("BOOT STORY");
    print_poster_row("Stage 1", "bootloader places the kernel in memory");
    print_poster_row("Stage 2", "kernel sets up CPU, memory, interrupts");
    print_poster_row("Stage 3", "drivers bring text input and output online");
    print_poster_row("Stage 4", "shell becomes the learning cockpit");
    println!();
    println!("Museum trail");
    print_poster_row("Start", "museum boot");
    print_poster_row("Kernel", "museum kernel");
    print_poster_row("Memory", "museum memory");
    print_poster_row("Deeper", "museum disk | filesystem | scheduler");
    println!();
    print_poster_row("Verification", "runtime checks still needed");
}

fn poster_system() {
    print_poster_header("SYSTEM STATUS CARD");
    print_poster_row("serial", "implemented in code");
    print_poster_row("framebuffer", "implemented in code");
    print_poster_row("timer", "implemented in code");
    print_poster_row("keyboard", "implemented in code");
    print_poster_row("mouse", "implemented in code");
    print_poster_row("filesystem", "implemented in code");
    print_poster_row("heap", "implemented in code");
    print_poster_row("network", "partially implemented");
    print_poster_row("SMP/core count", "partially implemented");
    print_poster_row("scheduler", "partially implemented");
    print_poster_row("userspace/ELF", "partially implemented");
    println!();
    print_poster_row("Note", "poster does not run doctor or probes");
}

fn poster_roadmap() {
    print_poster_header("BUILD-IN-PUBLIC ROADMAP");
    println!("implemented in code");
    print_poster_row("Shell", "help, eras, apps, guides, museum");
    print_poster_row("ChronoFS", "files, fsck, repair, tiny journal");
    print_poster_row("Learning", "demo, tour, capsule, doctor, poster");
    println!();
    println!("partially implemented");
    print_poster_row("Windows/tasks", "useful previews, not full desktop");
    print_poster_row("Userspace", "teaching path, not mature platform");
    print_poster_row("Recovery", "conservative, refuses guesses");
    println!();
    println!("roadmap/design-only");
    print_poster_row("Verification", "build and shell confidence loop");
    print_poster_row("Lessons", "richer quests and museum trails");
    print_poster_row("Apps", "small workflows before big systems");
    println!();
    println!("needs runtime verification");
    print_poster_row("Next", "build sanity and OS-shell checks");
}

fn poster_eras() {
    print_poster_header("ERA SHOWCASE");
    print_poster_row("1984", "monochrome early personal computer mood");
    print_poster_row("1995", "classic desktop GUI era");
    print_poster_row("2007", "glossy mobile-web transition");
    print_poster_row("2040", "speculative future lab");
    println!();
    println!("Travel examples");
    print_poster_row("travel 1987", "maps to era 1984");
    print_poster_row("travel 1998", "maps to era 1995");
    print_poster_row("travel 2004", "maps to era 2007");
    print_poster_row("travel 2049", "maps to era 2040");
    println!();
    print_poster_row("Note", "poster eras does not switch eras");
}

fn print_about() {
    let profile = theme::active_profile();

    println!("ChronoOS | Era: {} | v0.1", profile.name);
}

fn print_uptime() {
    println!("Uptime: {} seconds", timer::uptime_seconds());
}

fn print_clock() {
    println!("Ticks: {}", timer::ticks());
}

fn print_memory() {
    let stats = memory::stats();

    println!("Total memory: {} MB", stats.total_memory_bytes / 1024 / 1024);
    println!(
        "Heap: {} MB at {:#x}",
        stats.heap_size_bytes / 1024 / 1024,
        stats.heap_start
    );
    println!("Used: {} KB", stats.heap_used_bytes / 1024);
    println!("Free: {} KB", stats.heap_free_bytes / 1024);
    println!(
        "Largest free block: {} KB",
        stats.heap_largest_free_block_bytes / 1024
    );
}

fn print_cores() {
    let counts = crate::smp::tasks_per_core();
    let core_count = crate::smp::core_count();

    println!("Cores: {}", core_count);
    for core_id in 0..core_count {
        println!("core {}: {} task(s)", core_id, counts[core_id]);
    }
}

fn beep(command: &str) {
    let mut parts = command.split_ascii_whitespace();
    let _command_name = parts.next();

    let Some(frequency) = parts.next() else {
        println!("Usage: beep <hz>");
        return;
    };

    if parts.next().is_some() {
        println!("Usage: beep <hz>");
        return;
    }

    let frequency_hz: u32 = match frequency.parse() {
        Ok(frequency_hz) => frequency_hz,
        Err(_) => {
            println!("Usage: beep <hz>");
            return;
        }
    };

    match crate::sound::beep(frequency_hz, 500) {
        Ok(()) => {}
        Err(crate::sound::BeepError::FrequencyOutOfRange) => {
            println!("beep: frequency must be 20..20000 Hz");
        }
    }
}

fn list_files() {
    if !fs::list(|name| println!("{}", name)) {
        println!("(no files)");
    }
}

fn run_files_command(command: &str) {
    let mode = command.strip_prefix("files").unwrap_or("").trim();

    match mode {
        "" | "help" => print_files_workspace(),
        "list" => list_files_with_sizes(),
        "sample" => print_files_sample(),
        "demo" => print_files_demo(),
        mode if mode.starts_with("info ") => files_info(mode.strip_prefix("info").unwrap().trim()),
        mode if mode.starts_with("search ") => {
            files_search(mode.strip_prefix("search").unwrap().trim())
        }
        mode if mode.starts_with("copy ") => files_copy(mode.strip_prefix("copy").unwrap().trim()),
        mode if mode.starts_with("rename ") => {
            files_rename(mode.strip_prefix("rename").unwrap().trim())
        }
        _ => {
            println!("Usage: files [list|info <name>|search <term>|sample|demo]");
            println!("       files copy <src> <dst>");
            println!("       files rename <old> <new>");
        }
    }
}

fn list_files_with_sizes() {
    println!("ChronoFS files");

    if !fs::list(|name| match fs::file_info(name) {
        Ok(info) => println!("- {} ({} bytes)", info.name, info.size_bytes),
        Err(_) => println!("- {} (metadata unavailable)", name),
    }) {
        println!("(no files)");
    }

    println!("Hidden journal metadata is not listed as a user file.");
}

fn files_info(name: &str) {
    if name.is_empty() {
        println!("Usage: files info <name>");
        return;
    }

    match fs::file_info(name) {
        Ok(info) => {
            let status = fs::status();
            println!("ChronoFS file info");
            println!("Name: {}", info.name);
            println!("Size: {} bytes", info.size_bytes);
            println!("Storage: {}", info.storage.label());
            println!(
                "Disk: {}",
                if status.disk_available {
                    "available"
                } else {
                    "unavailable"
                }
            );
            println!(
                "Persistence note: {}",
                if status.persistent {
                    "writes use the ATA-backed ChronoFS path"
                } else {
                    "heap fallback is volatile and disappears on reboot"
                }
            );
            println!("Verification note: simple CRUD was QEMU-tested; repair/recovery was not.");
        }
        Err(error) => print_fs_operation_error("inspect", name, error),
    }
}

fn files_search(term: &str) {
    if term.is_empty() {
        println!("Usage: files search <term>");
        return;
    }

    println!("ChronoFS search: {}", term);
    let mut matches = 0usize;

    fs::list(|name| {
        let name_match = contains_ignore_ascii_case(name, term);
        let content_match = match fs::read(name) {
            Ok(content) => contains_ignore_ascii_case(content, term),
            Err(_) => false,
        };

        if name_match || content_match {
            print!("- {}: ", name);
            if name_match && content_match {
                println!("name and UTF-8 content match");
            } else if name_match {
                println!("name match");
            } else {
                println!("UTF-8 content match");
            }
            matches += 1;
        }
    });

    if matches == 0 {
        println!("No visible file matches.");
    } else {
        println!("Matched {} file(s). Contents were not dumped.", matches);
    }
}

fn print_files_sample() {
    println!("ChronoFS sample files");
    println!("This command is read-only. It suggests safe demo content.");
    println!();
    println!("Try:");
    println!("- write hello.txt hello from ChronoOS");
    println!("- write demo.txt ChronoOS file demo");
    println!("- files list");
    println!("- files info demo.txt");
    println!("- files search ChronoOS");
    println!();
    println!("Use a disposable image when verifying writes, deletes, copy, or fsck.");
}

fn print_files_demo() {
    println!("ChronoFS guided demo");
    println!("This walkthrough does not mutate files automatically.");
    println!();
    println!("1. files list");
    println!("2. write demo.txt ChronoOS file demo");
    println!("3. cat demo.txt");
    println!("4. files info demo.txt");
    println!("5. files search ChronoOS");
    println!("6. fs check");
    println!("7. journal");
    println!();
    println!("Optional copy check on a disposable image:");
    println!("8. files copy demo.txt demo-copy.txt");
    println!("9. cat demo-copy.txt");
}

fn files_copy(args: &str) {
    let mut parts = args.split_ascii_whitespace();
    let Some(src) = parts.next() else {
        println!("Usage: files copy <src> <dst>");
        return;
    };
    let Some(dst) = parts.next() else {
        println!("Usage: files copy <src> <dst>");
        return;
    };
    if parts.next().is_some() {
        println!("Usage: files copy <src> <dst>");
        return;
    }

    if src == dst {
        println!("files copy: source and destination must be different");
        return;
    }

    match fs::file_exists(dst) {
        Ok(true) => {
            println!("files copy: destination already exists: {}", dst);
            println!("No overwrite was performed.");
            return;
        }
        Ok(false) => {}
        Err(error) => {
            print_fs_operation_error("copy destination check", dst, error);
            return;
        }
    }

    let content = match fs::read_bytes(src) {
        Ok(bytes) => Vec::from(bytes),
        Err(error) => {
            print_fs_operation_error("copy source read", src, error);
            return;
        }
    };

    match fs::write_bytes(dst, &content) {
        Ok(()) => println!(
            "Copied {} bytes from {} to {} without overwriting.",
            content.len(),
            src,
            dst
        ),
        Err(error) => print_fs_operation_error("copy write", dst, error),
    }
}

fn files_rename(args: &str) {
    let mut parts = args.split_ascii_whitespace();
    let Some(old_name) = parts.next() else {
        println!("Usage: files rename <old> <new>");
        return;
    };
    let Some(new_name) = parts.next() else {
        println!("Usage: files rename <old> <new>");
        return;
    };
    if parts.next().is_some() {
        println!("Usage: files rename <old> <new>");
        return;
    }

    println!("files rename is intentionally not implemented yet.");
    println!("Requested: {} -> {}", old_name, new_name);
    println!("Reason: safe rename needs stronger persistence and failure-mode verification.");
    println!("Conservative path on a disposable image:");
    println!("1. files copy {} {}", old_name, new_name);
    println!("2. cat {}", new_name);
    println!("3. files info {}", new_name);
    println!("4. rm {} only after manual verification", old_name);
}

fn cat_file(command: &str) {
    let name = command.strip_prefix("cat").unwrap_or("").trim();

    if name.is_empty() {
        println!("Usage: cat <name>");
        return;
    }

    match fs::read(name) {
        Ok(content) => println!("{}", content),
        Err(error) => print_fs_operation_error("read", name, error),
    }
}

fn write_file(command: &str) {
    let rest = command.strip_prefix("write").unwrap_or("").trim_start();
    let Some((name, content)) = split_once_ascii_whitespace(rest) else {
        println!("Usage: write <name> <content>");
        return;
    };

    if content.is_empty() {
        println!("Usage: write <name> <content>");
        return;
    }

    match fs::write(name, content) {
        Ok(()) => {}
        Err(error) => print_fs_operation_error("write", name, error),
    }
}

fn remove_file(command: &str) {
    let name = command.strip_prefix("rm").unwrap_or("").trim();

    if name.is_empty() {
        println!("Usage: rm <name>");
        return;
    }

    match fs::remove(name) {
        Ok(()) => {}
        Err(error) => print_fs_operation_error("remove", name, error),
    }
}

fn run_fs_command(command: &str) {
    let mode = command.strip_prefix("fs").unwrap_or("").trim();

    match mode {
        "" | "status" => print_fs_status(),
        "info" => print_fs_info(),
        "check" => {
            let report = fs::check(false);
            print_fsck_report(&report, false);
        }
        "journal" => print_journal_status(fs::journal_status()),
        "help" => print_fs_namespace_help(),
        "repair" | "check repair" => {
            println!("fs repair is read-only by design.");
            println!("Use `fsck repair` intentionally with a controlled disk image.");
        }
        _ => {
            println!("Usage: fs [status|info|check|journal|help]");
            println!("Repair is explicit: fsck repair");
        }
    }
}

fn print_fs_status() {
    let status = fs::status();

    println!("ChronoFS status");
    println!(
        "Mode: {}",
        if status.persistent {
            "persistent ATA disk"
        } else {
            "heap fallback"
        }
    );
    println!(
        "Disk: {}",
        if status.disk_available {
            "available"
        } else {
            "unavailable"
        }
    );
    println!(
        "Files: visible={} cache={} disk={}",
        status.visible_file_count, status.cache_file_count, status.disk_file_count
    );
    println!(
        "Slots: used={} free={} max={}",
        status.used_file_slots, status.free_file_slots, status.max_files
    );
    println!(
        "Journal: {} / {}",
        if status.journal_present {
            "reserved"
        } else {
            "not reserved"
        },
        status.journal.state
    );
    println!("Next: fs info, fs check, fs journal");
}

fn print_fs_info() {
    let status = fs::status();

    println!("ChronoFS info");
    println!("Format: CHRONFS1 v1, fixed educational layout");
    println!("Total sectors: {} (512-byte sectors)", status.total_sectors);
    println!("Data starts at sector: {}", status.data_start_sector);
    println!("File slots: {}", status.max_files);
    println!("Max file bytes: {}", status.max_file_bytes);
    println!(
        "Journal slot: {}",
        if status.journal_present {
            "hidden __chronofs_journal file"
        } else {
            "not available"
        }
    );
    println!("No directories, permissions, timestamps, or POSIX compatibility.");
}

fn print_fs_namespace_help() {
    println!("ChronoFS inspection commands");
    println!("- fs / fs status   : mode, disk, file, slot, and journal summary");
    println!("- fs info          : layout and limits");
    println!("- fs check         : read-only fsck summary");
    println!("- fs journal       : one-record journal status");
    println!("- ls/cat/write/rm  : normal shell file commands");
    println!("- fsck repair      : explicit mutating repair path");
    println!("The fs namespace is read-only; it does not repair or rewrite metadata.");
}

fn exec_file(command: &str) {
    let name = command.strip_prefix("exec").unwrap_or("").trim();

    if name.is_empty() {
        println!("Usage: exec <name>");
        return;
    }

    match fs::read_bytes(name) {
        Ok(bytes) => match crate::process::exec_elf(name, bytes) {
            Ok(code) => println!("[process exited: {}]", code),
            Err(error) => print_exec_error(name, error),
        },
        Err(error) => print_fs_operation_error("exec read", name, error),
    }
}

fn run_fsck(command: &str) {
    let mode = command.strip_prefix("fsck").unwrap_or("").trim();
    let repair = match mode {
        "" => false,
        "repair" => true,
        _ => {
            println!("Usage: fsck [repair]");
            return;
        }
    };

    if repair {
        print_reliability_warning("ChronoFS repair");
        println!("Warning: fsck repair mutates ChronoFS metadata.");
        println!("Use a controlled disk image and record before/after evidence.");
        println!("It refuses untrusted geometry, duplicate ownership, and unsafe errors.");
    }

    let report = fs::check(repair);
    print_fsck_report(&report, repair);
}

fn print_fsck_report(report: &fs::FsckReport, repair: bool) {
    println!("ChronoFS check: {}", report.status_label());
    println!(
        "Clean: {}",
        if !report.disk_available {
            "not checkable"
        } else if report.warnings == 0 && report.errors == 0 {
            "yes"
        } else {
            "no"
        }
    );
    println!("Checked: superblock, file table, extents, bitmap, duplicate sectors");
    println!("Entries: checked={} live={}", report.checked_entries, report.live_entries);
    println!(
        "Suspicious: warnings={} errors={} invalid={} bitmap={} duplicates={}",
        report.warnings,
        report.errors,
        report.invalid_entries,
        report.bitmap_mismatches,
        report.duplicate_sectors
    );
    println!("Repaired: {} item(s)", report.repaired_items);

    if !report.disk_available {
        println!("Not repaired: persistent disk unavailable; heap fallback is not fsck-able.");
    } else if !repair {
        println!("Not repaired: read-only check. Use fsck repair only intentionally.");
    } else if report.errors > 0 {
        println!("Not repaired: unsafe errors require manual investigation.");
    } else if report.repaired_items == 0 {
        println!("Not repaired: no safe repair was needed.");
    }

    if report.findings.is_empty() {
        println!("Clean: no problems found by current checks.");
        return;
    }

    println!("Findings:");
    for finding in &report.findings {
        println!("- {}", finding);
    }
}

fn run_journal(command: &str) {
    let mode = command.strip_prefix("journal").unwrap_or("").trim();
    if !mode.is_empty() {
        println!("Usage: journal");
        return;
    }

    print_journal_status(fs::journal_status());
}

fn print_journal_status(status: fs::JournalStatus) {
    println!("ChronoFS journal: {}", status.state);
    println!("Available: {}", if status.available { "yes" } else { "no" });
    println!("Clean: {}", if status.clean { "yes" } else { "no" });
    println!("Operation: {}", status.operation);
    if !status.target.is_empty() {
        println!("Target: {}", status.target);
    }
    println!("{}", status.message);
    println!("Note: clean means no pending journal record, not full runtime proof.");
}

fn open_window(command: &str) {
    let name = command.strip_prefix("open").unwrap_or("").trim();

    match name {
        "notes" => open_windowed_app("notes", apps::notes_task_entry, wm::open_notes),
        "sysinfo" => open_windowed_app("sysinfo", apps::sysinfo_task_entry, wm::open_sysinfo),
        "paint" => {
            println!("open paint: paint is roadmap/design-only.");
            println!("Try: apps info paint");
        }
        _ => {
            println!("Usage: open notes|sysinfo");
            println!("Window mode is limited; shell apps remain available.");
        }
    }
}

fn open_windowed_app(name: &str, entry: fn() -> !, open: fn(u8) -> bool) {
    let Some(task_id) = sched::spawn(name, entry) else {
        println!("open {}: no free task slot", name);
        println!("Try: tasks");
        return;
    };

    if !open(task_id) {
        sched::kill(task_id);
        println!("open {}: too many windows open", name);
        println!("Try: windows list or windows close <id>");
    }
}

fn run_windows_command(command: &str) {
    let mode = command.strip_prefix("windows").unwrap_or("").trim();

    match mode {
        "" | "list" => list_windows(),
        "status" => print_windows_status(),
        "help" => print_windows_usage(),
        mode if mode.starts_with("focus ") => focus_window(mode),
        mode if mode.starts_with("close ") => close_window(mode),
        _ => print_windows_usage(),
    }
}

fn list_windows() {
    if wm::window_count() == 0 {
        println!("No windows open.");
        println!("Try: open notes or open sysinfo");
        return;
    }

    println!("ID  TITLE    TASK POS       SIZE      FOCUS");
    wm::for_each_window(|id, title, task_id, x, y, width, height, focused| {
        let marker = if focused { "*" } else { "" };
        println!(
            "{:<3} {:<8} {:<4} {:>3},{:<3} {:>3}x{:<3} {}",
            id, title, task_id, x, y, width, height, marker
        );
    });
}

fn print_windows_status() {
    println!("ChronoOS windows");
    println!("Open: {}/{}", wm::window_count(), wm::window_capacity());
    println!("Dragging: {}", if wm::is_dragging() { "yes" } else { "no" });
    println!("Supported window apps: notes, sysinfo");
    println!("Shell fallbacks: notes, sysinfo, apps launch notes, apps launch sysinfo");
    println!("Status: partially implemented, needs runtime verification.");
    println!("Boundary: fixed-capacity teaching windows, not a compositor.");
}

fn focus_window(command: &str) {
    let id = match parse_window_id(command.strip_prefix("focus").unwrap_or("").trim()) {
        Some(id) => id,
        None => {
            println!("Usage: windows focus <id>");
            return;
        }
    };

    if wm::focus_by_id(id) {
        println!("Focused window {}", id);
    } else {
        println!("windows: no window {}", id);
    }
}

fn close_window(command: &str) {
    let id = match parse_window_id(command.strip_prefix("close").unwrap_or("").trim()) {
        Some(id) => id,
        None => {
            println!("Usage: windows close <id>");
            return;
        }
    };

    if wm::close_by_id(id) {
        println!("Closed window {}", id);
    } else {
        println!("windows: no window {}", id);
    }
}

fn parse_window_id(text: &str) -> Option<u32> {
    if text.is_empty() || text.contains(' ') {
        return None;
    }

    text.parse().ok()
}

fn print_windows_usage() {
    println!("Usage: windows [list|status|focus <id>|close <id>|help]");
    println!("Open windows with: open notes | open sysinfo");
}

fn list_tasks() {
    println!("ID  NAME     STATE");
    sched::for_each_task(|id, name, state| {
        let state_str = match state {
            sched::TaskState::Running => "running",
            sched::TaskState::Ready => "ready",
            sched::TaskState::Blocked => "blocked",
            _ => "?",
        };
        println!("{:<3} {:<8} {}", id, name, state_str);
    });
}

fn kill_task(command: &str) {
    let rest = command.strip_prefix("kill").unwrap_or("").trim();

    let id: u8 = match rest.parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Usage: kill <id>");
            return;
        }
    };

    // Collect the name before killing so we can print it in the confirmation.
    let mut name_buf = [0u8; 16];
    let mut name_len = 0usize;
    let mut found = false;
    sched::for_each_task(|tid, name, _state| {
        if tid == id {
            found = true;
            let bytes = name.as_bytes();
            let len = bytes.len().min(16);
            name_buf[..len].copy_from_slice(&bytes[..len]);
            name_len = len;
        }
    });

    if !found {
        println!("kill: no such task {}", id);
        return;
    }

    // SAFETY: name_buf is always written from valid UTF-8 bytes above.
    let name_str = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };

    if sched::kill(id) {
        wm::close_for_task(id); // close the associated window, if any
        println!("Task {} ({}) terminated", id, name_str);
    } else {
        println!("kill: cannot terminate task {} ({})", id, name_str);
    }
}

fn split_once_ascii_whitespace(input: &str) -> Option<(&str, &str)> {
    let split_at = input
        .bytes()
        .position(|byte| byte.is_ascii_whitespace())?;
    let (head, tail) = input.split_at(split_at);

    Some((head, tail.trim_start()))
}

fn print_fs_operation_error(operation: &str, name: &str, error: FsError) {
    match error {
        FsError::NotFound => println!("{} failed: file not found: {}", operation, name),
        FsError::EmptyName => println!("{} failed: missing filename", operation),
        FsError::InvalidName => {
            println!("{} failed: invalid filename: {}", operation, name);
            println!("Names cannot contain whitespace or use the hidden journal name.");
        }
        FsError::NameTooLong => {
            println!("{} failed: filename too long: {}", operation, name);
            println!("Maximum filename length is {} bytes.", fs::MAX_FILENAME_LEN);
        }
        FsError::FileTooLarge => println!("{} failed: file too large: {}", operation, name),
        FsError::NoSpace => {
            println!("{} failed: not enough ChronoFS space for {}", operation, name);
            println!("Try files list, fs status, or use a fresh disposable data image.");
        }
        FsError::Disk => {
            println!("{} failed: disk or UTF-8 error for {}", operation, name);
            println!("Try fs status, fs check, and journal before retrying.");
        }
    }
}

fn print_exec_error(name: &str, error: crate::process::ExecError) {
    match error {
        crate::process::ExecError::AlreadyRunning => println!("exec: process already running"),
        crate::process::ExecError::BadElf(crate::elf::ElfError::BadMagic) => {
            println!("exec: not an ELF file: {}", name)
        }
        crate::process::ExecError::BadElf(crate::elf::ElfError::Unsupported) => {
            println!("exec: unsupported ELF: {}", name)
        }
        crate::process::ExecError::BadElf(_) => println!("exec: malformed ELF: {}", name),
        crate::process::ExecError::OutOfMemory => println!("exec: out of memory"),
        crate::process::ExecError::TooManyRanges => println!("exec: too many ELF segments"),
    }
}

fn run_era_command(command: &str) {
    let mut parts = command.split_ascii_whitespace();
    let _command_name = parts.next();

    let Some(year) = parts.next() else {
        print_era_usage();
        return;
    };

    if parts.next().is_some() {
        print_era_usage();
        return;
    }

    match Era::from_year(year) {
        Some(era) => switch_era(era),
        None => print_era_usage(),
    }
}

fn switch_era(era: Era) {
    let profile = era.profile();

    theme::set_active_era(era);
    console::set_theme(profile);
    println!("Switched to {} mode.", profile.name);
    serial_println!("[CHRONO] era: {}", profile.name);
    wm::redraw_if_open();
}

fn print_era_usage() {
    println!("Usage: era 1984|1995|2007|2040");
}

fn reboot() -> ! {
    print_reliability_warning("reboot");
    serial_println!("[CHRONO] reboot requested");

    // SAFETY: Port 0x64 is the PS/2 controller command port on the
    // PC-compatible machine QEMU emulates. Command 0xFE requests a CPU reset.
    unsafe {
        outb(RESET_COMMAND_PORT, CPU_RESET_COMMAND);
    }

    halt_forever()
}

fn draw_cursor() {
    let profile = theme::active_profile();

    print!("{}", profile.cursor_glyph);
}

fn erase_cursor() {
    for _ in 0..theme::active_profile().cursor_glyph.len() {
        console::backspace();
    }
}

fn show_cursor(cursor_visible: &mut bool) {
    if !*cursor_visible {
        draw_cursor();
        *cursor_visible = true;
    }
}

fn hide_cursor(cursor_visible: &mut bool) {
    if *cursor_visible {
        erase_cursor();
        *cursor_visible = false;
    }
}

fn toggle_cursor(cursor_visible: &mut bool) {
    if *cursor_visible {
        erase_cursor();
        *cursor_visible = false;
    } else {
        draw_cursor();
        *cursor_visible = true;
    }
}

fn cpu_relax() {
    // SAFETY: `pause` is a CPU hint used inside the polling shell loop. It does
    // not access memory or devices; it just makes the busy-wait friendlier.
    unsafe {
        core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
    }
}

fn halt_forever() -> ! {
    loop {
        // SAFETY: `hlt` stops the CPU until the next external interrupt. This
        // fallback is only reached if the reboot command does not reset.
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

unsafe fn outb(port: u16, value: u8) {
    // SAFETY: The caller must ensure `port` belongs to the intended hardware
    // device and `value` is a valid command for that port.
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}
