# Shell Command Reference

This reference is based on the current command dispatch in `kernel/src/shell.rs`
plus the built-in app, museum, quest, and networking modules.

Status: implemented in code, needs runtime verification.

Notes storage uses the ChronoFS file named `notes`. The direct `notes` shell
command is handled by `kernel/src/shell.rs`; window mode reads the same file.

## Core Shell

- `help`: list categorized command groups.
- `help search <term>`: search the small built-in help index without running
  probes.
- `help workspace`: explain `workspace`, `shortcuts`, `whereami`, `recent`,
  `status`, and `verify`.
- `help start`: explain `start`, `welcome`, `guide`, `demo`, and `tour`.
- `help learn`: explain structured learning paths.
- `help mode`: explain reliability/safe mode.
- `help apps`: explain the app launcher and window paths.
- `help fs`: explain `fs`, `ls`, `cat`, `write`, `rm`, `fsck`, and `journal`.
- `help system`: explain conservative status surfaces.
- `help network`: explain static IPv4 ARP/UDP networking boundaries.
- `help userspace`: explain the `userspace` inspection namespace plus
  `ring3`, `syshello`, and `exec <name>`.
- `help labs`: explain risky/intentional verification commands.
- `help roadmap`: explain roadmap/design-only systems.
- `help files`, `help theme`, `help net`, `help status`, `help verify`, and
  `help future`: beginner-friendly topic aliases.
- `workspace`: print a compact shell dashboard.
- `status`: alias for `workspace`.
- `verify`: print a read-only verification boundary summary.
- `shortcuts`: list the best demo/useful commands.
- `whereami`: explain current mode, era, UI context, and next action.
- `recent`: show a fixed-size in-memory list of commands typed since boot.
- `files`: show the shell-first ChronoFS command map and safety notes.
- `theme`: show the era/theme command card.
- `clear`: clear the framebuffer shell region and redraw the top bar.
- `about`: print the ChronoOS identity line.
- `reboot`: request a reset through the PS/2 controller.
- `uptime`: print elapsed seconds from PIT ticks.
- `clock`: print raw PIT ticks.
- `mem`: print total memory and heap used/free/largest-free values.
- `cores`: print online core count and task assignment counts.
- `beep <hz>`: play a PC speaker tone for 500 ms.
- `mode` / `mode status`: show the current safe/demo/experimental warning
  mode.
- `mode safe|demo|experimental`: set the shell reliability mode.
- `safe` / `safe status`: alias for `mode status`.
- `safe on`: alias for `mode safe`.
- `safe off`: return to `mode demo`.

## Help Categories

Top-level `help` groups commands by:

- Getting started
- Workspace
- Learning paths
- Reliability mode
- Eras and themes
- Apps
- Filesystem
- Museum and quests
- System status
- Userspace
- Networking
- Debug/lab
- Roadmap/future

The shell intentionally distinguishes overlapping product concepts:

- `guide` orients first-time users.
- `learn` connects curriculum paths to existing commands.
- `demo` previews the current surface without changing state.
- `tour` teaches OS concepts by subsystem.
- `workspace` is the compact dashboard, `status` aliases it, and `verify` is a
  read-only verification summary rather than a live certification command.
- `doctor` remains the conservative subsystem report.
- `apps` is the text launcher; `open` is the partially implemented small-window
  path.
- `fs` is the read-only inspection namespace; `ls`, `cat`, `write`, `rm`,
  `fsck`, and `journal` are the direct filesystem commands; `apps files` points
  users toward them.
- `museum` teaches concepts; `quest` shows progress and next goals.
- `mode` and `safe` categorize commands but do not block them.

## Workspace Commands

- `workspace`: show current era/theme, warning mode, verified BIOS/demo base,
  available app routes, ChronoFS summary, suggested next command, and learning
  suggestion.
- `status`: alias for `workspace`.
- `verify`: summarize QEMU-verified, partial, blocked, and unverified areas
  without running live tests.
- `shortcuts`: list the best first/demo commands.
- `whereami`: explain current shell context and next action.
- `recent`: list recent commands typed since boot. This is a fixed-size,
  in-memory shell log, not persistent history and not arrow-key recall.
- `files`: shell-first ChronoFS overview with inspection and demo guidance.
- `theme`: alias/card for `era`, `travel <year>`, and `poster eras`.

All workspace commands are text-only and read-only. They do not add runtime
verification claims.

## Era And Product Commands

