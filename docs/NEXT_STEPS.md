# Next Steps

## Current Priority Queue

1. Establish build sanity without changing architecture: verify toolchain, regenerate/validate lockfile only in a dedicated build task, and fix compile errors narrowly.
2. Align documentation with current source reality: README, roadmap, architecture notes, and command lists should distinguish code-present from runtime-verified.
3. Run staged manual verification of existing systems before adding new OS features.

## Technical Gaps

- Interrupt-driven keyboard: code-present but unverified; IRQ1 buffering was added and the polling path should remain usable as a fallback.
- Reusable heap allocator: code-present but unverified; current allocator uses a simple free list and needs build/runtime validation.
- ChronoFS repair/journaling: missing; current filesystem has no crash recovery or repair path.
- Graphics shell: partial; framebuffer console and small windows exist, but there is no full desktop/compositor.
- Process model: partial; ring 3, syscalls, and static ELF execution exist, but there is no dynamic linker, argv/env, or general multiprocess model.
- Scheduler: partial; cooperative task scheduling exists, but preemption and production-grade scheduling are not current goals.
- Networking: partial; ARP/UDP over RTL8139 exists in code, but DHCP, TCP, DNS, and broad hardware support are missing.
- Real hardware/USB: largely missing; UEFI path exists, but USB HID, USB storage, USB serial, and broad real hardware support need future design.

## Product / Indie Features

- Demo mode: planned idea; do not build until core runtime behavior is verified.
- Tour command: planned idea for guided educational exploration.
- Capsule timeline: planned idea for era/progress storytelling.
- Doctor command: planned idea for self-diagnostics and environment checks.
- Theme studio: planned idea for editing or previewing era themes.
- Travel command: planned idea for switching eras with more ceremony than the current `era` command.
- Poster mode: planned idea for shareable/demo-friendly visuals.

## Do Not Build Yet

Features that are too big or risky right now:

- Full TCP/IP stack
- Full desktop compositor
- Dynamic linker
- Package manager
- Complex GUI toolkit
- Full USB stack
- Browser
- Production-grade scheduler
