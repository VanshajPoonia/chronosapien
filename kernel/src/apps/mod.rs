//! Tiny built-in apps launched by shell command.

mod calc;
mod notes;
mod sysinfo;

/// Task entry point for a Notes window — yields cooperatively in a loop.
pub fn notes_task_entry() -> ! {
    loop {
        crate::sched::yield_now();
    }
}

/// Task entry point for a Sysinfo window — yields cooperatively in a loop.
pub fn sysinfo_task_entry() -> ! {
    loop {
        crate::sched::yield_now();
    }
}

pub fn run(command: &str) -> bool {
    // Return true only when the shell command belongs to a built-in app.
    if command == "notes" || command.starts_with("notes ") {
        crate::serial_println!("[CHRONO] app: notes launched");
        notes::run(command.strip_prefix("notes").unwrap_or("").trim_start());
        return true;
    }

    if command == "calc" || command.starts_with("calc ") {
        crate::serial_println!("[CHRONO] app: calc launched");
        calc::run(command.strip_prefix("calc").unwrap_or("").trim_start());
        return true;
    }

    if command == "sysinfo" || command.starts_with("sysinfo ") {
        crate::serial_println!("[CHRONO] app: sysinfo launched");
        sysinfo::run(command.strip_prefix("sysinfo").unwrap_or("").trim_start());
        return true;
    }

    false
}
