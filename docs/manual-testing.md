# Manual Testing Checklist

Use this file to collect runtime evidence before calling any ChronoOS behavior
runtime-verified. A checkbox means the tester actually observed the behavior in
QEMU or on hardware. Do not check boxes from source inspection alone.

Record the date, command used, host environment, image path, QEMU output, and
any screenshots or serial logs in `docs/AI_PROGRESS_LOG.md`.

Use `docs/CURRENT_STATUS.md` for the current status labels before and after each
test pass. `docs/status-audit.md` remains the Phase 2-specific risk snapshot.

## Shell Workspace Polish Verification

Status: implemented in code, needs runtime verification.

These commands were added as safe text-first shell polish. Do not mark them
verified until a QEMU or hardware pass records exact `cmd:` serial lines and
visible output.

- [ ] `workspace` shows era/theme, reliability mode, verification summary,
      app routes, ChronoFS status, suggested next command, and learning
      suggestion.
- [ ] `status` aliases `workspace`.
- [ ] `verify` prints a read-only verification boundary summary without running
      live tests.
- [ ] `shortcuts` lists the curated demo/useful commands.
- [ ] `whereami` explains current mode, era, UI context, verification boundary,
      and next action.
- [ ] `files` shows the ChronoFS command map and safety notes.
- [ ] `theme` shows the era/theme command card.
- [ ] `recent` shows starter commands when empty and then shows typed commands
      since boot without persistent history or arrow-key recall.
- [ ] `help search fs` finds filesystem/ChronoFS commands.
- [ ] `help search app` finds app/workspace commands.
- [ ] `help workspace`, `help theme`, `help status`, and `help verify` route to
      useful topic help.
- [ ] Near-miss commands such as `hlep`, `apss`, `verfy`, and `lern` suggest
      `help`, `apps`, `verify`, and `learn`.

## 2026-06-13 Visible BIOS Product Evidence

This verification pass used the normal BIOS image in visible single-core QEMU,
serial logging, QEMU monitor input, QEMU monitor `screendump`, and `sips`
conversion. It did not run hardware, UEFI, custom BIOS, SMP/AP, networking
packet behavior, GIF capture, or new feature work.

| Command | Status | Evidence | Notes |
| --- | --- | --- | --- |
| `boot` | verified in QEMU | `/private/tmp/chronoos-visible-bios-20260613-184819.serial.log`; `/private/tmp/chronoos-visible-bios-20260613-184819-boot.png`. | Reached `[CHRONO] boot complete`; visible prompt/top bar observed. |
| `start` | verified in QEMU | Serial `cmd: start`; `/private/tmp/chronoos-visible-bios-20260613-184819-start.png`. | First-run welcome screen observed. |
| `guide quick` | verified in QEMU | Serial `cmd: guide quick`; `/private/tmp/chronoos-visible-bios-20260613-184819-guide-quick.png`. | Quick guide path observed. |
| `mode status` | verified in QEMU | Serial `cmd: mode status`; output visible in `/private/tmp/chronoos-visible-bios-20260613-184819-safe-on.png`. | Evidence is in the later visible frame; the standalone `mode-status` screendump was inconclusive. |
| `safe on` | verified in QEMU | Serial `cmd: safe on`; `/private/tmp/chronoos-visible-bios-20260613-184819-safe-on.png`. | Warning-only safe mode observed; no sandbox claim. |
| `doctor` | verified in QEMU | Serial `cmd: doctor`; `/private/tmp/chronoos-visible-bios-20260613-184819-doctor.png`. | Conservative subsystem report observed. |
| `poster system` | needs manual verification | Logs show `poster s`, `possteerr  ssyyssttem`, and `pposter system`. | QEMU monitor input garbled this command; no exact command evidence. |
| `capsule current` | needs manual verification | Log `/private/tmp/chronoos-visible-bios-20260613-185808.serial.log` shows `ccapsule current`. | QEMU monitor input garbled this command; no exact command evidence. |
| `apps list` | verified in QEMU | Serial `cmd: apps list`; `/private/tmp/chronoos-visible-bios-20260613-185808-apps-list.png`. | Static app registry list observed. |
| screenshot capture | verified in QEMU | QEMU `screendump` PPMs and `sips` PNGs under `/private/tmp/chronoos-visible-bios-20260613-*`. | GIF capture remains unverified. |

