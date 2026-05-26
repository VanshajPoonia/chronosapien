//! Tiny line-based shell for the first interactive milestone.

use crate::apps;
use crate::console;
use crate::fs::{self, FsError};
use crate::keyboard::{self, KeyEvent};
use crate::memory;
use crate::mouse;
use crate::museum;
use crate::net;
use crate::quest;
use crate::sched;
use crate::theme::{self, Era};
use crate::timer;
use crate::wm;
use crate::{print, println, serial_println};

const COMMAND_BUFFER_CAPACITY: usize = 80;
const RESET_COMMAND_PORT: u16 = 0x64;
const CPU_RESET_COMMAND: u8 = 0xFE;

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

                execute_command(buffer.as_str());
                buffer.clear();
                print_prompt();
                show_cursor(&mut cursor_visible);
                wm::redraw_if_open();
                idle_ticks = 0;
            }
            Some(_) => {
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

fn execute_command(command: &str) {
    let command = command.trim();

    if !command.is_empty() {
        serial_println!("[CHRONO] cmd: {}", command);
    }

    match command {
        "" => {}
        "help" => print_help(),
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
        "ring3" => crate::ring3::run_demo(),
        "syshello" => crate::ring3::run_syshello(),
        "ls" => list_files(),
        command if command == "cat" || command.starts_with("cat ") => cat_file(command),
        command if command == "write" || command.starts_with("write ") => write_file(command),
        command if command == "exec" || command.starts_with("exec ") => exec_file(command),
        command if command == "rm" || command.starts_with("rm ") => remove_file(command),
        command if command == "fsck" || command.starts_with("fsck ") => run_fsck(command),
        command if command == "journal" || command.starts_with("journal ") => run_journal(command),
        command if command == "era" || command.starts_with("era ") => run_era_command(command),
        command if command == "open" || command.starts_with("open ") => open_window(command),
        "tasks" => list_tasks(),
        command if command == "kill" || command.starts_with("kill ") => kill_task(command),
        command if net::run(command) => {}
        command if museum::run(command) => {}
        command if quest::run(command) => {}
        command if apps::run(command) => {}
        _ => println!("unknown command: {}", command),
    }
}

fn print_help() {
    println!(
        "Commands: help, demo, tour, capsule, doctor, poster, travel <year>, clear, about, reboot, era, uptime, clock, mem, cores, beep <hz>, ring3, syshello"
    );
    println!("Files: ls, cat <name>, write <name> <content>, rm <name>, exec <name>, fsck [repair], journal");
    println!("Apps: apps [notes|calc|sysinfo|files|clock|museum|theme|tasks]");
    println!("Notes: notes read, notes write <text>, notes clear, notes save, notes open");
    println!("Windows: open notes, open sysinfo");
    println!("Tasks: tasks, kill <id>");
    println!("Network: net, net arp, net send [ip port text]");
    println!("Museum: museum boot|kernel|memory|interrupts|keyboard|serial|era");
    println!("Deep museum: museum disk|filesystem|userspace|syscalls|elf|networking|smp|scheduler");
    println!("Quests: quest list, quest status, stats, inventory");
}

fn run_demo(command: &str) {
    let rest = command.strip_prefix("demo").unwrap_or("").trim();
    if !rest.is_empty() {
        println!("Usage: demo");
        return;
    }

    let profile = theme::active_profile();

    println!("Time Capsule OS demo");
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

    println!("Time Capsule OS tour");
    println!("Active era: {}", profile.name);
    println!("Prompt style: {}", profile.screen_prompt);
    println!();
    println!("This tour explains what is already code-present inside the OS.");
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
    println!("Time Capsule OS begins with the bootloader placing the kernel in memory.");
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
    println!("Time Capsule OS has code-present pieces for tracking memory, using a heap,");
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
    println!("ChronoFS is the small educational filesystem for Time Capsule OS.");
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
    println!("Time Capsule OS has small shell apps and window previews that help show");
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
    println!("Time Capsule OS has code-present demos for entering ring 3, making a simple");
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
    println!("Time Capsule OS should keep growing in small, understandable steps.");
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

    println!("A build-in-public map of Time Capsule OS.");
    println!("This command is read-only: it does not change eras, files, apps, tasks, or userspace.");
    println!();
    println!("Legend:");
    println!("- code-present: implemented in the source tree");
    println!("- partial: present, but still limited or educational");
    println!("- planned: next ideas, not finished systems");
    println!("- runtime verification needed: still needs build or OS-shell checks");
    println!();
    println!("Open a capsule:");
    println!("- capsule milestones");
    println!("- capsule current");
    println!("- capsule next");
}

fn capsule_milestones() {
    print_capsule_header("Capsule milestones");

    println!("[code-present]");
    println!("- Shell command center with help, eras, museum pages, quests, and guides");
    println!("- Era themes for 1984, 1995, 2007, and 2040");
    println!("- Museum explanations for boot, kernel, memory, interrupts, keyboard, serial, and eras");
    println!("- ChronoFS basics: ls, cat, write, rm, exec, fsck, fsck repair, and journal status");
    println!("- Tiny journal and conservative fsck/repair paths for ChronoFS metadata");
    println!("- Guided learning commands: demo, tour, and capsule");
    println!("- Small app commands and previews: notes, calc, sysinfo, open notes, open sysinfo");
    println!("- User-space demos where available: ring3, syshello, and exec <name>");
    println!();

    println!("[partial]");
    println!("- Window and task features are useful previews, not a complete desktop environment");
    println!("- User-space execution exists as an educational path, not a full process platform");
    println!("- ChronoFS recovery is conservative and refuses ambiguous repairs");
    println!("- Apps are intentionally small and shell-first");
    println!("- Guides explain current behavior, but still need runtime walkthrough checks");
    println!();

    println!("[planned]");
    println!("- Stronger build and shell verification loops");
    println!("- Richer guided lessons that connect quests, museum pages, and commands");
    println!("- Deeper app workflows while staying beginner-friendly");
    println!("- Safer filesystem recovery examples without guessing at data");
    println!("- Clearer user-space examples and documentation");
    println!();

    println!("[runtime verification needed]");
    println!("- Build sanity for the current source tree");
    println!("- Shell checks for capsule, tour, demo, and invalid command forms");
    println!("- ChronoFS checks for ls, cat, write, rm, fsck, fsck repair, and journal");
    println!("- Journal recovery scenarios on real or emulated disk state");
    println!("- App/window/task/user-space demos in the OS runtime");
}

fn capsule_current() {
    print_capsule_header("Capsule current");

    println!("[code-present]");
    println!("- Time Capsule OS can present itself through help, demo, tour, museum, and capsule text");
    println!("- ChronoFS has basic file commands, fsck/repair, and journal status support");
    println!("- Era identity and educational shell commands are part of the current experience");
    println!();

    println!("[partial]");
    println!("- Some systems are intentionally small teaching versions");
    println!("- Recovery, user-space, windows, tasks, and apps still need more runtime confidence");
    println!();

    println!("[runtime verification needed]");
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
    println!("[planned, not implemented here]");
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

    println!("Time Capsule OS doctor");
    println!("Era lens: {}", profile.name);
    println!("Prompt style: {}", profile.screen_prompt);
    println!();
    println!("Legend:");
    println!("- ok: checked safely inside this command");
    println!("- code-present: source paths or shell commands exist, but no live probe was run");
    println!("- partial: useful teaching version with known limits");
    println!("- unknown: no safe read-only status is exposed here");
    println!("- needs runtime verification: build or OS-shell testing is still needed");
    println!();
    println!("Subsystem health:");
    print_doctor_line(
        "serial",
        "code-present",
        "serial logging paths exist; doctor does not perform a port loopback test",
    );
    print_doctor_line(
        "framebuffer",
        "code-present",
        "shell text uses the display path; doctor does not probe graphics hardware",
    );
    print_doctor_line(
        "timer",
        "code-present",
        "uptime and clock commands are available; interrupt timing is not certified here",
    );
    print_doctor_line(
        "keyboard",
        "code-present",
        "the shell has an input path; doctor does not run an interactive key test",
    );
    print_doctor_line(
        "mouse",
        "unknown",
        "no safe mouse health probe is exposed to this report",
    );
    print_doctor_line(
        "filesystem",
        "code-present",
        "ChronoFS, fsck, repair, and journal commands exist; doctor does not run fsck",
    );
    print_doctor_line(
        "heap",
        "code-present",
        "memory reporting exists through mem; doctor does not allocate test blocks",
    );
    print_doctor_line(
        "network",
        "unknown",
        "no safe network status API is exposed to this report",
    );
    print_doctor_line(
        "SMP/core count",
        "code-present",
        "the cores command can report detected cores; doctor does not read or validate it",
    );
    print_doctor_line(
        "scheduler",
        "partial",
        "task commands are present as teaching tools; this is not a full scheduler audit",
    );
    print_doctor_line(
        "userspace/ELF support",
        "code-present",
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

    println!("Time Capsule Notes");
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
        "" => print_apps_launcher(),
        "notes" => launch_existing_app("notes"),
        "calc" => launch_existing_app("calc"),
        "sysinfo" => launch_existing_app("sysinfo"),
        "clock" => launch_existing_app("clock"),
        "tasks" => launch_existing_app("tasks"),
        "files" => print_files_app_card(),
        "museum" => print_museum_app_card(),
        "theme" => print_theme_app_card(),
        _ => print_apps_usage(),
    }
}

fn print_apps_usage() {
    println!("Usage: apps [notes|calc|sysinfo|files|clock|museum|theme|tasks]");
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
        ("App Launcher", "[-]", "Time Capsule shelf")
    }
}

