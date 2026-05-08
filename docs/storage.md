# ChronoFS Persistent Storage

ChronoOS stores shell files on a second QEMU IDE disk named
`chronofs-data.img`. The boot image stays separate, so the filesystem can own
sector 0 of the data disk without overwriting the BIOS boot sector.

## ATA PIO

ATA is the old PC hard-disk interface that QEMU can emulate with simple port
I/O. ChronoOS talks to the primary IDE channel at ports `0x1F0..0x1F7` and
selects the primary slave disk. PIO means "programmed I/O": the CPU copies each
16-bit word through the data port itself. That is slower than DMA, but it is the
simplest disk path to understand because every sector read and write is visible
in the driver.

For one-sector reads, the kernel:

1. Waits until the disk is not busy.
2. Selects the slave drive and the high LBA bits through port `0x1F6`.
3. Writes sector count `1` and the low 24 LBA bits to ports `0x1F2..0x1F5`.
4. Sends command `0x20` to port `0x1F7`.
5. Polls status until `DRQ` says data is ready.
6. Reads 256 words from port `0x1F0`.

Writes use command `0x30`, copy 256 words to port `0x1F0`, then send cache
flush command `0xE7`. Every successful operation logs a line such as
`[CHRONO] disk: write sector 42`.

## LBA Addressing

LBA means logical block addressing. Instead of the older cylinder/head/sector
geometry, the disk is treated as a flat array of 512-byte sectors:

```text
sector 0, sector 1, sector 2, ...
```

ChronoOS uses 28-bit LBA, which is plenty for the 16 MiB teaching disk. The
first version reads and writes one sector at a time so the math stays obvious.

## On-Disk Layout

The data disk is 16 MiB, or 32,768 sectors of 512 bytes.

```text
sector 0      superblock
sectors 1-8   file table
sectors 9-16  free-sector bitmap
sectors 17+   file data
```

The superblock is the filesystem's table of contents. It stores the magic
number `CHRONFS1`, format version, disk size, file count, metadata locations,
and a tiny checksum. If the magic or layout does not match, ChronoOS formats an
empty ChronoFS disk.

The file table has 64 fixed-size entries. Each entry stores:

- whether the slot is used
- filename length and up to 32 filename bytes
- file size in bytes
- first data sector
- number of data sectors

The free-sector bitmap has one bit per sector. A set bit means the sector is in
use. Metadata sectors are marked used during format. File data is allocated as
one contiguous run of sectors, which keeps reads and writes easy to inspect.

## No Journaling Yet

ChronoFS writes file data before metadata, but it does not journal operations.
If power is lost or QEMU is killed during a write or remove, the file table and
bitmap can disagree. Real filesystems use journals, copy-on-write trees, or
repair tools to recover from that kind of interrupted update. ChronoFS skips
that for now so the storage format stays small enough to understand fully.

## QEMU Testing

The run scripts create this disk if it does not exist:

```text
target\x86_64-unknown-none\debug\chronofs-data.img
```

The smoke test is:

```text
CHRONO/84> write hello.txt Hi there
CHRONO/84> reboot
CHRONO/84> cat hello.txt
Hi there
```
