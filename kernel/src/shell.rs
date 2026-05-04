//! Tiny line-based shell for the first interactive milestone.

use crate::apps;
use crate::console;
use crate::fs::{self, FsError};
use crate::keyboard::{self, KeyEvent};
use crate::memory;
use crate::mouse;
use crate::museum;
use crate::sched;
use crate::theme::{self, Era};
use crate::timer;
use crate::wm;
use crate::{print, println, serial_println};

const COMMAND_BUFFER_CAPACITY: usize = 80;
const CURSOR_BLINK_TICKS: usize = 80_000;
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

                if idle_ticks >= CURSOR_BLINK_TICKS {
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
        "clear" => console::clear(),
        "about" => print_about(),
        "reboot" => reboot(),
        "uptime" => print_uptime(),
        "clock" => print_clock(),
        "mem" => print_memory(),
        "ls" => list_files(),
        command if command == "cat" || command.starts_with("cat ") => cat_file(command),
        command if command == "write" || command.starts_with("write ") => write_file(command),
        command if command == "rm" || command.starts_with("rm ") => remove_file(command),
        command if command == "era" || command.starts_with("era ") => run_era_command(command),
        command if command == "open" || command.starts_with("open ") => open_window(command),
        "tasks" => list_tasks(),
        command if command == "kill" || command.starts_with("kill ") => kill_task(command),
        command if museum::run(command) => {}
        command if apps::run(command) => {}
        _ => println!("unknown command: {}", command),
    }
}

fn print_help() {
    println!("Commands: help, clear, about, reboot, era, uptime, clock, mem");
    println!("Files: ls, cat <name>, write <name> <content>, rm <name>");
    println!("Apps: notes, calc, sysinfo");
    println!("Windows: open notes, open sysinfo");
    println!("Tasks: tasks, kill <id>");
    println!("Museum: museum boot|kernel|memory|interrupts|keyboard|serial|era");
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
    print!("_");
}

fn erase_cursor() {
    console::backspace();
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
