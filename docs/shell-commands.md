# Shell Command Reference

This reference is based on the current command dispatch in `kernel/src/shell.rs`
plus the built-in app, museum, quest, and networking modules.

Status: implemented in code, needs runtime verification.

Notes storage uses the ChronoFS file named `notes`. The direct `notes` shell
command is handled by `kernel/src/shell.rs`; window mode reads the same file.

## Core Shell

- `help`: list categorized command groups.
- `help start`: explain `start`, `welcome`, `guide`, `demo`, and `tour`.
- `help apps`: explain the app launcher and window paths.
- `help fs`: explain `fs`, `ls`, `cat`, `write`, `rm`, `fsck`, and `journal`.
- `help system`: explain conservative status surfaces.
- `help network`: explain static IPv4 ARP/UDP networking boundaries.
- `help userspace`: explain the `userspace` inspection namespace plus
  `ring3`, `syshello`, and `exec <name>`.
- `help labs`: explain risky/intentional verification commands.
- `help roadmap`: explain roadmap/design-only systems.
- `help files`, `help net`, `help status`, `help verify`, and `help future`: beginner-friendly topic aliases.
- `clear`: clear the framebuffer shell region and redraw the top bar.
- `about`: print the ChronoOS identity line.
- `reboot`: request a reset through the PS/2 controller.
- `uptime`: print elapsed seconds from PIT ticks.
- `clock`: print raw PIT ticks.
- `mem`: print total memory and heap used/free/largest-free values.
- `cores`: print online core count and task assignment counts.
- `beep <hz>`: play a PC speaker tone for 500 ms.

## Help Categories

Top-level `help` groups commands by:

- Getting started
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
- `demo` previews the current surface without changing state.
- `tour` teaches OS concepts by subsystem.
- `doctor` is the conservative status surface; there is no separate `status` or
  `verify` command.
- `apps` is the text launcher; `open` is the partially implemented small-window
  path.
- `fs` is the read-only inspection namespace; `ls`, `cat`, `write`, `rm`,
  `fsck`, and `journal` are the direct filesystem commands; `apps files` points
  users toward them.
- `museum` teaches concepts; `quest` shows progress and next goals.

## Era And Product Commands

- `start`: print the polished first-run welcome screen.
- `welcome`: alias for `start`.
- `guide`: print the guided onboarding topic menu.
- `guide quick|full|eras|apps|systems|status|next`: print focused first-run guide pages that route to existing commands.
- `era 1984|1995|2007|2040`: switch the active era profile.
- `travel <year>`: map a year to an era and switch through the existing `era` path.
- `demo`: print a read-only guided demo.
- `tour`: print the tour overview.
- `tour boot|memory|files|apps|userspace|future`: print a focused educational page.
- `capsule`: print the build-in-public timeline overview.
- `capsule milestones|current|next`: print milestone/current/next timeline views.
- `doctor`: print a conservative read-only subsystem report.
- `poster`: print a screenshot-friendly overview card.
- `poster boot|system|roadmap|eras`: print focused poster screens.

## Apps

- `apps`: print the text app launcher.
- `apps notes|calc|sysinfo|files|clock|museum|theme|tasks`: launch or describe the selected app area.
- `notes`: print the notes home screen.
- `notes read`: read the `notes` ChronoFS file.
- `notes write <text>`: save text to the `notes` ChronoFS file.
- `notes clear`: remove the `notes` ChronoFS file.
- `notes save`: explain that notes save immediately.
- `notes open`: delegate to `open notes`.
- `calc <int> +|-|*|/ <int>`: evaluate one integer operation.
- `sysinfo`: print era, uptime, and memory information.

## Filesystem

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

The `fs` namespace is intentionally inspection-only. `fsck repair` remains the
only filesystem repair command.

## Windows And Tasks

- `open notes`: open the notes window and spawn its cooperative task.
- `open sysinfo`: open the sysinfo window and spawn its cooperative task.
- `tasks`: list active cooperative tasks.
- `kill <id>`: terminate a non-running task and close its associated window.

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

- `net`: print RTL8139/static IPv4 status.
- `net arp`: send an ARP request for the QEMU gateway. Prints an ARP/UDP-only
  runtime-verification warning.
- `net send`: send the default UDP payload. Prints the same networking warning.
- `net send <ip> <port> <text>`: send a custom UDP payload. Prints the same
  networking warning.

Networking is static IPv4 ARP/UDP only. TCP, DHCP, and DNS remain
roadmap/design-only.

## Museum And Quest

- `museum boot|kernel|memory|interrupts|keyboard|serial|era`: print core museum exhibits.
- `museum disk|filesystem|userspace|syscalls|elf|networking|smp|scheduler`: print deeper OS concept pages.
- `quest list`: print quest progress.
- `quest status`: print active quest/status information.
- `stats`: print player-style project stats.
- `inventory`: print unlocked capability artifacts.
