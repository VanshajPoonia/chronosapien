# ChronoFS Persistent Storage

ChronoOS stores shell files on a second QEMU IDE disk named `chronofs-data.img`.
The boot image stays separate, so the filesystem can own sector 0 of the data
disk without overwriting the BIOS boot sector.

Status: implemented in code, partially verified in QEMU for the disposable
image flow recorded on 2026-06-13.

## ATA PIO

ATA is the old PC hard-disk interface that QEMU can emulate with simple port
I/O. ChronoOS talks to the primary IDE channel at ports `0x1F0..0x1F7` and
selects the primary slave disk. PIO means "programmed I/O": the CPU copies each
16-bit word through the data port itself.

For one-sector reads, the kernel waits for the disk, selects the LBA, sends
command `0x20`, waits for `DRQ`, and reads 256 words from port `0x1F0`. Writes
use command `0x30`, copy 256 words to the data port, then flush with `0xE7`.

## On-Disk Layout

The data disk is 16 MiB, or 32,768 sectors of 512 bytes.

```text
sector 0      superblock
sectors 1-8   file table
sectors 9-16  free-sector bitmap
sectors 17+   file data
```

The superblock stores magic `CHRONFS1`, format version, disk size, file count,
metadata locations, and a small checksum. The file table has 64 fixed-size
entries. File data is allocated as contiguous sector runs so the layout stays
easy to inspect.

Current limits:

- filenames are at most 32 bytes
- filenames cannot contain whitespace
- each file can use at most 64 KiB
- there are 64 file slots
- there are no directories, permissions, timestamps, or POSIX compatibility

## Inspection Commands

ChronoOS now has a read-only `fs` inspection namespace:

- `fs` / `fs status`: print mode, disk availability, file counts, slot usage,
  and journal state.
- `fs info`: print the fixed layout, limits, and journal reservation.
- `fs check`: run a read-only `fsck` summary.
- `fs journal`: print the same journal status as `journal`.
- `fs help`: show the ChronoFS inspection command map.

The `fs` namespace does not repair or rewrite metadata. Mutating repair remains
explicit through `fsck repair`.

## fsck And Repair

`fsck` is implemented in code as a conservative checker. It inspects the
superblock, file table, filename validity, file extents, allocation bitmap, and
duplicate sector claims.

`fsck repair` is intentionally narrow. It can repair safe bitmap mismatches and
clear stale metadata in unused file table slots. It refuses ambiguous damage,
duplicate-sector ownership, untrusted superblocks, and cases where guessing
would risk user data.

The shell output groups what was checked, whether the check is clean, what
looks suspicious, what was repaired, and what was intentionally not repaired.
`fsck repair` prints a mutation warning and should be used only with controlled
disk images and before/after evidence.

Status: implemented in code, partially verified in QEMU for clean read-only
checks on 2026-06-13. Mutating repair remains unverified.

## Journal And Recovery

ChronoFS now has a tiny hidden one-sector journal stored as `__chronofs_journal`.
The journal records one write/remove intent at a time, marks it committed after
metadata sync, and is checked during mount.

Mount recovery can roll back an uncommitted intent or roll forward a committed
operation when the journal record is safe. Recovery also rebuilds the bitmap
from file table entries. Unsafe or corrupt journal records are refused and
reported through serial logs.

Status: implemented in code, partially verified in QEMU for clean/empty journal
status on 2026-06-13. Crash recovery has not been proven in QEMU or on hardware
in this repo.

A clean journal means there is no pending one-record journal operation. It does
not prove full filesystem runtime verification.

## Hardening Notes

See `docs/chronofs-hardening.md` for the current design, risks, inspection
commands, repair boundaries, and recommended verification path.

## 2026-06-13 Disposable QEMU Verification

A controlled visible single-core BIOS QEMU pass used a fresh disposable 16 MiB
data image at `/private/tmp/chronoos-chronofs-20260613-191106.img`, not the
repo's `target/x86_64-unknown-none/debug/chronofs-data.img`.

Observed with exact serial `cmd:` lines and screenshots:

- fresh format/mount reached `[CHRONO] boot complete`
- `fs status` reported persistent ATA disk, disk availability, file/slot counts,
  and journal summary
- `fs info` reported the `CHRONFS1` v1 layout and limits
- initial `ls` showed no visible user files
- `write verify.txt chrono verification test` completed
- `cat verify.txt` printed `chrono verification test`
- `fs check` and `fsck` reported a clean read-only check with checked areas,
  suspicious counts, repaired `0`, and read-only not-repaired wording
- `fs journal` and `journal` reported a clean/empty journal after completed
  operations
- `rm verify.txt` completed and a later `ls` no longer showed `verify.txt`
- rebooting with the same disposable image showed `verify.txt` remained absent,
  and `cat verify.txt` reported `file not found: verify.txt`

Evidence:

- main serial log: `/private/tmp/chronoos-chronofs-20260613-191106.serial.log`
- reboot serial log:
  `/private/tmp/chronoos-chronofs-20260613-191106-reboot.serial.log`
- screenshots:
  `/private/tmp/chronoos-chronofs-20260613-191106-current-before-rm-retry.png`,
  `/private/tmp/chronoos-chronofs-20260613-191106-post-delete-ls.png`, and
  `/private/tmp/chronoos-chronofs-20260613-191106-reboot-persistence.png`

This verifies read/write/read/delete and delete persistence on the disposable
image. It does not verify independent write persistence before deletion,
`fsck repair`, journal rollback/roll-forward, corrupt journal refusal, heap
fallback, disk-error handling, or hardware.

## QEMU Smoke Test Target

The run scripts create this disk if it does not exist:

```text
target\x86_64-unknown-none\debug\chronofs-data.img
```

Manual verification should use `docs/manual-testing.md` and record real
evidence before changing any status to runtime-verified.
