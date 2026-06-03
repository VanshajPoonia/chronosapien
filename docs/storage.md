# ChronoFS Persistent Storage

ChronoOS stores shell files on a second QEMU IDE disk named `chronofs-data.img`.
The boot image stays separate, so the filesystem can own sector 0 of the data
disk without overwriting the BIOS boot sector.

Status: implemented in code, needs runtime verification.

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

The shell output groups what was checked, what looks suspicious, what was
repaired, and what was intentionally not repaired. `fsck repair` prints a
mutation warning and should be used only with controlled disk images and
before/after evidence.

Status: implemented in code, needs runtime verification.

## Journal And Recovery

ChronoFS now has a tiny hidden one-sector journal stored as `__chronofs_journal`.
The journal records one write/remove intent at a time, marks it committed after
metadata sync, and is checked during mount.

Mount recovery can roll back an uncommitted intent or roll forward a committed
operation when the journal record is safe. Recovery also rebuilds the bitmap
from file table entries. Unsafe or corrupt journal records are refused and
reported through serial logs.

Status: implemented in code, needs runtime verification. Crash recovery has not
been proven in QEMU or on hardware in this repo.

A clean journal means there is no pending one-record journal operation. It does
not prove full filesystem runtime verification.

## Hardening Notes

See `docs/chronofs-hardening.md` for the current design, risks, inspection
commands, repair boundaries, and recommended verification path.

## QEMU Smoke Test Target

The run scripts create this disk if it does not exist:

```text
target\x86_64-unknown-none\debug\chronofs-data.img
```

Manual verification should use `docs/manual-testing.md` and record real
evidence before changing any status to runtime-verified.