## 2026-06-13 Window/Input Evidence

This verification pass used the normal BIOS image in visible single-core QEMU,
serial logging, QEMU monitor input, QEMU monitor `screendump`, QEMU HMP mouse
commands, `sips` conversion, and a fresh disposable data image:
`/private/tmp/chronoos-window-input-20260613-193131.img`. It did not run
hardware, UEFI, custom BIOS, SMP/AP, networking expansion, or source changes.
Manual keyboard typing was blocked by the Codex environment, so Backspace and
Shift remain unverified.

| Test | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Boot | verified in QEMU | `/private/tmp/chronoos-window-input-20260613-193131.serial.log`; `/private/tmp/chronoos-window-input-20260613-193131-boot.png`. | Reached `[CHRONO] boot complete`. |
| Manual typing | blocked by environment | No manual visible-QEMU typing was available. | QEMU monitor `sendkey` evidence is not manual keyboard proof. |
| Backspace | blocked by environment | Not manually typed. | Keep unchecked. |
| Shift | blocked by environment | Not manually typed. | Keep unchecked. |
| `windows status` | verified in QEMU | Serial `cmd: windows status`; `/private/tmp/chronoos-window-input-20260613-193131-windows-status.png`. | Window boundary/status text observed. |
| `open notes` | verified in QEMU | Serial `cmd: open notes`, task spawn, and `wm: open notes`; `/private/tmp/chronoos-window-input-20260613-193131-open-notes.png`. | Notes window observed. |
| `open sysinfo` | verified in QEMU | Serial `cmd: open sysinfo`, task spawn, and `wm: open sysinfo`; `/private/tmp/chronoos-window-input-20260613-193131-current-after-sysinfo-attempt.png`. | Input retry opened two sysinfo windows. |
| `windows list` | partially verified in QEMU | Exact `windows list` garbled as `wwindows list`; exact alias `windows` listed windows in `/private/tmp/chronoos-window-input-20260613-193131-windows-list-exact.png`. | Retest exact command with manual input. |
| `windows focus 1` | verified in QEMU | Serial `cmd: windows focus 1`; `/private/tmp/chronoos-window-input-20260613-193131-mouse-click-notes-attempt.png`. | Notes window was brought to front. |
| Mouse movement/click | partially verified in QEMU | Serial `mouse: click at 70,65`; `/private/tmp/chronoos-window-input-20260613-193131-mouse-click-notes-attempt.png`. | Click packet observed; visible cursor movement not clearly proven. |
| Mouse drag | needs manual verification | No clear before/after movement evidence. | Keep unchecked. |
| Mouse close | needs manual verification | No successful close-button click evidence. | Keep unchecked. |
| `windows close 2` | partially verified in QEMU | Serial `cmd: windows close 2`, `sched: killed task 2`, and `wm: close sysinfo`; `/private/tmp/chronoos-window-input-20260613-193131-windows-close-2.png`. | Serial close/kill path observed, but the screenshot was a breakpoint-like black framebuffer and no follow-up list/tasks output was captured. |
| `tasks` | implemented in code, not verified | Not run after the close attempt. | Needs a fresh pass. |
| `kill <id>` | implemented in code, not verified | Not run after the close attempt. | Use only a real non-shell task ID; never `kill 0`. |

## 2026-06-13 Disposable ChronoFS Evidence

This verification pass used the normal BIOS image in visible single-core QEMU,
serial logging, QEMU monitor input, QEMU monitor `screendump`, `sips`
conversion, and a fresh disposable data image:
`/private/tmp/chronoos-chronofs-20260613-191106.img`. It did not use the repo
data image, run `fsck repair`, test crash recovery, run hardware, or add
features.

