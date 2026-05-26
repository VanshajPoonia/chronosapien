//! Era-styled system information app.

pub fn run(args: &str) {
    if !args.trim().is_empty() {
        crate::println!("Usage: sysinfo");
        return;
    }

    let profile = crate::theme::active_profile();
    let style = profile.sysinfo;
    let memory = crate::memory::stats();
    let uptime = crate::timer::uptime_seconds();
    let used_kb = memory.heap_used_bytes / 1024;
    let free_kb = memory.heap_free_bytes / 1024;

    crate::println!("{}", style.header);
    if style.compact {
        crate::println!(
            "{}{}Chronosapian {}{}{}",
            style.os_label,
            style.separator,
            style.era_label,
            style.separator,
            profile.name
        );
        crate::println!(
            "{}{}{} {}{}{} free{}{}",
            style.uptime_label,
            style.separator,
            uptime,
            style.mem_label,
            style.separator,
            used_kb,
            style.separator,
            free_kb
        );
    } else {
        crate::println!("{}{}Chronosapian", style.os_label, style.separator);
        crate::println!("{}{}{}", style.era_label, style.separator, profile.name);
        crate::println!(
            "{}{}{} seconds",
            style.uptime_label,
            style.separator,
            uptime
        );
        crate::println!("{}{}{} KB", style.mem_label, style.separator, used_kb);
        crate::println!("FREE{}{} KB", style.separator, free_kb);
    }
}