fn print_apps_launcher() {
    let profile = theme::active_profile();
    let (title, marker, subtitle) = apps_style_for_era(profile.name);

    println!("{} - {}", title, subtitle);
    println!("Era lens: {}", profile.name);
    println!("Prompt style: {}", profile.screen_prompt);
    println!();
    println!("Launch with: apps <name>");
    println!();
    print_app_entry(marker, "notes", "Write and review simple notes", "apps notes");
    print_app_entry(marker, "calc", "Open the built-in calculator", "apps calc");
    print_app_entry(marker, "sysinfo", "Show system information", "apps sysinfo");
    print_app_entry(marker, "files", "Browse file commands and ChronoFS tools", "apps files");
    print_app_entry(marker, "clock", "Show current clock information", "apps clock");
    print_app_entry(marker, "museum", "Explore educational OS exhibits", "apps museum");
    print_app_entry(marker, "theme", "Preview era/theme switching commands", "apps theme");
    print_app_entry(marker, "tasks", "Inspect task and scheduler commands", "apps tasks");
    println!();
    println!("This is a text launcher, not a desktop. It reuses existing commands where safe.");
}

fn print_app_entry(marker: &str, name: &str, summary: &str, command: &str) {
    println!("{} {:8} - {} ({})", marker, name, summary, command);
}

