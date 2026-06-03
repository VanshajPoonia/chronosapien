# Manual Testing Checklist

Use this file to collect runtime evidence before calling any ChronoOS behavior
runtime-verified. A checkbox means the tester actually observed the behavior in
QEMU or on hardware. Do not check boxes from source inspection alone.

Record the date, command used, host environment, image path, QEMU output, and
any screenshots or serial logs in `docs/AI_PROGRESS_LOG.md`.

Use `docs/CURRENT_STATUS.md` for the current status labels before and after each
test pass. `docs/status-audit.md` remains the Phase 2-specific risk snapshot.

## Tooling Preflight

- [ ] Confirm `rustc -vV` uses the intended pinned nightly toolchain.
- [ ] Confirm `cargo check -p kernel --offline --locked` passes before QEMU tests.
- [ ] Confirm the host-side package check passes with the host target, for example `cargo check -p chronosapien --target <host> --offline --locked`.
- [ ] Confirm `pwsh` is available before using PowerShell helper scripts.
- [ ] Confirm `qemu-system-x86_64` is available before runtime verification.
- [ ] Confirm OVMF exists before UEFI QEMU verification, for example `/opt/homebrew/share/qemu/edk2-x86_64-code.fd`.
- [ ] Confirm `nasm` is available before custom BIOS verification.
- [ ] Confirm a UDP helper such as `nc` is available before ARP/UDP verification.
- [ ] If `pwsh` or QEMU is unavailable, record runtime verification as blocked instead of checking runtime boxes.
- [ ] If using `-display none`, record it as serial-only evidence and do not check framebuffer or visible shell-prompt boxes.

## 0. Build Sanity

Build sanity is not runtime verification, but it gates every QEMU/hardware pass.

- [ ] Run the intended build command for the image under test.
- [ ] Record the exact command, toolchain, and result in `docs/AI_PROGRESS_LOG.md`.
- [ ] If the build fails, record the failing files/symbols and stop before making runtime claims.
- [ ] If the build succeeds, keep all runtime-facing systems marked `needs runtime verification` until QEMU or hardware evidence exists.

## 1. BIOS Boot

- [ ] Build the normal BIOS image with `.\scripts\build.ps1`.
- [ ] Boot it with `.\scripts\run.ps1`.
- [ ] For serial-only smoke, confirm `[CHRONO] boot start`.
- [ ] For serial-only smoke, confirm `[CHRONO] boot complete`.
- [ ] Confirm the framebuffer appears.
- [ ] Confirm the shell prompt appears.
- [ ] Confirm single-core and multi-core boot outcomes are recorded separately.

## 2. UEFI Boot

- [ ] Build the UEFI image with `.\scripts\build-uefi.ps1`.
- [ ] Boot it with `.\scripts\run-uefi.ps1`.
- [ ] Prefer direct single-core QEMU for first verification if the helper script uses `-smp 2`.
- [ ] If the UEFI loader fails to compile, record `blocked: build failure` and do not attempt QEMU boot.
- [ ] Confirm the UEFI loader prints/logs loader start.
- [ ] Confirm GOP framebuffer output reaches the kernel.
- [ ] Confirm the ChronoOS shell prompt appears.
- [ ] Confirm the UEFI path does not require Secure Boot.

## 3. Custom BIOS Boot Path

- [ ] Build the custom BIOS image with `.\scripts\build-custom.ps1`.
- [ ] Confirm `nasm` is available before building this path.
- [ ] If `nasm` is missing, record `blocked: build dependency missing` and stop.
- [ ] Boot it with `.\scripts\run-custom.ps1`.
- [ ] Confirm Stage 1 serial output appears.
- [ ] Confirm the custom handoff reaches `chrono_custom_entry`.
- [ ] Confirm the normal shell prompt appears after handoff.

## 4. Framebuffer And Serial Output

- [ ] Confirm text renders in the framebuffer.
- [ ] Confirm the top bar renders and updates.
- [ ] Confirm serial logs are visible with `-serial stdio`.
- [ ] Confirm `clear` redraws the shell area.
- [ ] Confirm panic/fault text is inspectable if a controlled fault test is run.

## 5. Shell Commands

