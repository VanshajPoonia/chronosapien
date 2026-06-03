# ChronoOS Known Limitations

ChronoOS is an educational Rust `no_std` x86_64 hobby OS. It is designed to be
readable, demo-friendly, and honest about its verification state.

## Not A Linux Replacement

ChronoOS is not a Linux replacement, not a POSIX system, and not a production
operating system. It is a learning OS with a portfolio/product shell.

## Educational Hobby OS

Many systems are intentionally small teaching versions. They are useful for
learning how an OS is shaped, but they are not hardened production subsystems.

## Runtime Verification Is Incomplete

The repo records limited QEMU evidence: single-core BIOS serial-only boot
reached `[CHRONO] boot complete`. That evidence does not prove visible
framebuffer output, shell interaction, keyboard input, apps, filesystem
workflows, userspace, networking, mouse/windows, UEFI, custom BIOS, SMP/AP, or
hardware behavior.

## Hardware Support Limitations

- BIOS and UEFI paths exist in code/docs, but only the normal BIOS path has
  limited serial-only QEMU proof.
- No hardware boot proof is recorded.
- USB HID, USB storage, USB serial, and broad hardware support are
  roadmap/design-only.
- Hardware-specific failures should be documented before adding new hardware
  features.

## Networking Limitations

- Current networking is static IPv4 ARP/UDP over RTL8139 teaching code.
- `net status`, `net config`, `net arp`, `net udp`, `net send`, `net log`,
  `net demo`, and `net roadmap` exist in code but need runtime verification.
- DHCP, DNS, TCP, sockets, and a full network stack are roadmap/design-only.
- Networking commands should not be presented as broad hardware or internet
  connectivity.

## Filesystem Limitations

- ChronoFS is a small educational filesystem.
- `ls`, `cat`, `write`, `rm`, `fsck`, `fsck repair`, and `journal` exist in
  code but still need shell-level runtime verification.
- `fsck repair` is conservative and should be tested only with controlled disk
  images.
- The one-record journal is a teaching mechanism, not a general crash-safe
  filesystem guarantee.

## Userspace Limitations

- Ring 3, syscalls, and static ELF execution are teaching paths.
- There is no general userland, libc, dynamic linker, package manager, argv/env
  model, process isolation story, or mature app loading platform.
- `ring3`, `syshello`, and `exec <name>` should be run only during intentional
  verification until stronger evidence exists.

## GUI And Windowing Limitations

- The framebuffer console, top bar, mouse path, and small windows exist in code.
- The window layer is fixed-capacity and educational.
- There is no full compositor, desktop environment, GUI toolkit, or windowed
  file explorer.
- Mouse movement, drag, focus, close, and window-task interaction need visible
  runtime verification.

## Scheduler And SMP Limitations

- Cooperative scheduler paths exist in code.
- SMP/AP startup is partially implemented and high-risk.
- A two-core serial-only smoke exited before `[CHRONO] boot complete`.
- Production-grade preemptive scheduling is roadmap/design-only.

## Long-Term Systems Not Implemented

These are not part of v0.1 unless a future source audit and runtime evidence say
otherwise:

- TCP, DHCP, DNS, sockets, or full internet networking.
- USB HID, USB storage, USB serial, or broad hardware support.
- Dynamic linker or package manager.
- Full desktop compositor or browser.
- Production-grade preemptive scheduler.
- General-purpose userland.