fn launch_existing_app(command: &str) {
    println!("Launching existing command: {}", command);
    execute_command(command);
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
            println!("Time Capsule OS currently maps years from 1980 onward.");
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
    println!("What Time Capsule OS currently does");
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
        "Time Capsule OS uses disk-backed paths for ChronoFS when available, and falls back conservatively when persistent disk state is not available.",
        "Real operating systems have storage drivers, request queues, caches, partitions, error handling, and careful ordering so data survives crashes.",
        "Robust driver coverage, broad device support, deep caching, full crash-safety, and runtime-verified recovery are still missing.",
    );
}

fn print_museum_filesystem_page() {
    print_deep_museum_page(
        "filesystem",
        "A filesystem gives names and metadata to bytes on disk so people can work with files instead of raw sectors.",
        "Time Capsule OS has ChronoFS with simple named files, contiguous extents, ls/cat/write/rm, fsck/repair, and a tiny journal for metadata safety.",
        "Real operating systems add directories, permissions, caching, journaling or copy-on-write, quotas, timestamps, and many compatibility rules.",
        "ChronoFS is still intentionally small: no full POSIX model, no directories, limited recovery, and no broad real-world filesystem compatibility.",
    );
}

fn print_museum_userspace_page() {
    print_deep_museum_page(
        "userspace",
        "Userspace is where regular programs run with less privilege than the kernel, so mistakes are easier to contain.",
        "Time Capsule OS has educational paths like ring3, syshello, and exec <name> that show the idea without pretending to be a full process platform.",
        "Real operating systems isolate processes with virtual memory, permissions, handles, signals, scheduling, and carefully validated kernel entry points.",
        "A mature process model, strong isolation, broad runtime verification, and production-grade program lifecycle management are still missing.",
    );
}

