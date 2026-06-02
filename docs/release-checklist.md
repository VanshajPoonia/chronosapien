# ChronoOS Release Checklist

Use this checklist before publishing a demo build, portfolio update, README
refresh, or build-in-public post. A release can be documentation-only, a QEMU
demo release, or a hardware experiment, but each claim must match recorded
evidence.

For the v0.1 release candidate, start with `docs/RELEASE_v0.1.md`. It contains
the v0.1 release story, status summary, and compact release checklist.

## 2026-06-02 High-Risk Verification Pass

This pass targeted UEFI, custom BIOS, SMP/AP, networking, and hardware only.
No source features or low-level architecture were changed.

| Area | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Tooling/build preflight | verified in QEMU | PowerShell, QEMU, Rustup, OVMF, and `nc` were available; both cargo checks passed. | `nasm` was not available. |
| UEFI build | blocked: build failure | `scripts/build-uefi.ps1` downloaded dependencies after escalation, then `uefi-loader` failed on `uefi::boot::MemoryMap`. | No UEFI QEMU boot was attempted. |
| UEFI QEMU boot | implemented in code, not verified | OVMF exists at `/opt/homebrew/share/qemu/edk2-x86_64-code.fd`. | Blocked behind the UEFI loader compile failure. |
| Custom BIOS | blocked: build dependency missing | `command -v nasm` returned no path. | `scripts/build-custom.ps1` and `scripts/run-custom.ps1` were intentionally not run. |
| SMP/AP | partially verified in QEMU | `/private/tmp/chronoos-smp-20260602-162000.serial.log` reached `smp: BSP online (core 0)` and `active era: 1984`. | No AP-online, `smp: 2 cores ready`, or boot-complete evidence; keep high-risk. |
| Networking device init | partially verified in QEMU | `/private/tmp/chronoos-net-20260602-162000.serial.log` reached boot complete and logged RTL8139 discovery plus MAC `52:54:00:12:34:56`. | Device init only; not ARP/UDP behavior. |
| ARP/UDP behavior | implemented in code, not verified | `hostfwd=udp::9000-:9000` conflicted with the host UDP listener; later input attempts submitted `n7et` and `neett`. | Host UDP log was 0 bytes. |
| DHCP/DNS/TCP | roadmap/design-only | Not implemented or tested. | Do not include in release claims. |
| Hardware | needs manual verification | No hardware run or image write was performed. | See `docs/hardware-testing.md`. |

## 2026-06-02 UI/Input QEMU Verification Pass

This pass reused the normal BIOS image in single-core QEMU with visible display,
serial logging, QEMU monitor input, RTL8139 attached, and the existing ChronoFS
data disk. Evidence is recorded in `docs/AI_PROGRESS_LOG.md`.

| Area | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Tooling/build | verified in QEMU | PowerShell, QEMU, `nc`, and `sips` were available; both cargo checks and `scripts/build.ps1` passed. | `ffmpeg`, ImageMagick `magick`, and `gifsicle` were unavailable for GIF creation. |
| Framebuffer shell | verified in QEMU | `/private/tmp/chronoos-ui-input-20260602-150049-boot.png` showed the ChronoOS top bar, prompt, and cursor. | This extends the earlier visible boot evidence. |
| Keyboard Enter/basic input | partially verified in QEMU | Serial log includes `cmd: apps`, `cmd: notes`, `cmd: calc 6 - 7`, and `cmd: open notes`. | QEMU monitor input worked for command submission but was not fully reliable. |
| Keyboard Backspace/Shift/polling fallback | needs manual verification | Backspace attempt produced `abouut`; Shift and polling fallback were not proven. | Do not claim broad keyboard behavior yet. |
| Apps launcher | verified in QEMU | `/private/tmp/chronoos-ui-input-20260602-150049-apps.png` shows the `apps` launcher. | Text launcher only; not a full desktop. |
| Notes shell app | verified in QEMU | `/private/tmp/chronoos-ui-input-20260602-150049-notes-attempt.png` shows the notes home screen. | Notes read/write and persistence still need verification. |
| Calc shell app | verified in QEMU | Serial log includes `cmd: calc 6 - 7` and `app: calc launched`; screenshot shows result `-1`. | Narrow arithmetic path only. |
| Sysinfo shell app | implemented in code, not verified | Attempt was submitted as `ssysinfo`, so no valid `sysinfo` command was observed. | Retest with a more reliable input path. |
| Open notes window | partially verified in QEMU | Serial log includes `cmd: open notes`, task spawn, and `wm: open notes`; screenshot shows a visible window boundary. | Window content, focus, drag, and close still need verification. |
| Open sysinfo window | implemented in code, not verified | `open sysinfo` was not submitted successfully. | Retest after notes/window baseline. |
| Mouse click | partially verified in QEMU | Serial log includes `[CHRONO] mouse: click at 740,410`. | Click packet observed; target effect not proven. |
| Mouse movement/drag/close | needs manual verification | Mouse move and drag screenshots were captured, but movement/drag/close behavior was not clearly observed. | Do not mark window interaction verified yet. |
| Still screenshots | verified in QEMU | QEMU monitor `screendump` plus `sips` produced PNG evidence under `/private/tmp/chronoos-ui-input-20260602-150049-*.png`. | Keep screenshot claims tied to exact files. |
| GIF capture | needs manual verification | No GIF encoder was available on PATH. | Manual capture/tooling is documented in `docs/screenshots.md`. |

