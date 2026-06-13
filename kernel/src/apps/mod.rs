//! Tiny built-in apps launched by shell command.

mod calc;
mod notes;
mod sysinfo;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum AppStatus {
    Implemented,
    PartiallyImplemented,
    Roadmap,
}

impl AppStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Implemented => "implemented in code",
            Self::PartiallyImplemented => "partially implemented",
            Self::Roadmap => "roadmap/design-only",
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum VerificationStatus {
    PartiallyVerifiedQemu,
    NeedsRuntimeVerification,
    RoadmapOnly,
}

impl VerificationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::PartiallyVerifiedQemu => "partially verified in QEMU",
            Self::NeedsRuntimeVerification => "needs runtime verification",
            Self::RoadmapOnly => "roadmap/design-only",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::PartiallyVerifiedQemu => "qemu-partial",
            Self::NeedsRuntimeVerification => "needs-test",
            Self::RoadmapOnly => "roadmap",
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Future,
}

impl RiskLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Future => "future",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::Low => "risk:low",
            Self::Medium => "risk:medium",
            Self::High => "risk:high",
            Self::Future => "risk:future",
        }
    }
}

pub struct AppManifest {
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub launch_command: &'static str,
    pub status: AppStatus,
    pub verification: VerificationStatus,
    pub risk: RiskLevel,
    pub related_commands: &'static [&'static str],
    pub help: &'static str,
    pub demo_commands: &'static [&'static str],
    pub featured: bool,
}

