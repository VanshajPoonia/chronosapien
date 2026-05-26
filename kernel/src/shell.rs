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
        "Commands: help, demo, tour, clear, about, reboot, era, uptime, clock, mem, cores, beep <hz>, ring3, syshello"
    );
    println!("Files: ls, cat <name>, write <name> <content>, rm <name>, exec <name>, fsck [repair], journal");
    println!("Apps: notes, calc, sysinfo");
    println!("Windows: open notes, open sysinfo");
    println!("Tasks: tasks, kill <id>");
    println!("Network: net, net arp, net send [ip port text]");
    println!("Museum: museum boot|kernel|memory|interrupts|keyboard|serial|era");
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
