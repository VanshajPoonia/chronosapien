# Ring 3 User Mode Demo

ChronoOS now has a tiny opt-in user mode demo behind the `ring3` shell command.
It is deliberately not a process model yet: it proves that the CPU can leave
kernel mode, run code at ring 3, and trap back to the kernel when user code
tries to execute a privileged instruction.

## CPU Privilege Rings

x86 CPUs use privilege rings to separate trusted kernel code from less-trusted
program code. Ring 0 is the most privileged level. Kernel code runs there and
can configure hardware, descriptor tables, page tables, interrupts, and I/O
ports. Ring 3 is the normal user level. Code at ring 3 can compute and access
memory pages marked user-accessible, but it cannot execute privileged
instructions such as `hlt`.

Ring 3 exists so a bug in an application does not automatically become a bug in
the kernel. If user code tries something privileged, the CPU raises an exception
instead of allowing the operation.

## GDT Descriptors

The Global Descriptor Table still matters in 64-bit mode even though most
old-style segmentation is disabled. ChronoOS keeps kernel descriptors and adds
ring 3 descriptors:

- kernel code: ring 0 code segment used by the kernel
- kernel data: ring 0 data descriptor kept explicit for teaching
- user code: ring 3 code segment loaded into `CS` by `iretq`
- user data: ring 3 data segment loaded into `SS` by `iretq`
- TSS: system descriptor pointing at the Task State Segment

The ring 3 code and data descriptors have Descriptor Privilege Level 3. That
tells the CPU they are valid selectors for user mode.

## TSS

The Task State Segment is not used for hardware task switching in this kernel.
In long mode, its important job is stack selection. When an interrupt or
exception arrives while the CPU is running at ring 3, the CPU needs a trusted
ring 0 stack before it can safely call a kernel handler. ChronoOS stores that
stack pointer in `TSS.privilege_stack_table[0]`.

The existing double-fault Interrupt Stack Table entry remains separate. That
keeps double faults on their emergency stack while normal ring 3 traps use the
ring 0 privilege stack.

## General Protection Fault

A general protection fault, or `#GP`, is the CPU saying that an instruction or
state transition violated x86 protection rules. In this demo, the user page
starts with `hlt`. `hlt` is privileged, so executing it at ring 3 raises `#GP`.
The kernel handler recognizes that exact instruction pointer, logs the caught
violation, skips the one-byte `hlt`, and returns to user mode.

Expected serial lines:

```text
[CHRONO] kernel: entered ring 3
[CHRONO] ring3: transition ok
[CHRONO] ring3: privilege violation caught — GP fault at 0x...
```

After the handler skips `hlt`, the user code enters a tiny infinite loop. This
demo stays focused on privilege enforcement; `docs/syscalls.md` covers the
separate `SYSCALL/SYSRET` path that builds on this foundation.