- [ ] `help` lists the expected command groups.
- [ ] `help start` explains `start`, `welcome`, `guide`, `demo`, and `tour`.
- [ ] `help apps` explains `apps` versus `open`.
- [ ] `help fs` explains file commands and warns about `fsck repair`.
- [ ] `help system` points to `doctor`, `poster system`, and conservative status surfaces.
- [ ] `help network` says networking is static IPv4 ARP/UDP only.
- [ ] `help userspace` says userspace demos are partial and need runtime verification.
- [ ] `help labs` groups risky/intentional verification commands.
- [ ] `help roadmap` marks long-term systems as roadmap/design-only.
- [ ] `help files`, `help net`, `help status`, `help verify`, and `help future` route to useful topic help.
- [ ] `about` prints the current ChronoOS identity line.
- [ ] `uptime` increases over time.
- [ ] `clock` prints raw PIT ticks.
- [ ] `mem` prints heap used/free/largest-free values.
- [ ] `cores` prints online core/task counts.
- [ ] `beep 440` plays or logs a tone without hanging.
- [ ] Invalid commands return `unknown command` plus `help` guidance.
- [ ] Common confusions such as `status`, `verify`, `files`, `apps now`, and `net now` print helpful hints.
- [ ] Unknown help topics such as `help coffee` print the valid topic list.

## 6. IRQ Keyboard With Polling Fallback

- [ ] Normal typing appears in the shell.
- [ ] Shifted characters work for letters and symbols.
- [ ] Backspace edits the command line.
- [ ] Enter submits commands.
- [ ] Serial does not show keyboard buffer overflow during normal typing.
- [ ] Keyboard still works if IRQ input is delayed and the polling fallback reads a key.

## 7. PS/2 Mouse And Window Interactions

- [ ] Mouse initializes according to serial output.
- [ ] Pointer/cursor movement is visible.
- [ ] `open notes` creates a window.
- [ ] `open sysinfo` creates a window.
- [ ] Clicking a title bar focuses a window.
- [ ] Dragging a title bar moves a window.
- [ ] Clicking close removes the window and associated task.
- [ ] Opening the same window repeatedly respects the fixed-capacity window limit.
- [ ] `tasks` reflects task creation after `open notes` / `open sysinfo`.
- [ ] `tasks` reflects task removal after window close or `kill <id>`.
- [ ] Window behavior is documented as partially implemented, not a full compositor.

## 8. Heap Allocator Reuse

- [ ] `mem` shows initial free and largest-free values.
- [ ] Repeated `open notes` / close cycles do not monotonically exhaust heap.
- [ ] Repeated file writes/removes do not permanently consume all heap.
- [ ] Repeated app/task creation and kill paths leave reusable heap space.
- [ ] No allocator-related panic occurs during the above.

## 9. ChronoFS Read / Write / Delete

- [ ] `fs status` prints mode, disk availability, file counts, slots, and journal summary.
- [ ] `fs info` prints the fixed layout and limits without mutating metadata.
- [ ] `ls` works on a clean disk.
- [ ] `write hello.txt Hi there` succeeds.
- [ ] `cat hello.txt` prints `Hi there`.
- [ ] `fs check` reports the same read-only status class as `fsck`.
- [ ] `rm hello.txt` succeeds.
- [ ] `cat hello.txt` reports file not found after removal.
- [ ] A written file persists after reboot.
- [ ] If ATA is unavailable, heap fallback is clearly reported as non-persistent.

## 10. fsck And fsck repair

- [ ] `fsck` on a clean disk reports clean or only expected warnings.
- [ ] `fs check` groups checked, suspicious, repaired, and not-repaired status.
- [ ] `fsck repair` prints a mutation warning before reporting repair results.
- [ ] `fs repair` and `fs check repair` refuse to mutate and point to `fsck repair`.
- [ ] `fsck repair` on a clean disk does not damage files.
- [ ] A controlled bitmap mismatch is reported by `fsck`.
- [ ] `fsck repair` fixes only safe bitmap/stale-slot issues.
- [ ] `fsck repair` refuses unsafe duplicate-sector or bad-superblock cases.
- [ ] Any repair test records the disk image, serial log, and before/after command output.

## 11. Journal / Recovery Behavior

- [ ] `journal` reports available and clean on a normal mounted disk.
- [ ] `fs journal` reports the same journal state as `journal`.
- [ ] Journal output says clean means no pending record, not full filesystem proof.
- [ ] Writing a file leaves the journal clean after completion.
- [ ] Removing a file leaves the journal clean after completion.
- [ ] A controlled intent-state journal record rolls back safely on mount.
- [ ] A controlled committed-state journal record rolls forward safely on mount.
- [ ] A corrupt journal record is refused and reported without guessing.
- [ ] Recovery refuses ambiguous duplicate-sector or bad-superblock cases.
- [ ] Recovery rebuilds the bitmap only when file-table metadata is trusted.
- [ ] Any crash/recovery test records the before/after disk image and serial log.

## 12. Apps

- [ ] `notes` prints the notes home screen.
- [ ] `notes write hello` saves to the `notes` file.
- [ ] `notes read` prints the saved note.
- [ ] `notes clear` removes the note.
- [ ] `notes open` opens the notes window.
- [ ] `calc 6 * 7` prints `42`.
- [ ] `calc 1 / 0` reports divide by zero.
- [ ] `sysinfo` prints era, uptime, and memory data.