| Test | Command | Status | Evidence | Notes |
| --- | --- | --- | --- | --- |
| Fresh disposable mount | boot | verified in QEMU | `/private/tmp/chronoos-chronofs-20260613-191106.serial.log`; `/private/tmp/chronoos-chronofs-20260613-191106-boot.png`. | Fresh image formatted, journal was created, and boot reached `[CHRONO] boot complete`. |
| Status | `fs status` | verified in QEMU | Serial `cmd: fs status`; `/private/tmp/chronoos-chronofs-20260613-191106-fs-status.png`. | Persistent ATA disk and journal summary observed. |
| Layout/listing | `fs info`; `ls` | verified in QEMU | Serial exact commands; `/private/tmp/chronoos-chronofs-20260613-191106-fs-info-ls.png`. | Layout and empty visible listing observed. |
| Write/read | `write verify.txt chrono verification test`; `cat verify.txt` | verified in QEMU | Serial exact commands; `/private/tmp/chronoos-chronofs-20260613-191106-current-before-rm-retry.png`. | Readback printed `chrono verification test`. |
| Check/journal | `fs check`; `fs journal`; `fsck`; `journal` | verified in QEMU | Serial exact commands; `/private/tmp/chronoos-chronofs-20260613-191106-current-before-rm-retry.png`. | Clean read-only check and clean/empty journal state observed. |
| Delete/listing | `rm verify.txt`; `ls` | verified in QEMU | Serial exact commands; `/private/tmp/chronoos-chronofs-20260613-191106-post-delete-ls.png`. | `verify.txt` was absent after deletion. |
| Reboot delete persistence | `ls`; `cat verify.txt`; `fs status`; `journal` | verified in QEMU | Reboot serial `/private/tmp/chronoos-chronofs-20260613-191106-reboot.serial.log`; `/private/tmp/chronoos-chronofs-20260613-191106-reboot-persistence.png`. | Same disposable image remounted clean; `cat verify.txt` reported file not found. |
| Input artifacts | `lls`; `ffs check` | needs manual verification | Main serial log includes these stray commands before exact retries. | QEMU monitor input artifacts; not counted as filesystem behavior. |
| Repair/recovery | `fsck repair`; crash recovery | implemented in code, not verified | Not executed. | Requires controlled corruption/journal images before any status upgrade. |

## 2026-06-13 Userspace Boundary Evidence

This verification pass used the normal BIOS image in visible single-core QEMU,
serial logging, QEMU monitor input, QEMU monitor `screendump`, `sips`
conversion, RTL8139 attached, and fresh disposable 16 MiB data images under
`/private/tmp/chronoos-userspace-20260613-195220-*.img`. It did not run
hardware, mutate the repo data disk, create a new ELF test program, or add
features.

| Test | Command | Status | Evidence | Notes |
| --- | --- | --- | --- | --- |
| Boundary status | `userspace status` | verified in QEMU | Serial `cmd: userspace status`; `/private/tmp/chronoos-userspace-20260613-195220-userspace-status.png`. | Read-only status screen observed. |
| Syscall table | `userspace syscalls` | verified in QEMU | Serial `cmd: userspace syscalls`; `/private/tmp/chronoos-userspace-20260613-195220-userspace-syscalls-clean.png`. | Listed write/read/exit/uptime; this is not runtime syscall execution. |
| ELF boundary screen | `userspace elf` | needs manual verification | Serial attempts logged `uuserspace elf` and `serspace elf`. | No exact command evidence; do not check the manual item yet. |
| Ring 3 demo | `ring3` | verified in QEMU | Serial `cmd: ring3`, `kernel: entered ring 3`, `ring3: transition ok`, and `ring3: privilege violation caught`; `/private/tmp/chronoos-userspace-20260613-195220-ring3.png`. | Fixed teaching demo only, not a general process model. |
| Syscall hello demo | `syshello` | needs manual verification | Serial attempts logged `ssyshello` and `yshello`; diagnostic screenshot `/private/tmp/chronoos-userspace-20260613-195220-syshello.png`. | No exact command evidence; no syscall write/exit proof. |
| Test ELF setup | build/install `hello.elf` | blocked by tooling | `command -v ld.lld` returned no path. | `user/hello.c` and `user/user.ld` exist, but no safe `hello.elf` was installed. |
| Static ELF exec | `exec hello.elf` | blocked by tooling | Not run. | Blocked because the known test ELF was unavailable. |