## 2026-06-02 Core QEMU Verification Pass

This pass used the normal BIOS image in single-core QEMU with visible display,
serial logging, RTL8139 attached, and the existing ChronoFS data disk. Evidence
is recorded in `docs/AI_PROGRESS_LOG.md`.

| Area | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Tooling | verified in QEMU | PowerShell 7.6.2 and QEMU 11.0.1 available locally. | Tooling availability was checked before boot. |
| Build status | verified in QEMU | `cargo check -p kernel --offline --locked`, host package check, and `scripts/build.ps1` passed. | This is build evidence, not runtime evidence for every subsystem. |
| BIOS boot | verified in QEMU | Serial log `/private/tmp/chronoos-qemu-20260602-013807.serial.log` reached `[CHRONO] boot complete`. | Single-core only; SMP/AP still needs separate verification. |
| Serial output | verified in QEMU | Serial log includes boot start, framebuffer init, filesystem mount, timer, mouse, RTL8139, keyboard init, and boot complete. | Shell-command serial logging is only partially observed. |
| Visible framebuffer shell | verified in QEMU | QEMU screendump `/private/tmp/chronoos-qemu-20260602-013807-screendump.png` showed top bar, boot text, and `CHRONO>` prompt. | Broader redraw/window/app behavior still needs checks. |
| Screenshot capture | verified in QEMU | QEMU monitor `screendump` captured framebuffer PNGs. | macOS `screencapture` failed with `could not create image from display`; GIF capture still needs manual verification. |
| Shell startup | verified in QEMU | Prompt was visible in QEMU screendumps after boot. | This does not verify every command. |
| Keyboard input | verified in QEMU | QEMU monitor `sendkey` submitted `help` and `help start`. | Manual typing, Backspace, Shift, and polling fallback still need verification. |
| Basic commands | verified in QEMU | `help`, `help start`, and `about` output were visible in framebuffer screendumps. | Longer command batch injection became unreliable. |
| Product/status commands | implemented in code, not verified | `demo`, `tour`, `doctor`, `capsule`, and `poster` were not cleanly observed in this pass. | Test one command at a time in the next pass. |
| Top-level `status`, `verify`, `timeline` | roadmap/design-only | No top-level commands were verified in this pass. | Current status/verification language is routed through `doctor`, `help system`, `capsule current`, and related surfaces. |
| Apps | implemented in code, not verified | `apps`, `notes`, `calc`, and `sysinfo` were not cleanly observed in this pass. | Keep release claims as code-present until tested. |
| ChronoFS shell workflows | implemented in code, not verified | `ls`, `write`, `cat`, `rm`, `fsck`, `fsck repair`, and `journal` were not exercised. | Do not claim file workflow verification yet. |
| Userspace/syscalls/ELF | implemented in code, not verified | `ring3`, `syshello`, and `exec` were not run. | Keep as intentional verification paths. |
| Networking behavior | implemented in code, not verified | Serial log showed RTL8139 initialization; `net`, ARP, and UDP send were not tested. | Static IPv4 ARP/UDP only; no TCP/DHCP/DNS. |
| Mouse/window interaction | implemented in code, not verified | Serial log showed mouse initialization; movement and windows were not tested. | Requires visual interaction pass. |
| UEFI/custom BIOS | implemented in code, not verified | Not tested in this pass. | Separate boot-path verification required. |
| SMP/AP | implemented in code, not verified | Not tested in this pass. | Keep high-risk after earlier two-core boot did not reach boot complete. |
| Hardware | needs manual verification | No hardware run was performed. | Do not claim hardware support from QEMU evidence. |

## Release Type

- [ ] Documentation-only release.
- [ ] QEMU demo release.
- [ ] Hardware experiment.
- [ ] Internal stabilization checkpoint.

## Build Check

- [ ] `cargo check -p kernel --offline --locked`
- [ ] Host package check, for example `cargo check -p chronosapien --target <host> --offline --locked`
- [ ] BIOS image build with `scripts/build.ps1`, if releasing a bootable image.
- [ ] UEFI/custom BIOS builds only if those paths are part of the release.
- [ ] Record command, toolchain, host, and result in `docs/AI_PROGRESS_LOG.md`.

## QEMU Boot Check

