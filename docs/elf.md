# ELF64 Loading

ChronoOS can run one foreground static ELF64 program with `exec <filename>`.
The file is read from ChronoFS as bytes, parsed in the kernel, mapped into a
new user address space, and entered at its ELF entry point.

## ELF Header

An ELF file starts with a fixed header. ChronoOS accepts only the small subset
needed for first user programs:

- Magic bytes: `0x7f E L F`.
- Class: ELF64.
- Endianness: little-endian.
- Type: `ET_EXEC`, a fixed-address executable.
- Machine: `EM_X86_64`.
- Entry point: the user RIP where execution begins.
- Program-header table: an array describing which file ranges become memory.

Dynamic linking is not supported yet. There is no interpreter segment, loader,
relocation processing, libc, argv, or environment.

## PT_LOAD Segments

A `PT_LOAD` program header describes bytes that should exist in memory when the
program starts. It includes:

- `p_offset`: where the bytes begin in the ELF file.
- `p_vaddr`: the user virtual address where those bytes should appear.
- `p_filesz`: how many file bytes to copy.
- `p_memsz`: how many memory bytes to reserve.
- `p_flags`: executable, writable, and readable intent.

ChronoOS maps every page touched by a loadable segment. It copies the file
bytes into freshly allocated frames and leaves the rest zeroed, which gives
simple BSS behavior when `p_memsz` is larger than `p_filesz`.

## Process Page Tables

Each `exec` builds a new PML4. Kernel mappings are copied so interrupts,
exceptions, and syscalls still have supervisor-only kernel code and stacks after
CR3 changes. User program mappings live in ChronoOS's ELF user window starting
at `0x0000008000000000`, separate from the older fixed `syshello` demo pages.

The kernel maps ELF pages with `USER_ACCESSIBLE`. Writable segments and the user
stack also get `WRITABLE`. Syscalls validate user buffers against the active
process ranges before reading or writing through user pointers.

## Test Program

`user/hello.c` is a freestanding program with `_start`. It uses inline assembly
to call:

```text
sys_write(1, "Hello from user space!\n", 23)
sys_exit(0)
```

Build and install it into the ChronoFS data disk with:

```powershell
.\scripts\build-user.ps1
```

Then boot ChronoOS and run:

```text
CHRONO/84> exec hello.elf
Hello from user space!
[process exited: 0]
```

The serial log should include lines like:

```text
[CHRONO] elf: loading hello.elf, entry 0x8000100000
[CHRONO] elf: PT_LOAD vaddr=...
[CHRONO] syscall: write fd=1 len=23
```