## 2026-06-13 UEFI Build/Boot Evidence

This verification pass used the UEFI loader, the repo UEFI image builder,
single-core QEMU with OVMF, serial logging, QEMU monitor `screendump`, and
`sips` PNG conversion. It did not run hardware, custom BIOS, SMP/AP,
networking, USB, or unrelated kernel work.

| Step | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Tooling preflight | build fixed, boot not verified | `pwsh`, `qemu-system-x86_64`, `sips`, `nc`, `/opt/homebrew/share/qemu/edk2-x86_64-code.fd`, and `/opt/homebrew/share/qemu/edk2-i386-vars.fd` were present. | This is tooling evidence only. |
| UEFI loader build | build fixed, boot not verified | `cargo build -p uefi-loader --target x86_64-unknown-uefi --offline --locked` passed. | Fixed UEFI crate API drift in the loader. |
| UEFI image build | build fixed, boot not verified | `pwsh -NoLogo -NoProfile -File scripts/build-uefi.ps1` produced `target/x86_64-unknown-none/debug/chronosapien-uefi.img` (64 MiB). | Fixed portable path handling and FAT sizing in the image build path. |
| Single-core UEFI QEMU | partially verified in QEMU UEFI | `/private/tmp/chronoos-uefi-20260613-220234.serial.log`; `/private/tmp/chronoos-uefi-20260613-220234-boot.png`. | OVMF started the ChronoOS UEFI loader, then the loader failed with `Out of Resources` and firmware entered the UEFI shell. |
| Kernel framebuffer/shell | implemented in code, not verified | No `[CHRONO] uefi: framebuffer`, `handoff ok`, or `[CHRONO] boot complete` appeared in the UEFI serial log. | Do not claim UEFI kernel boot or shell verification. |

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

- [x] Build the UEFI image with `.\scripts\build-uefi.ps1`.
- [ ] Boot it with `.\scripts\run-uefi.ps1`.
- [x] Prefer direct single-core QEMU for first verification if the helper script uses `-smp 2`.
- [ ] If the UEFI loader fails to compile, record `blocked: build failure` and do not attempt QEMU boot.
- [x] Confirm the UEFI loader prints/logs loader start.
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
- [ ] `help mode` explains reliability/safe mode.
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

## 5A. Reliability And Safe Mode

- [ ] `mode` and `mode status` show the current mode, command categories,
      warning-only policy, no persistence, and no sandbox claim.
- [ ] `mode safe` sets safe mode and says commands are not blocked.
- [ ] `mode demo` returns to the default portfolio/demo mode.
- [ ] `mode experimental` marks intentional lab/verification mode without
      claiming runtime verification.
- [ ] `safe`, `safe status`, `safe on`, and `safe off` route correctly.
- [ ] `safe off` returns to `mode demo`, not `mode experimental`.
- [ ] Unknown `mode now` and `safe maybe` forms print usage.
- [ ] In safe mode, `ring3`, `syshello`, `exec <name>`, `fsck repair`,
      `net arp`, `net send`, and `reboot` print mode-aware warnings before
      continuing.
- [ ] In experimental mode, the same commands say evidence is still required.
- [ ] Safe mode does not claim security isolation or runtime verification.

## 6. IRQ Keyboard With Polling Fallback

- [ ] Normal typing appears in the shell.
- [ ] Shifted characters work for letters and symbols.
- [ ] Backspace edits the command line.
- [ ] Enter submits commands.
- [ ] Serial does not show keyboard buffer overflow during normal typing.
- [ ] Keyboard still works if IRQ input is delayed and the polling fallback reads a key.

## 7. PS/2 Mouse And Window Interactions

- [x] Mouse initializes according to serial output.
- [ ] Pointer/cursor movement is visible.
- [x] `windows status` reports count/capacity, drag state, and supported window apps.
- [x] `open notes` creates a window.
- [x] `windows list` shows the notes window with an ID and owning task ID.
- [x] `open sysinfo` creates a window.
- [x] `windows focus <id>` brings an existing window to the front.
- [ ] `windows close <id>` closes a window and stops its owning task.
- [ ] `open paint` reports roadmap/design-only instead of pretending paint exists.
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

