# ChronoFS Hardening

Status: implemented in code, partially verified in QEMU for the disposable
image flow recorded on 2026-06-13.

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

`fsck` and `fs check` print an explicit clean/not-clean line before listing
checked areas, suspicious findings, repaired items, and not-repaired reasons.
This keeps demos and verification notes from inferring cleanliness from a
summary label alone.

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
- ChronoFS shell workflows have narrow QEMU evidence for the 2026-06-13
  disposable image pass, but controlled repair, recovery states, heap fallback,
  and disk-error behavior remain unverified.

## 2026-06-13 Disposable QEMU Verification

This pass used visible single-core BIOS QEMU and a fresh disposable data image:

```text
/private/tmp/chronoos-chronofs-20260613-191106.img
```

Verified in QEMU with exact serial command lines plus framebuffer screenshots:

- `fs status`
- `fs info`
- `ls`
- `write verify.txt chrono verification test`
- `cat verify.txt`
- `fs check`
- `fs journal`
- `fsck`
- `journal`
- `rm verify.txt`
- post-delete `ls`
- reboot with the same disposable image
- post-reboot `ls`, `cat verify.txt`, `fs status`, and `journal`

Observed result: the fresh image formatted and mounted, the file was written and
read back, read-only `fs check`/`fsck` reported clean, the journal reported
clean/empty after completed operations, the file was removed, and after reboot
with the same image `verify.txt` remained absent.

Evidence:

- `/private/tmp/chronoos-chronofs-20260613-191106.serial.log`
- `/private/tmp/chronoos-chronofs-20260613-191106-reboot.serial.log`
- `/private/tmp/chronoos-chronofs-20260613-191106-current-before-rm-retry.png`
- `/private/tmp/chronoos-chronofs-20260613-191106-post-delete-ls.png`
- `/private/tmp/chronoos-chronofs-20260613-191106-reboot-persistence.png`

Not verified by this pass: `fsck repair`, bitmap/stale-slot repair, repair
refusal cases, journal rollback, journal roll-forward, corrupt journal refusal,
independent write persistence before deletion, heap fallback, disk-error
handling, hardware, or non-disposable image behavior.

## Verification Path

Recommended repair/recovery follow-up pass:

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
state. Do not upgrade repair or recovery labels until QEMU or hardware logs and
screenshots are captured.
