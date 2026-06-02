# ChronoOS Hardware Testing

Status: needs manual verification.

This guide is for cautious real-hardware experiments. Do not treat ChronoOS as
a production OS or Linux replacement. Record all hardware evidence in
`docs/AI_PROGRESS_LOG.md` before changing any status label to `verified on
hardware`.

## Prerequisites

- A sacrificial test machine or removable-media-only setup.
- Secure Boot disabled unless ChronoOS is later signed.
- A known-good recovery path for the machine.
- A serial logging path if available, such as COM1, USB serial firmware support,
  or a capture setup documented before boot.
- The exact image path, build command, and write command recorded before use.
- A backup of any removable media or disk that could be overwritten.

## USB Limitation Warning

USB HID, USB storage, and USB serial drivers are roadmap/design-only. The UEFI
loader can boot from a firmware-readable image, but the kernel should not be
expected to drive USB keyboards, USB mice, USB disks, or USB serial adapters
after handoff unless future driver work adds that support.

## Keyboard And Mouse Limitation Warning

ChronoOS currently targets legacy-style PS/2 keyboard and mouse paths. On modern
hardware, firmware may emulate PS/2 during boot and then stop after handoff, or
may expose only USB devices. Treat missing keyboard or mouse input as an
expected hardware limitation, not a verified kernel failure.

## Storage And Data-Loss Warning

ChronoFS and ATA storage paths are educational and still need staged runtime
verification. Never point ChronoOS at a disk containing important data. Prefer a
fresh removable drive, a sacrificial test disk, or QEMU disk images until real
hardware storage behavior is understood.

## UEFI Boot Checklist

- [ ] Build the UEFI image with `scripts/build-uefi.ps1`.
- [ ] Confirm the image path, size, and timestamp.
- [ ] Disable Secure Boot.
- [ ] Boot from removable media through the firmware boot picker.
- [ ] Record whether the UEFI loader screen appears.
- [ ] Record whether the kernel reaches framebuffer output.
- [ ] Record whether serial logging is available.
- [ ] Record whether keyboard input works after kernel handoff.
- [ ] Keep the result as `needs manual verification` unless the observed
      evidence is captured and logged.

## BIOS Boot Checklist

- [ ] Build the normal BIOS image with `scripts/build.ps1` or the custom BIOS
      image only after `nasm` and the custom path are verified.
- [ ] Confirm the target machine supports BIOS/CSM boot if using BIOS images.
- [ ] Boot from removable media through the firmware boot picker.
- [ ] Record serial output, framebuffer output, input behavior, and any hang.
- [ ] Keep normal BIOS, custom BIOS, and UEFI results separate.

## Serial Logging Checklist

- [ ] Identify the serial device and baud assumptions before boot.
- [ ] Capture logs to a timestamped file.
- [ ] Look for `[CHRONO] boot start`.
- [ ] Look for `[CHRONO] boot complete`.
- [ ] Record missing, garbled, or partial serial output honestly.
- [ ] Do not infer framebuffer, keyboard, mouse, filesystem, or networking
      success from serial-only evidence.

## Rollback And Recovery Plan

- [ ] Keep the original OS boot media available.
- [ ] Know the firmware key for boot-device selection.
- [ ] Keep a second machine available for rewriting removable media.
- [ ] If a boot attempt hangs, power off and remove the ChronoOS media before
      changing firmware settings.
- [ ] If storage behavior is unexpected, stop testing and preserve logs instead
      of retrying with important disks attached.

## Status Policy

- Real hardware remains `needs manual verification` until actual hardware
  evidence is recorded.
- USB HID/storage/serial remain `roadmap/design-only`.
- Hardware networking remains unverified unless packet/log evidence is captured
  on the real device.
- A successful QEMU boot does not imply hardware support.