## 13. Product Commands

- [ ] `start` prints the first-run welcome screen.
- [ ] `welcome` prints the same first-run welcome screen.
- [ ] `guide` prints the guide topic menu.
- [ ] `guide quick` prints the short first-demo path.
- [ ] `guide full` prints the full demo route without executing risky commands.
- [ ] `guide eras` points to era/travel/poster era commands.
- [ ] `guide apps` points to the app launcher, notes, calc, sysinfo, and window paths.
- [ ] `guide systems` points to museum and tour commands.
- [ ] `guide status` points to `doctor`, `poster system`, `capsule current`, `quest status`, `fsck`, and `journal`.
- [ ] `guide next` separates safe demo commands from intentional verification commands.
- [ ] `guide nope` prints usage and does not fall through to `unknown command`.
- [ ] `demo` prints the read-only demo guide.
- [ ] `tour` prints the tour overview.
- [ ] `tour boot`, `tour memory`, `tour files`, `tour apps`, `tour userspace`, and `tour future` work.
- [ ] `capsule`, `capsule milestones`, `capsule current`, and `capsule next` work.
- [ ] `doctor` prints conservative subsystem status.
- [ ] `poster`, `poster boot`, `poster system`, `poster roadmap`, and `poster eras` work.
- [ ] `travel 1987`, `travel 1998`, `travel 2004`, and `travel 2049` map to expected eras.
- [ ] `apps` prints the launcher.
- [ ] `apps notes`, `apps calc`, `apps sysinfo`, `apps files`, `apps clock`, `apps museum`, `apps theme`, and `apps tasks` behave as documented.
- [ ] Product commands use conservative status labels and do not claim runtime verification.
- [ ] `doctor` and `poster system` report mouse/window/network/userspace limits truthfully.

## 14. Ring 3 Demo

- [ ] `userspace status` reports Ring 3/syscall/ELF boundaries without running demos.
- [ ] `userspace help` lists the read-only userspace inspection commands.
- [ ] `ring3` prints the userspace runtime-verification warning before running.
- [ ] `ring3` enters the user-mode demo.
- [ ] The privileged instruction fault is caught as expected.
- [ ] The kernel logs the ring 3 transition and violation.
- [ ] The system remains inspectable after the demo.
- [ ] The result is documented as a teaching demo, not a general userland.

## 15. Syscalls

- [ ] `userspace syscalls` lists syscall numbers 1-4: write, read, exit, uptime.
- [ ] `syshello` prints the userspace runtime-verification warning before running.
- [ ] `syshello` enters ring 3.
- [ ] `sys_write` prints hello text through the kernel dispatcher.
- [ ] `sys_exit` reports or logs exit behavior.
- [ ] Invalid syscall behavior is handled conservatively if tested.
- [ ] Syscall tests use only the documented tiny ABI: write, read, exit, uptime.
- [ ] Results are not described as a mature process model.

## 16. Static ELF Exec

- [ ] `userspace elf` explains the static ELF64 boundary without loading a program.
- [ ] `userspace roadmap` marks argv/env, process table, dynamic linker, package manager, and preemptive scheduler as future work.
- [ ] Build and install `hello.elf` with `.\scripts\build-user.ps1`.
- [ ] `ls` shows `hello.elf`.
- [ ] `exec hello.elf` prints the userspace runtime-verification warning before loading.
- [ ] `exec hello.elf` prints the user-space hello text.
- [ ] The process exits back to the shell with an exit code.
- [ ] Invalid or missing ELF files report a clean error.
- [ ] The test does not imply support for dynamic linking, argv/env, libc, or packages.

## 17. ARP / UDP Networking

- [ ] Boot with the RTL8139 QEMU device enabled.
- [ ] If a host UDP listener conflicts with `hostfwd=udp::9000-:9000`, record the conflict and test guest-to-host send without claiming host-to-guest forwarding.
- [ ] Confirm serial logs `net: rtl8139 found`.
- [ ] Confirm serial logs the expected guest MAC.
- [ ] `net` prints MAC, static IP, gateway state, and TX/RX counts.
- [ ] `net arp` prints the ARP/UDP-only runtime-verification warning.
- [ ] `net arp` sends an ARP request.
- [ ] Gateway MAC becomes learned after an ARP reply.
- [ ] `net send` prints the ARP/UDP-only runtime-verification warning.
- [ ] `net send` sends the default UDP payload.
- [ ] `net send <ip> <port> <text>` prints the ARP/UDP-only runtime-verification warning.
- [ ] `net send <ip> <port> <text>` sends a custom UDP payload.
- [ ] Incoming UDP packets are logged to serial when sent through QEMU forwarding.
- [ ] Results are described as static IPv4 ARP/UDP only.
- [ ] No TCP, DHCP, DNS, socket, or broad hardware support is implied.
- [ ] If QEMU monitor key injection garbles `net` commands, record networking as partially verified at most.

