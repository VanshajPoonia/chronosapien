# ChronoOS Resume Bullets

Use these bullets as starting points. Keep final wording aligned with the role
and with `docs/VERIFICATION_MATRIX.md`.

## 5 Concise Resume Bullets

- Built ChronoOS, an educational Rust `no_std` x86_64 hobby OS with a
  shell-first product experience.
- Implemented ChronoFS tooling for file commands, filesystem inspection, fsck
  diagnostics, conservative repair boundaries, and journal status.
- Designed in-OS learning surfaces including eras, museum pages, quests, guided
  onboarding, demo scripts, and status screens.
- Added static app, window, userspace, networking, and safe-mode observability
  layers without introducing heavy runtime dependencies.
- Maintained evidence-based documentation separating QEMU-verified behavior,
  code-present systems, blocked tooling, and roadmap-only work.

## 5 More Technical Resume Bullets

- Developed Rust `no_std` x86_64 kernel paths covering boot handoff, serial and
  framebuffer output, GDT/IDT/PIC/PIT setup, PS/2 input paths, and heap/storage
  foundations.
- Built ChronoFS as a compact educational filesystem with fixed file slots,
  bounded file size, ATA-backed storage paths, consistency checks, and journal
  recovery documentation.
- Created read-only shell namespaces for filesystem, userspace, networking,
  app registry, window lifecycle, safe mode, and learning-path diagnostics.
- Documented Ring 3, syscall, and static ELF execution boundaries while keeping
  dynamic linking, package management, argv/env, and full process semantics out
  of scope.
- Added network observability for a static IPv4 RTL8139 ARP/UDP stack while
  keeping DHCP, DNS, TCP, sockets, and packet capture as roadmap-only systems.

## 5 Product-Minded Resume Bullets

- Turned a hobby OS into an educational product by making the shell a guided
  museum-style interface instead of only a debug console.
- Created demo, showcase, release, screenshot, and portfolio docs that make a
  low-level systems project understandable to technical and non-technical
  audiences.
- Designed safe/demo/experimental command framing to help users distinguish
  read-only demos from controlled verification and risky labs.
- Built a static app registry and learning paths that improve discoverability
  without pretending ChronoOS has dynamic app loading.
- Practiced transparent build-in-public communication by pairing technical
  progress with known limitations and verification status.

## 3 Short Project Descriptions

1. ChronoOS is an educational Rust `no_std` x86_64 hobby operating system that
   combines low-level kernel work with a shell-first learning product.
2. ChronoOS v0.1 RC, "Time-Museum Shell", presents boot, memory, filesystem,
   app, userspace, networking, and roadmap concepts through eras, museum pages,
   quests, guides, and conservative status screens.
3. ChronoOS is a portfolio systems project focused on readable kernel work,
   honest verification, and beginner-friendly explanations from inside the OS.

## GitHub README Summary

ChronoOS is a Rust `no_std` x86_64 educational hobby OS with a product-minded
shell: eras, museum pages, quests, ChronoFS, small app surfaces, learning paths,
safe-mode framing, and evidence-based docs that separate QEMU-tested behavior
from code-present and roadmap-only systems.

## Portfolio-Card Summary

ChronoOS is a tiny educational operating system and portfolio case study: real
Rust `no_std` x86_64 kernel work wrapped in a memorable time-museum shell, with
ChronoFS, learning paths, app/status surfaces, and honest verification labels.