- `start`: print the polished first-run welcome screen.
- `welcome`: alias for `start`.
- `guide`: print the guided onboarding topic menu.
- `guide quick|full|eras|apps|systems|status|next`: print focused first-run guide pages that route to existing commands.
- `learn`: print the curriculum overview.
- `learn boot|memory|interrupts|filesystem|gui|userspace|networking|scheduler|eras|roadmap|next`:
  print structured subsystem learning paths.
- `era 1984|1995|2007|2040`: switch the active era profile.
- `travel <year>`: map a year to an era and switch through the existing `era` path.
- `demo`: print a read-only guided demo.

## Learning Paths

- `learn`: list the curriculum paths.
- `learn boot`: bootloader, kernel handoff, and boot evidence.
- `learn memory`: memory map, paging, heap, and `mem`.
- `learn interrupts`: IDT, timer, keyboard, mouse, and input status.
- `learn filesystem`: ChronoFS, `fs status`, `fs check`, fsck, and journal.
- `learn gui`: apps, the app registry, tiny windows, and window limits.
- `learn userspace`: Ring 3, syscalls, static ELF, and process limits.
- `learn networking`: RTL8139, static IPv4, ARP, UDP, and observability.
- `learn scheduler`: cooperative tasks, SMP/AP boundary, and scheduler limits.
- `learn eras`: era profiles, `travel <year>`, and `poster eras`.
- `learn roadmap`: future systems marked roadmap/design-only.
- `learn map`: compact learning-area status and verification map.
- `learn progress`: static progress/badge/dependency summary.
- `learn beginner`: safe first curriculum route.
- `learn advanced`: intentional verification-oriented route.
- `learn next`: recommends the map and the next safe route.
- `explain <term>`: short glossary entry for common OS terms.

Learning paths are read-only educational screens. They do not run probes or
upgrade runtime verification labels.

## Reliability And Safe Mode

- `mode` / `mode status`: show the current mode, safe demo path, verification
  paths, experimental paths, and caveats.
- `mode safe`: prefer read-only demo paths with stronger warnings.
- `mode demo`: default portfolio/demo mode.
- `mode experimental`: intentional lab/verification mode.
- `safe`, `safe status`, `safe on`, `safe off`: beginner-friendly aliases.

Reliability mode is warning-only, in-memory, and not a security sandbox. It does
not remove commands, block commands, persist across reboot, or upgrade runtime
verification.
- `tour`: print the tour overview.
- `tour boot|memory|files|apps|userspace|future`: print a focused educational page.
- `capsule`: print the build-in-public timeline overview.
- `capsule milestones|current|next`: print milestone/current/next timeline views.
- `doctor`: print a conservative read-only subsystem report.
- `poster`: print a screenshot-friendly overview card.
- `poster boot|system|roadmap|eras`: print focused poster screens.

## Apps

- `apps` / `apps list`: print the static app registry with verification and
  risk badges.
- `apps featured`: show the best shell-first demo app surfaces.
- `apps recent`: show app launcher routes used since boot; this is in-memory
  only.
- `apps category <name>`: browse Core, Files, Learning, System, Networking,
  Visual, Debug/Lab, and Roadmap/Future app categories.
- `apps info <name>`: print one app manifest plus related commands.
- `apps help <name>`: print app-specific help and related commands.
- `apps demo <name>`: print a safe demo path without running commands.
- `apps launch <name>`: run the existing launch command when the app is
  implemented and safe to route through the shell.
- `apps verified`: list app entries with recorded partial QEMU evidence.
- `apps roadmap`: list roadmap/design-only app ideas.
- `apps notes|calc|sysinfo|files|clock|museum|theme|tasks`: legacy direct
  aliases that launch or describe the selected app area.
- `notes`: print the notes home screen.
- `notes read`: read the `notes` ChronoFS file.
- `notes write <text>`: save text to the `notes` ChronoFS file.
- `notes clear`: remove the `notes` ChronoFS file.
- `notes save`: explain that notes save immediately.
- `notes open`: delegate to `open notes`.
- `calc <int> +|-|*|/ <int>`: evaluate one integer operation.
- `sysinfo`: print era, uptime, and memory information.

The app registry is static metadata compiled into the kernel. It is not a
package manager, dynamic linker, or dynamic app loader.

## Filesystem

- `files`: show a ChronoFS usability overview and safety notes.
- `files list`: list visible files with byte sizes.
- `files info <name>`: print file name, size, storage mode, disk/fallback
  status, and verification notes.
- `files search <term>`: search visible filenames and UTF-8 file contents
  without dumping file bodies.
