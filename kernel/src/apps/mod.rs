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
}

pub struct AppManifest {
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub launch_command: &'static str,
    pub status: AppStatus,
    pub verification: VerificationStatus,
    pub risk: RiskLevel,
}

pub const APP_REGISTRY: &[AppManifest] = &[
    AppManifest {
        name: "notes",
        category: "apps",
        description: "One-line ChronoFS-backed notes surface.",
        launch_command: "notes",
        status: AppStatus::Implemented,
        verification: VerificationStatus::PartiallyVerifiedQemu,
        risk: RiskLevel::Medium,
    },
    AppManifest {
        name: "calc",
        category: "apps",
        description: "Tiny integer calculator.",
        launch_command: "calc",
        status: AppStatus::Implemented,
        verification: VerificationStatus::PartiallyVerifiedQemu,
        risk: RiskLevel::Low,
    },
    AppManifest {
        name: "sysinfo",
        category: "apps",
        description: "Era-aware system information screen.",
        launch_command: "sysinfo",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
    },
    AppManifest {
        name: "files",
        category: "storage",
        description: "Shell-first ChronoFS command card.",
        launch_command: "apps files",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Medium,
    },
    AppManifest {
        name: "museum",
        category: "education",
        description: "Educational OS exhibit launcher.",
        launch_command: "apps museum",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
    },
    AppManifest {
        name: "theme",
        category: "eras",
        description: "Era/theme preview card, not a full studio.",
        launch_command: "apps theme",
        status: AppStatus::PartiallyImplemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
    },
    AppManifest {
        name: "tasks",
        category: "system",
        description: "Cooperative task inspection route.",
        launch_command: "tasks",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Medium,
    },
    AppManifest {
        name: "paint",
        category: "creative",
        description: "Tiny paint idea for a future app surface.",
        launch_command: "",
        status: AppStatus::Roadmap,
        verification: VerificationStatus::RoadmapOnly,
        risk: RiskLevel::Future,
    },
    AppManifest {
        name: "network",
        category: "networking",
        description: "Future guided ARP/UDP demo mode.",
        launch_command: "net",
        status: AppStatus::Roadmap,
        verification: VerificationStatus::RoadmapOnly,
        risk: RiskLevel::High,
    },
    AppManifest {
        name: "userspace",
        category: "systems",
        description: "Ring 3/syscall/static ELF teaching surface.",
        launch_command: "userspace status",
        status: AppStatus::PartiallyImplemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::High,
    },
    AppManifest {
        name: "timeline",
        category: "product",
        description: "Text boot/product timeline through capsule and poster.",
        launch_command: "capsule",
        status: AppStatus::PartiallyImplemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
    },
    AppManifest {
        name: "crashlab",
        category: "debug",
        description: "Future controlled crash/fault learning lab.",
        launch_command: "",
        status: AppStatus::Roadmap,
        verification: VerificationStatus::RoadmapOnly,
        risk: RiskLevel::Future,
    },
    AppManifest {
        name: "doctor",
        category: "status",
        description: "Conservative status/verification center.",
        launch_command: "doctor",
        status: AppStatus::Implemented,
        verification: VerificationStatus::NeedsRuntimeVerification,
        risk: RiskLevel::Low,
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