fn print_museum_syscalls_page() {
    print_deep_museum_page(
        "syscalls",
        "A syscall is a controlled doorway from userspace into the kernel for services like output, files, memory, and time.",
        "Time Capsule OS includes simple syscall-style teaching demos through userspace examples such as syshello.",
        "Real operating systems define stable syscall ABIs, validate every pointer and argument, enforce permissions, and keep compatibility for years.",
        "A broad syscall table, hardened validation, permissions, tracing, and complete ABI documentation are still missing.",
    );
}

fn print_museum_elf_page() {
    print_deep_museum_page(
        "ELF",
        "ELF is a common executable file format that describes program code, data, entry points, and loading rules.",
        "Time Capsule OS exposes exec <name> as a code-present educational path for loading stored programs where supported.",
        "Real operating systems parse many ELF sections and segments, map pages, handle relocations, link libraries, and prepare process state.",
        "Broad ELF feature support, dynamic linking, relocation coverage, safety hardening, and runtime verification are still missing.",
    );
}

fn print_museum_networking_page() {
    print_deep_museum_page(
        "networking",
        "Networking moves packets between machines through devices, protocols, addresses, and routing rules.",
        "Time Capsule OS treats networking conservatively in the shell reports; no full verified network stack is claimed here.",
        "Real operating systems drive network cards, handle interrupts, implement protocols like Ethernet/IP/TCP/UDP, and expose sockets to programs.",
        "A verified driver path, protocol stack, socket API, packet tools, and runtime network tests are still missing.",
    );
}

fn print_museum_smp_page() {
    print_deep_museum_page(
        "SMP",
        "SMP means symmetric multiprocessing: more than one CPU core can run kernel or program work at the same time.",
        "Time Capsule OS has code-present core-count and multicore teaching concepts, surfaced through commands like cores.",
        "Real operating systems bring up application processors, send inter-processor interrupts, balance work, and protect shared data with locks.",
        "Deep multicore verification, mature synchronization, cross-core scheduling confidence, and stress testing are still missing.",
    );
}