- [ ] `files` shows the ChronoFS usability overview without mutating files.
- [ ] `files list` lists visible files with byte sizes.
- [ ] `files info <name>` reports file name, size, storage mode, disk/fallback
      status, and verification note.
- [ ] `files search <term>` finds filename matches and UTF-8 content matches
      without dumping full file contents.
- [ ] `files sample` prints read-only sample commands and creates no files.
- [ ] `files demo` prints a guided walkthrough and creates no files.
- [ ] `files copy <src> <dst>` copies only when the destination is absent.
- [ ] `files copy <src> <dst>` refuses to overwrite an existing destination.
- [ ] `files rename <old> <new>` refuses mutation and explains the conservative
      copy, inspect, then manual `rm` path.
- [x] `fs status` prints mode, disk availability, file counts, slots, and journal summary.
- [x] `fs info` prints the fixed layout and limits without mutating metadata.
- [x] `ls` works on a clean disk.
- [x] `write verify.txt chrono verification test` succeeds.
- [x] `cat verify.txt` prints `chrono verification test`.
- [x] `fs check` reports the same read-only status class as `fsck`.
- [x] `rm verify.txt` succeeds.
- [x] `cat verify.txt` reports file not found after removal and reboot.
- [ ] A written file persists after reboot.
- [ ] If ATA is unavailable, heap fallback is clearly reported as non-persistent.

## 10. fsck And fsck repair

- [x] `fsck` on a clean disk reports clean or only expected warnings.
- [x] `fsck` and `fs check` print an explicit `Clean:` line.
- [x] `fs check` groups checked, suspicious, repaired, and not-repaired status.
- [ ] `fsck repair` prints a mutation warning before reporting repair results.
- [ ] `fs repair` and `fs check repair` refuse to mutate and point to `fsck repair`.
- [ ] `fsck repair` on a clean disk does not damage files.
- [ ] A controlled bitmap mismatch is reported by `fsck`.
- [ ] `fsck repair` fixes only safe bitmap/stale-slot issues.
- [ ] `fsck repair` refuses unsafe duplicate-sector or bad-superblock cases.
- [ ] Any repair test records the disk image, serial log, and before/after command output.

## 11. Journal / Recovery Behavior

- [x] `journal` reports available and clean on a normal mounted disk.
- [x] `fs journal` reports the same journal state as `journal`.
- [x] Journal output says clean means no pending record, not full filesystem proof.
- [x] Writing a file leaves the journal clean after completion.
- [x] Removing a file leaves the journal clean after completion.
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

- [ ] `learn map` prints Boot, Memory, Interrupts/Input, Filesystem, Apps/UI,
      Userspace, Networking, Scheduler/SMP, and Roadmap/Future with status,
      verification, suggested command, and related command.
- [ ] `learn progress` states that progress is static/sessionless and points to
      `quest badges`, `quest dependencies`, and the next safe route.
- [ ] `learn beginner` prints a safe read-only first route.
- [ ] `learn advanced` prints a verification-oriented route and warns away from
      risky casual-demo commands.
- [ ] `learn next` recommends `learn map` and does not claim live tracking.
- [ ] `explain kernel` prints a short glossary entry and suggested command.
- [ ] `explain filesystem` prints a short glossary entry and suggested command.
- [ ] `explain syscall` prints a short glossary entry and suggested command.
- [ ] `museum index` lists core and deeper museum topics.
- [ ] `quest dependencies` prints a static dependency route.
- [ ] `quest badges` prints static learning badges derived from quest state.
- [x] `start` prints the first-run welcome screen.
- [ ] `welcome` prints the same first-run welcome screen.
- [ ] `guide` prints the guide topic menu.
- [x] `guide quick` prints the short first-demo path.
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
- [x] `doctor` prints conservative subsystem status.
- [ ] `poster`, `poster boot`, `poster system`, `poster roadmap`, and `poster eras` work.
- [ ] `travel 1987`, `travel 1998`, `travel 2004`, and `travel 2049` map to expected eras.
- [ ] `apps` prints the launcher.
- [ ] `apps featured` lists notes, calc, sysinfo, files, museum/learn, theme,
      and status/verify surfaces.
