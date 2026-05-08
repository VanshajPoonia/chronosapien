# ChronoOS System Calls

ChronoOS now has a tiny system call path for ring 3 code. This is not a full
process model yet: the demo program still runs in the same address space as the
kernel, and `sys_exit` parks the CPU instead of returning to the shell. The goal
is to prove the controlled transition from user code into kernel services.

## SYSCALL and SYSRET

`SYSCALL` and `SYSRET` are x86_64 instructions for fast system call entry and
return. The kernel writes model-specific registers (MSRs) during boot:

- `IA32_EFER` enables the syscall extension.
- `IA32_LSTAR` stores the kernel entry address.
- `IA32_STAR` stores the kernel and user GDT selectors.
- `IA32_FMASK` tells the CPU which flags to clear on entry.

When ring 3 code executes `SYSCALL`, the CPU loads the configured ring 0 code
selector, jumps to `IA32_LSTAR`, saves the user return RIP in `rcx`, and saves
the user flags in `r11`. When the kernel is ready to return, `SYSRET` restores
the user code/data selectors and resumes at the RIP in `rcx`.

One important catch: `SYSCALL` does not switch stacks by itself. Interrupts and
exceptions can use the TSS to find a ring 0 stack, but syscall entry starts with
the user stack still in `rsp`. ChronoOS immediately switches to a dedicated
kernel syscall stack before calling Rust code.

## Calling Convention

The first ChronoOS syscall ABI is intentionally small:

- `rax` holds the syscall number.
- `rdi`, `rsi`, and `rdx` hold the first three arguments.
- `rax` holds the return value.
- `rcx` and `r11` are clobbered by the CPU as part of `SYSCALL/SYSRET`.

Initial syscall numbers:

| Number | Name | Meaning |
| --- | --- | --- |
| `1` | `sys_write(fd, buf, len)` | Write bytes to screen (`fd=1`) or serial (`fd=2`). |
| `2` | `sys_read(fd, buf, len)` | Read one keyboard line from stdin (`fd=0`). |
| `3` | `sys_exit(code)` | Exit an ELF process back to the shell, or park the older demo. |
| `4` | `sys_uptime()` | Return the PIT tick count. |

In debug builds, the kernel logs calls to serial, for example:

```text
[CHRONO] syscall: write fd=1 len=32
```

## Why User Code Needs Syscalls

Ring 3 code cannot safely call arbitrary kernel functions directly. Kernel code
runs with ring 0 privileges and can touch device ports, descriptor tables,
interrupt state, and memory mappings. A direct jump would either violate the CPU
privilege rules or give user code uncontrolled access to kernel internals.

System calls are the narrow gate. User programs put a syscall number in `rax`,
put arguments in registers, and ask the CPU to enter one audited kernel entry
point. The kernel validates user buffers before touching them and returns a
simple success or error value.

## Current Demo

The `syshello` shell command copies a small machine-code program into the fixed
user code page and enters ring 3. The user program calls:

```text
sys_write(1, "Hello from ring 3 via sys_write\n", 32)
sys_exit(0)
```

The text appears on the framebuffer from ring 3 through the kernel syscall
dispatcher. The separate `exec` path now uses the same syscall ABI for static
ELF programs, but there is still no process scheduler, dynamic linker, argv, or
environment.
