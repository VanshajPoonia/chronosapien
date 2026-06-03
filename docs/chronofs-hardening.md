# ChronoFS Hardening

Status: implemented in code, needs runtime verification.

ChronoFS is the small educational filesystem used by ChronoOS shell commands.
This hardening pass keeps the existing disk format and focuses on inspection,
diagnostics, and conservative repair boundaries.

## Current Design

ChronoFS uses a fixed 16 MiB ATA data disk with 512-byte sectors:

```text
sector 0      superblock
sectors 1-8   file table
sectors 9-16  allocation bitmap
sectors 17+   file data
```

The format has 64 fixed file slots, contiguous file extents, 32-byte filenames,
and a 64 KiB maximum file size. It intentionally does not provide directories,
permissions, timestamps, large-file support, or POSIX compatibility.

When the persistent disk is unavailable, the filesystem falls back to the heap
cache. Heap fallback is useful for keeping the shell understandable, but it is
not persistent and cannot be checked like the on-disk format.

## Inspection Commands

- `fs` / `fs status`: non-destructive mode, disk, file-slot, and journal summary.
- `fs info`: fixed layout, limits, and journal reservation.
- `fs check`: read-only `fsck` summary.
- `fs journal`: one-record journal status.
- `fs help`: command map for ChronoFS inspection.

The `fs` namespace is read-only. Mutating repair remains explicit through
`fsck repair`.

## fsck Boundaries

`fsck` checks:

- superblock magic, version, geometry, file count, and checksum
- file table entries and filename validity
- file extents and maximum file size
- duplicate sector ownership
- allocation bitmap mismatches
- stale metadata in unused file table slots

`fsck repair` can only repair safe bitmap mismatches and clear stale metadata in
unused file table slots. It refuses untrusted superblocks, duplicate-sector
ownership, invalid extents, unsafe errors, and cases where guessing could risk
user data.

## Journal And Recovery

ChronoFS reserves a hidden `__chronofs_journal` file for one metadata operation
at a time. Write/remove operations record intent, sync metadata, mark the record
committed, then clear the journal.

Mount recovery can roll back an uncommitted intent or roll forward a committed
record only when the record and file-table metadata are safe. Corrupt, unknown,
or ambiguous journal records are refused. A clean journal means no pending
journal record; it does not prove that every filesystem behavior has been
runtime-verified.

## Known Risks

- The journal consumes one hidden file slot and one data sector.
- Recovery writes metadata during mount, so recovery tests need controlled disk
  images and serial logs.
- Cache and disk can diverge after lower-level disk write failures.
- File deletion depends on bitmap, file table, superblock, and journal sync
  order.
- Heap fallback has no persistence.
- ChronoFS shell workflows remain unverified until a focused QEMU pass records
  `ls`, `write`, `cat`, `rm`, `fs status`, `fs check`, `journal`, reboot
  persistence, and controlled repair evidence.

## Verification Path

Recommended first runtime pass:

```text
fs status
fs info
ls
write verify.txt hello
cat verify.txt
fs check
fs journal
rm verify.txt
fs check
journal
```

Use `fsck repair` only on a controlled disk image after recording the pre-repair
state. Do not upgrade ChronoFS verification labels until QEMU or hardware logs
and screenshots are captured.