- [ ] `apps recent` starts empty or with in-memory app launcher routes from the
      current boot only.
- [ ] `apps category Core`, `apps category Files`, `apps category Learning`,
      `apps category System`, `apps category Networking`, `apps category Visual`,
      `apps category Debug/Lab`, and `apps category Roadmap/Future` filter the
      static registry.
- [ ] `apps info notes` shows name, category, description, launch command,
      status, verification status, risk level, and related commands.
- [ ] `apps help notes` shows app-specific help and related commands.
- [ ] `apps demo notes` prints a safe demo path without executing commands.
- [ ] `apps notes`, `apps calc`, `apps sysinfo`, `apps files`, `apps clock`, `apps museum`, `apps theme`, and `apps tasks` behave as documented.
- [ ] Product commands use conservative status labels and do not claim runtime verification.
- [ ] `doctor` and `poster system` report mouse/window/network/userspace limits truthfully.

## 14. Ring 3 Demo

- [x] `userspace status` reports Ring 3/syscall/ELF boundaries without running demos.
- [ ] `userspace help` lists the read-only userspace inspection commands.
- [ ] `ring3` prints the userspace runtime-verification warning before running.
- [x] `ring3` enters the user-mode demo.
- [x] The privileged instruction fault is caught as expected.
- [x] The kernel logs the ring 3 transition and violation.
- [ ] The system remains inspectable after the demo.
- [ ] The result is documented as a teaching demo, not a general userland.

## 15. Syscalls

- [x] `userspace syscalls` lists syscall numbers 1-4: write, read, exit, uptime.
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
- [ ] Build and install `hello.elf` into a disposable ChronoFS image; do not mutate the repo data disk for verification.
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
- [ ] `net` and `net status` print NIC state, MAC, static IP, gateway state,
      TX/RX counts, ARP/UDP counters, malformed RX, last event, and last error.
- [ ] `net config` prints static IP, QEMU gateway, netmask, default UDP target,
      and says there is no DHCP/DNS/TCP/socket support.
- [ ] `net log` prints counters and last event/error without claiming packet
      capture.
- [ ] `net demo` is read-only and does not transmit packets.
- [ ] `net roadmap` keeps DHCP, DNS, TCP, sockets, packet capture, and broader
      hardware networking as roadmap/design-only.
- [ ] `net udp` explains UDP send syntax without transmitting.
- [ ] `net arp` explains ARP and prints the ARP/UDP-only runtime-verification warning.
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
- [ ] `apps list` displays the same static registry surface.
- [ ] `apps featured` displays only curated demo-friendly entries.
- [ ] `apps recent` displays only app launcher routes from this boot.
- [ ] `apps category <name>` filters the static registry without launching apps.
- [ ] `apps info notes` shows name, category, launch command, status, verification, and risk.
- [ ] `apps help notes` shows help text and related commands.
- [ ] `apps demo notes` prints commands without running them.
- [ ] `apps launch calc` delegates to an existing shell command instead of dynamic loading.
- [ ] `apps verified` lists only entries with recorded partial QEMU evidence.
- [ ] `apps roadmap` lists roadmap/design-only app ideas without launching them.
- [ ] `apps notes`, `apps calc`, and `apps sysinfo` route to existing app behavior.
- [ ] `apps files`, `apps clock`, `apps museum`, `apps theme`, and `apps tasks` describe or route to existing shell areas.
- [ ] Roadmap entries such as `paint`, `network`, and `crashlab` do not pretend to be installed runtime apps.
- [ ] `windows`, `windows list`, `windows status`, `windows focus <id>`, `windows close <id>`, and `windows help` behave as documented.
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
- [ ] Confirm `network demo mode` is documented as roadmap/design-only, while
      current networking is limited to static IPv4 ARP/UDP commands such as
      `net status`, `net config`, `net arp`, `net udp`, `net send`, `net log`,
      `net demo`, and `net roadmap`.
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