- [ ] Normal BIOS image boots in QEMU.
- [ ] Serial output includes `[CHRONO] boot start`.
- [ ] Serial output includes `[CHRONO] boot complete`.
- [ ] Visible framebuffer output appears.
- [ ] First shell prompt appears.
- [ ] Single-core and multi-core outcomes are recorded separately.
- [ ] If using `-display none`, mark the result as serial-only and do not claim
      framebuffer or visible shell verification.

## Serial Output Check

- [ ] Serial boot log is captured.
- [ ] Serial log includes enough startup lines to identify the path tested.
- [ ] Serial output from shell commands is checked separately if claimed.
- [ ] Serial log is referenced from the progress log or release notes.

## Keyboard Input Check

- [ ] Normal typing works at the shell.
- [ ] Backspace works.
- [ ] Enter submits commands.
- [ ] Shifted characters work.
- [ ] IRQ keyboard path and polling fallback are documented honestly.

## Mouse And Window Check

Run only if mouse/window behavior is included in the release.

- [ ] Mouse initializes.
- [ ] Pointer movement is visible.
- [ ] `open notes` opens a window.
- [ ] `open sysinfo` opens a window.
- [ ] Focus, drag, and close behavior are observed.
- [ ] `tasks` and `kill <id>` match window/task behavior.
- [ ] Release notes describe this as a small teaching window layer, not a full
      compositor.

## Filesystem Check

- [ ] `ls`
- [ ] `write demo.txt <content>`
- [ ] `cat demo.txt`
- [ ] `rm demo.txt`
- [ ] Persistence across reboot, if claimed.
- [ ] `fsck`
- [ ] `journal`
- [ ] `fsck repair` only with a controlled disk image and explicit notes.

## App Check

- [ ] `apps`
- [ ] `notes`
- [ ] `notes write <text>`
- [ ] `notes read`
- [ ] `notes clear`
- [ ] `calc 6 * 7`
- [ ] `sysinfo`
- [ ] `open notes` and `open sysinfo`, only if window behavior is included.

## Demo Command Check

- [ ] `demo`
- [ ] `tour`
- [ ] `tour boot`
- [ ] `tour files`
- [ ] `capsule`
- [ ] `capsule current`
- [ ] `doctor`
- [ ] `poster`
- [ ] `poster system`
- [ ] `travel <year>`
- [ ] `quest list`
- [ ] `stats`
- [ ] `inventory`

## Optional Userspace Check

Run only during intentional verification.

- [ ] `ring3`
- [ ] `syshello`
- [ ] `exec hello.elf`
- [ ] Failure or return-to-shell behavior recorded honestly.
- [ ] Release notes do not imply general userland, dynamic linking, libc, argv,
      packages, or a mature process model.

## Optional Networking Check

Run only during intentional ARP/UDP verification.

- [ ] QEMU RTL8139 device configuration recorded.
- [ ] `net`
- [ ] `net arp`
- [ ] `net send`
- [ ] Incoming/outgoing packet evidence captured if claimed.
- [ ] Release notes say static IPv4 ARP/UDP only.
- [ ] No DHCP, DNS, TCP, socket, or broad hardware support is implied.

## Docs Check

- [ ] `README.md` links to current status, demo script, screenshots, release
      checklist, showcase, and manual testing docs.
- [ ] `docs/CURRENT_STATUS.md` matches the release claims.
- [ ] `docs/showcase.md` stays conservative.
- [ ] `docs/demo-script.md` demo paths match implemented commands.
- [ ] `docs/screenshots.md` contains planned or verified capture entries only.
- [ ] `docs/manual-testing.md` remains the detailed verification checklist.
- [ ] `docs/shell-commands.md` matches `kernel/src/shell.rs`.
- [ ] `docs/AI_PROGRESS_LOG.md` records the release result.

## README Check

- [ ] Public name is ChronoOS.
- [ ] Repo/package names remain `chronosapien` where appropriate.
- [ ] The current verification boundary is explicit.
- [ ] Portfolio section explains what the project is and why it matters.
- [ ] Known limitations are visible.
- [ ] No stale public Time Capsule OS naming remains outside historical context.

## Known Limitations Check

- [ ] Not described as a Linux replacement.
- [ ] Not described as production-ready.
- [ ] Runtime verification gaps are listed.
- [ ] Networking is limited to static IPv4 ARP/UDP unless future evidence says
      otherwise.
- [ ] USB is roadmap/design-only.
- [ ] TCP, DHCP, and DNS are roadmap/design-only.
- [ ] Dynamic linker and package manager are roadmap/design-only.
- [ ] Full desktop compositor is roadmap/design-only.
- [ ] Preemptive scheduler is roadmap/design-only.
- [ ] Hardware support is not claimed unless hardware evidence is recorded.

## Final Release Notes Template

```text
Release:
Date:
Type:
Build status:
Runtime evidence:
Screenshots/GIFs:
Implemented in code:
Verified:
Needs runtime verification:
Known limitations:
Next safest step:
```