## 18. Cooperative Scheduler And Task Lifecycle

- [ ] `tasks` works at a fresh shell prompt.
- [ ] `open notes` spawns a cooperative task.
- [ ] `open sysinfo` spawns a cooperative task.
- [ ] `kill <id>` removes the selected task without hanging the shell.
- [ ] Closing a window removes or stops the associated task.
- [ ] `cores` prints online core/task counts without claiming AP startup success unless separately verified.
- [ ] Scheduler behavior is documented as cooperative and partially implemented.

## 19. SMP / AP Startup

- [ ] Boot a single-core QEMU config and confirm the shell still works.
- [ ] Boot a multi-core QEMU config intentionally, such as `-smp 2`.
- [ ] Serial output records ACPI MADT discovery or a clear fallback path.
- [ ] Serial output records whether AP startup succeeded, failed, or was skipped.
- [ ] Mark SMP/AP verified only if serial shows AP-online evidence such as `smp: core 1 online` and `smp: 2 cores ready`.
- [ ] If the run reaches only `smp: BSP online (core 0)`, keep SMP/AP partially implemented and high-risk.
- [ ] `cores` reports a result consistent with the serial log.
- [ ] Scheduler task placement does not imply SMP success unless AP startup evidence exists.
- [ ] SMP remains marked risky until repeated QEMU evidence is recorded.

## 19a. Hardware Safety

- [ ] Read `docs/hardware-testing.md` before writing any image to removable media.
- [ ] Confirm Secure Boot is disabled or intentionally handled before UEFI boot.
- [ ] Confirm the target machine has a safe serial logging path or a clear fallback observation method.
- [ ] Confirm USB HID/storage/serial are not expected to work unless future drivers are implemented.
- [ ] Confirm storage writes are isolated to a sacrificial test disk or removable media.
- [ ] Record hardware status as `needs manual verification` until real evidence exists.

## 20. Window Manager And App Platform Boundaries

- [ ] `apps` displays launcher entries without requiring graphics-only interaction.
- [ ] `apps notes`, `apps calc`, and `apps sysinfo` route to existing app behavior.
- [ ] `apps files`, `apps clock`, `apps museum`, `apps theme`, and `apps tasks` describe or route to existing shell areas.
- [ ] `open notes` and `open sysinfo` are the only documented window app paths.
- [ ] Window close, drag, and focus are checked with both mouse movement and shell task state.
- [ ] Results are described as a small teaching app/window platform, not a full GUI toolkit.

## 21. Phase 3 Product Idea Boundaries

- [ ] Confirm `demo`, `tour`, `capsule`, `doctor`, `poster`, `travel <year>`, and `apps` are documented as implemented in code.
- [ ] Confirm `theme studio` is documented as roadmap/design-only, with `apps theme` treated as a text preview only.
- [ ] Confirm `crash lab` is documented as roadmap/design-only.
- [ ] Confirm `tiny paint` is documented as roadmap/design-only.
- [ ] Confirm `file explorer window mode` is documented as roadmap/design-only, with `apps files` treated as a text card for file commands only.
- [ ] Confirm `boot chime selector` is documented as roadmap/design-only, even though era tones exist.
- [ ] Confirm `network demo mode` is documented as roadmap/design-only, with current networking limited to `net`, `net arp`, and `net send`.
- [ ] Confirm `user-space showcase` is documented as partially implemented through `ring3`, `syshello`, `exec`, and guide pages.
- [ ] Confirm `visual boot timeline` is documented as partially implemented through text-only `capsule` and `poster boot`.
- [ ] Confirm the mini desktop/app launcher is documented as partially implemented, not a full desktop.

## 22. Phase 4 Roadmap Boundaries

- [ ] Networking Track A: ARP/UDP is tested before any DHCP, DNS, TCP, or socket claims are added.
- [ ] USB/Hardware Track B: BIOS/UEFI/runtime evidence is recorded before any USB HID/storage/serial claims are added.
- [ ] Scheduler/User Mode Track C: cooperative tasks, `ring3`, `syshello`, and `exec` are tested before any preemptive scheduler or broad userland claims are added.
- [ ] App Loading Track D: built-in apps and static ELF are tested before any dynamic linker or package manager claims are added.
- [ ] GUI Track E: framebuffer, pointer, drag, close, and window app paths are tested before any full compositor or GUI toolkit claims are added.
- [ ] `docs/CURRENT_STATUS.md`, `README.md`, `docs/roadmap.md`, and `docs/showcase.md` agree on which items are roadmap/design-only.
- [ ] Any future upgrade to `verified in QEMU` or `verified on hardware` points to concrete evidence in `docs/AI_PROGRESS_LOG.md`.
