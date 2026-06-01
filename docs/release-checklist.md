# ChronoOS Release Checklist

Use this checklist before publishing a demo build, portfolio update, README
refresh, or build-in-public post. A release can be documentation-only, a QEMU
demo release, or a hardware experiment, but each claim must match recorded
evidence.

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
