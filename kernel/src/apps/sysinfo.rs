//! Era-styled system information app.

use crate::theme::Era;

pub fn run(args: &str) {
    if !args.trim().is_empty() {
        crate::println!("Usage: sysinfo");
        return;
    }

    let era = crate::theme::active_era();
    let profile = crate::theme::active_profile();
    let memory = crate::memory::stats();
    let uptime = crate::timer::uptime_seconds();
    let used_kb = memory.heap_used_bytes / 1024;

    match era {
        Era::Eighties => {
            crate::println!("== CHRONO SYSINFO 1984 ==");
            crate::println!("OS...... ChronoOS");
            crate::println!("ERA..... {}", profile.name);
            crate::println!("UPTIME.. {} seconds", uptime);
            crate::println!("MEM..... {} KB used", used_kb);
        }
        Era::Nineties => {
            crate::println!("C:\\CHRONO\\SYSINFO.EXE");
            crate::println!("OS      : ChronoOS");
            crate::println!("ERA     : {}", profile.name);
            crate::println!("UPTIME  : {} seconds", uptime);
            crate::println!("MEM USED: {} KB", used_kb);
        }
        Era::TwoThousands => {
            crate::println!("chrono sysinfo");
            crate::println!("os: ChronoOS");
            crate::println!("era: {}", profile.name);
            crate::println!("uptime: {} seconds", uptime);
            crate::println!("mem_used_kb: {}", used_kb);
        }
        Era::Future => {
            crate::println!("[chrono::sysinfo]");
            crate::println!("os=ChronoOS era={}", profile.name);
            crate::println!("uptime_s={} mem_kb={}", uptime, used_kb);
        }
    }
}