pub const APP_REGISTRY: &[AppManifest] = &[
    AppManifest {
        name: "notes",
        category: "Core",
        description: "One-line ChronoFS-backed notes surface.",
        launch_command: "notes",
        status: AppStatus::Implemented,
        verification: VerificationStatus::PartiallyVerifiedQemu,
        risk: RiskLevel::Medium,
        related_commands: &["notes", "notes read", "notes write <text>", "notes open", "open notes"],
        help: "Use notes for a tiny ChronoFS-backed scratchpad. Persistence depends on ChronoFS behavior.",
        demo_commands: &["notes", "notes write hello from ChronoOS", "notes read"],
        featured: true,
    },
    AppManifest {
        name: "calc",
        category: "Core",
        description: "Tiny integer calculator.",
        launch_command: "calc",
        status: AppStatus::Implemented,
        verification: VerificationStatus::PartiallyVerifiedQemu,
        risk: RiskLevel::Low,
        related_commands: &["calc 6 * 7", "calc 6 - 7"],
        help: "Use calc for simple integer arithmetic from the shell.",
        demo_commands: &["calc 6 * 7", "calc 6 - 7"],
        featured: true,
    },
    AppManifest {
        name: "sysinfo",
        category: "System",
        description: "Era-aware system information screen.",
        launch_command: "sysinfo",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
        related_commands: &["sysinfo", "open sysinfo", "mem", "uptime", "clock"],
        help: "Use sysinfo for a compact status screen; open sysinfo uses the partial window path.",
        demo_commands: &["sysinfo", "mem", "uptime"],
        featured: true,
    },
    AppManifest {
        name: "files",
        category: "Files",
        description: "Shell-first ChronoFS usability surface.",
        launch_command: "files",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Medium,
        related_commands: &["files", "files list", "files info <name>", "fs status", "journal"],
        help: "Use files to inspect ChronoFS through beginner-friendly commands.",
        demo_commands: &["files", "files sample", "files list", "fs status"],
        featured: true,
    },
    AppManifest {
        name: "museum",
        category: "Learning",
        description: "Educational OS exhibit launcher.",
        launch_command: "apps museum",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
        related_commands: &["museum filesystem", "museum userspace", "tour apps", "learn"],
        help: "Use museum to read focused OS concept exhibits without changing state.",
        demo_commands: &["apps museum", "museum filesystem", "museum userspace"],
        featured: true,
    },
    AppManifest {
        name: "learn",
        category: "Learning",
        description: "Guided learning paths through ChronoOS concepts.",
        launch_command: "learn",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
        related_commands: &["learn", "learn filesystem", "learn userspace", "guide quick"],
        help: "Use learn for beginner-friendly subsystem paths.",
        demo_commands: &["learn", "learn filesystem", "tour apps"],
        featured: true,
    },
    AppManifest {
        name: "theme",
        category: "Visual",
        description: "Era/theme preview card, not a full studio.",
        launch_command: "apps theme",
        status: AppStatus::PartiallyImplemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
        related_commands: &["theme", "era", "era 1995", "poster eras", "travel 2004"],
        help: "Use theme and era commands to preview the ChronoOS time-capsule identity.",
        demo_commands: &["theme", "era", "poster eras"],
        featured: true,
    },
    AppManifest {
        name: "tasks",
        category: "System",
        description: "Cooperative task inspection route.",
        launch_command: "tasks",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Medium,
        related_commands: &["tasks", "open notes", "open sysinfo", "windows list"],
        help: "Use tasks only after opening window apps; task kill still needs careful verification.",
        demo_commands: &["windows status", "open notes", "tasks"],
        featured: false,
    },
    AppManifest {
        name: "clock",
        category: "System",
        description: "Simple timer tick and uptime helpers.",
        launch_command: "clock",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
        related_commands: &["clock", "uptime"],
        help: "Use clock and uptime for tiny time/status checks from the shell.",
        demo_commands: &["clock", "uptime"],
        featured: false,
    },
    AppManifest {
        name: "status",
        category: "System",
        description: "Status, verification, and doctor command hub.",
        launch_command: "workspace",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
        related_commands: &["workspace", "status", "verify", "doctor", "mode status"],
        help: "Use status surfaces to understand what is verified, blocked, or still risky.",
        demo_commands: &["workspace", "verify", "doctor"],
        featured: true,
    },
    AppManifest {
        name: "paint",
        category: "Roadmap/Future",
        description: "Tiny paint idea for a future app surface.",
        launch_command: "",
        status: AppStatus::Roadmap,
        verification: VerificationStatus::RoadmapOnly,
        risk: RiskLevel::Future,
        related_commands: &["apps roadmap"],
        help: "Paint is a roadmap idea; no canvas app exists yet.",
        demo_commands: &["apps roadmap"],
        featured: false,
    },
    AppManifest {
        name: "network",
        category: "Networking",
        description: "Future guided ARP/UDP demo mode.",
        launch_command: "net",
        status: AppStatus::Roadmap,
        verification: VerificationStatus::RoadmapOnly,
        risk: RiskLevel::High,
        related_commands: &["net status", "net config", "net log", "net demo"],
        help: "Network app expansion is deferred; current net commands are teaching/status surfaces.",
        demo_commands: &["net status", "net demo"],
        featured: false,
    },
    AppManifest {
        name: "userspace",
        category: "System",
        description: "Ring 3/syscall/static ELF teaching surface.",
        launch_command: "userspace status",
        status: AppStatus::PartiallyImplemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::High,
        related_commands: &["userspace status", "userspace syscalls", "ring3", "syshello"],
        help: "Userspace is an educational boundary, not a full process platform.",
        demo_commands: &["userspace status", "userspace syscalls"],
        featured: false,
    },
    AppManifest {
        name: "timeline",
        category: "Learning",
        description: "Text boot/product timeline through capsule and poster.",
        launch_command: "capsule",
        status: AppStatus::PartiallyImplemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
        related_commands: &["capsule", "capsule current", "poster boot", "poster roadmap"],
        help: "Use timeline surfaces to tell the product story; graphical timeline is future work.",
        demo_commands: &["capsule", "poster boot", "poster roadmap"],
        featured: false,
    },
    AppManifest {
        name: "crashlab",
        category: "Debug/Lab",
        description: "Future controlled crash/fault learning lab.",
        launch_command: "",
        status: AppStatus::Roadmap,
        verification: VerificationStatus::RoadmapOnly,
        risk: RiskLevel::Future,
        related_commands: &["apps roadmap"],
        help: "Crash lab is deferred until controlled fault demos are designed and verified.",
        demo_commands: &["apps roadmap"],
        featured: false,
    },
    AppManifest {
        name: "doctor",
        category: "System",
        description: "Conservative status/verification center.",
        launch_command: "doctor",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
        related_commands: &["doctor", "verify", "mode status", "poster system"],
        help: "Use doctor for a conservative subsystem report; it does not certify runtime behavior.",
        demo_commands: &["doctor", "verify", "mode status"],
        featured: false,
    },
];

pub fn registry() -> &'static [AppManifest] {
    APP_REGISTRY
}

pub fn find_manifest(name: &str) -> Option<&'static AppManifest> {
    APP_REGISTRY.iter().find(|app| app.name == name)
}

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
