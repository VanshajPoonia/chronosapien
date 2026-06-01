# Shell Command Reference

This reference is based on the current command dispatch in `kernel/src/shell.rs`
plus the built-in app, museum, quest, and networking modules.

Status: implemented in code, needs runtime verification.

Notes storage uses the ChronoFS file named `notes`. The direct `notes` shell
command is handled by `kernel/src/shell.rs`; window mode reads the same file.

## Core Shell

- `help`: list command groups.
- `clear`: clear the framebuffer shell region and redraw the top bar.
- `about`: print the ChronoOS identity line.
- `reboot`: request a reset through the PS/2 controller.
- `uptime`: print elapsed seconds from PIT ticks.
- `clock`: print raw PIT ticks.
- `mem`: print total memory and heap used/free/largest-free values.
- `cores`: print online core count and task assignment counts.
- `beep <hz>`: play a PC speaker tone for 500 ms.

## Era And Product Commands

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

- `ls`: list files.
- `cat <name>`: print a file as UTF-8 text.
- `write <name> <content>`: create or overwrite a file.
- `rm <name>`: remove a file.
- `fsck`: check ChronoFS metadata.
- `fsck repair`: perform conservative safe repairs.
- `journal`: print ChronoFS journal status.

## Windows And Tasks

- `open notes`: open the notes window and spawn its cooperative task.
- `open sysinfo`: open the sysinfo window and spawn its cooperative task.
- `tasks`: list active cooperative tasks.
- `kill <id>`: terminate a non-running task and close its associated window.

## Userspace And Process Demos

- `ring3`: enter the opt-in ring 3 privilege demo.
- `syshello`: enter ring 3 and print through `sys_write`.
- `exec <name>`: load and run a static ELF64 file from ChronoFS.

## Networking

- `net`: print RTL8139/static IPv4 status.
- `net arp`: send an ARP request for the QEMU gateway.
- `net send`: send the default UDP payload.
- `net send <ip> <port> <text>`: send a custom UDP payload.

## Museum And Quest

- `museum boot|kernel|memory|interrupts|keyboard|serial|era`: print core museum exhibits.
- `museum disk|filesystem|userspace|syscalls|elf|networking|smp|scheduler`: print deeper OS concept pages.
- `quest list`: print quest progress.
- `quest status`: print active quest/status information.
- `stats`: print player-style project stats.
- `inventory`: print unlocked capability artifacts.
