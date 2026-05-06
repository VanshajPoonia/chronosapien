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
