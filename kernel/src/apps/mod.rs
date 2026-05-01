//! Tiny built-in apps launched by shell command.

mod calc;
mod notes;
mod sysinfo;

pub fn run(command: &str) -> bool {
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