fn print_museum_scheduler_page() {
    print_deep_museum_page(
        "scheduler",
        "A scheduler chooses which task gets CPU time next, making one processor feel like it can do many things.",
        "Time Capsule OS has task and scheduler teaching paths that are useful previews, not a production scheduler audit.",
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
    print_poster_row("poster roadmap", "code-present, partial, planned");
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
    print_poster_row("serial", "code-present");
    print_poster_row("framebuffer", "code-present");
    print_poster_row("timer", "code-present");
    print_poster_row("keyboard", "code-present");
    print_poster_row("mouse", "unknown");
    print_poster_row("filesystem", "code-present: ChronoFS, fsck, journal");
    print_poster_row("heap", "code-present");
    print_poster_row("network", "unknown");
    print_poster_row("SMP/core count", "code-present");
    print_poster_row("scheduler", "partial teaching path");
    print_poster_row("userspace/ELF", "code-present, verification needed");
    println!();
    print_poster_row("Note", "poster does not run doctor or probes");
}

fn poster_roadmap() {
    print_poster_header("BUILD-IN-PUBLIC ROADMAP");
    println!("code-present");
    print_poster_row("Shell", "help, eras, apps, guides, museum");
    print_poster_row("ChronoFS", "files, fsck, repair, tiny journal");
    print_poster_row("Learning", "demo, tour, capsule, doctor, poster");
    println!();
    println!("partial");
    print_poster_row("Windows/tasks", "useful previews, not full desktop");
    print_poster_row("Userspace", "teaching path, not mature platform");
    print_poster_row("Recovery", "conservative, refuses guesses");
    println!();
    println!("planned");
    print_poster_row("Verification", "build and shell confidence loop");
    print_poster_row("Lessons", "richer quests and museum trails");
    print_poster_row("Apps", "small workflows before big systems");
    println!();
    println!("runtime verification needed");
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

    println!("Chronosapian | Era: {} | v0.1", profile.name);
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

fn cat_file(command: &str) {
    let name = command.strip_prefix("cat").unwrap_or("").trim();

    if name.is_empty() {
        println!("Usage: cat <name>");
        return;
    }

    match fs::read(name) {
        Ok(content) => println!("{}", content),
        Err(error) => print_fs_error(name, error),
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
        Err(error) => print_fs_error(name, error),
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
        Err(error) => print_fs_error(name, error),
    }
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
        Err(error) => print_fs_error(name, error),
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

    let report = fs::check(repair);

    println!("ChronoFS check: {}", report.status_label());
    println!(
        "Entries: checked={} live={} invalid={}",
        report.checked_entries, report.live_entries, report.invalid_entries
    );
    println!(
        "Bitmap mismatches: {} | duplicate sectors: {} | repaired: {}",
        report.bitmap_mismatches, report.duplicate_sectors, report.repaired_items
    );
    println!("Warnings: {} | errors: {}", report.warnings, report.errors);

    if report.findings.is_empty() {
        println!("No problems found.");
        return;
    }

    for finding in report.findings {
        println!("- {}", finding);
    }
}

fn run_journal(command: &str) {
    let mode = command.strip_prefix("journal").unwrap_or("").trim();
    if !mode.is_empty() {
        println!("Usage: journal");
        return;
    }

    let status = fs::journal_status();
    println!("ChronoFS journal: {}", status.state);
    println!("Available: {}", if status.available { "yes" } else { "no" });
    println!("Clean: {}", if status.clean { "yes" } else { "no" });
    println!("Operation: {}", status.operation);
    if !status.target.is_empty() {
        println!("Target: {}", status.target);
    }
    println!("{}", status.message);
}

fn open_window(command: &str) {
    let name = command.strip_prefix("open").unwrap_or("").trim();

    match name {
        "notes" => {
            // Spawn the task first so we can hand its ID to the window.
            let task_id = sched::spawn("notes", apps::notes_task_entry).unwrap_or(0xFF);
            if !wm::open_notes(task_id) {
                // Window failed to open — roll back the task slot.
                sched::kill(task_id);
                println!("too many windows open");
            }
        }
        "sysinfo" => {
            let task_id = sched::spawn("sysinfo", apps::sysinfo_task_entry).unwrap_or(0xFF);
            if !wm::open_sysinfo(task_id) {
                sched::kill(task_id);
                println!("too many windows open");
            }
        }
        _ => {
            println!("Usage: open notes|sysinfo");
        }
    }
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

fn print_fs_error(name: &str, error: FsError) {
    match error {
        FsError::NotFound => println!("file not found: {}", name),
        FsError::EmptyName | FsError::InvalidName => println!("invalid filename"),
        FsError::NameTooLong => println!("filename too long"),
        FsError::FileTooLarge => println!("file too large"),
        FsError::NoSpace => println!("not enough disk space"),
        FsError::Disk => println!("disk error"),
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