- `files sample`: print read-only sample file commands; does not create files.
- `files demo`: print a guided walkthrough for list/write/read/info/check/journal;
  does not mutate automatically.
- `files copy <src> <dst>`: copy a file only when the destination does not
  already exist.
- `files rename <old> <new>`: intentionally refuse and explain the conservative
  copy, inspect, then manual `rm` path.
- `fs` / `fs status`: print a read-only ChronoFS mode, disk, file-slot, and
  journal summary.
- `fs info`: print fixed layout limits and journal reservation details.
- `fs check`: run a read-only fsck summary without repairing metadata.
- `fs journal`: print ChronoFS journal status.
- `fs help`: print the ChronoFS inspection command map.
- `fs repair` / `fs check repair`: refuse to mutate and point to `fsck repair`.
- `ls`: list files.
- `cat <name>`: print a file as UTF-8 text.
- `write <name> <content>`: create or overwrite a file.
- `rm <name>`: remove a file.
- `fsck`: check ChronoFS metadata.
- `fsck repair`: perform conservative safe repairs. This prints a warning
  because it mutates ChronoFS metadata and should be used during intentional
  verification.
- `journal`: print ChronoFS journal status.

The `fs` namespace is intentionally inspection-only. `files sample` and
`files demo` are read-only. `files copy` mutates only by writing a new
destination and refuses overwrites. `files rename` is deferred. `fsck repair`
remains the only filesystem repair command.

## Windows And Tasks

- `windows` / `windows list`: list open windows with id, title, task id,
  position, size, and focused marker.
- `windows status`: show count/capacity, drag state, supported window apps, and
  conservative verification boundary.
- `windows focus <id>`: bring a window to the front.
- `windows close <id>`: close a window and terminate its owning cooperative
  task.
- `windows help`: show window command usage.
- `open notes`: open the notes window and spawn its cooperative task.
- `open sysinfo`: open the sysinfo window and spawn its cooperative task.
- `open paint`: report that paint is roadmap/design-only.
- `tasks`: list active cooperative tasks.
- `kill <id>`: terminate a non-running task and close its associated window.

Window mode is a fixed-capacity teaching layer, not a full compositor or GUI
toolkit.

## Userspace And Process Demos

- `userspace` / `userspace status`: summarize the current Ring 3, syscall,
  static ELF, scheduler, and active-ELF boundary.
- `userspace syscalls`: print the tiny syscall ABI table.
- `userspace elf`: explain the supported static ELF64 subset.
- `userspace roadmap`: list future process-model work as roadmap/design-only.
- `userspace help`: print the userspace inspection command map.
- `ring3`: enter the opt-in ring 3 privilege demo. Prints a warning because the
  path is partially implemented and needs runtime verification.
- `syshello`: enter ring 3 and print through `sys_write`. Prints the same
  userspace warning.
- `exec <name>`: load and run a static ELF64 file from ChronoFS. Prints the same
  userspace warning.

The `userspace` namespace is read-only. It does not create processes, run user
programs, or change scheduler behavior.

## Networking

- `net` / `net status`: print RTL8139/static IPv4 status, counters, last
  event, last error, and the verification caveat.
- `net config`: show static IP, QEMU gateway, netmask, default UDP target, and
  current protocol limits.
- `net arp`: explain ARP, show gateway-MAC state, then send an ARP request for
  the QEMU gateway. Prints an ARP/UDP-only runtime-verification warning.
- `net udp`: explain UDP support and send syntax without transmitting.
- `net send`: send the default UDP payload. Prints the same networking warning.
- `net send <ip> <port> <text>`: send a custom UDP payload. Prints the same
  networking warning.
- `net log`: print counters and the last event/error. This is not packet
  capture.
- `net demo`: read-only networking walkthrough.
- `net roadmap`: explain DHCP, DNS, TCP, sockets, and hardware networking as
  future work.
- `net help`: print valid networking subcommands.

Networking is static IPv4 ARP/UDP only. TCP, DHCP, DNS, sockets, and packet
capture remain roadmap/design-only.

## Museum And Quest

- `museum boot|kernel|memory|interrupts|keyboard|serial|era`: print core museum exhibits.
- `museum index`: list core and deeper museum topics.
- `museum disk|filesystem|userspace|syscalls|elf|networking|smp|scheduler`: print deeper OS concept pages.
- `quest list`: print quest progress.
- `quest status`: print active quest/status information.
- `quest dependencies`: print a static dependency-style learning route.
- `quest badges`: print static learning badges derived from quest state.
- `stats`: print player-style project stats.
- `inventory`: print unlocked capability artifacts.
