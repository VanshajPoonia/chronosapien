typedef unsigned long usize;
typedef long isize;

enum {
    SYS_WRITE = 1,
    SYS_EXIT = 3,
    STDOUT = 1,
};

static inline isize syscall3(usize number, usize arg0, usize arg1, usize arg2) {
    isize result;

    __asm__ volatile(
        "syscall"
        : "=a"(result)
        : "a"(number), "D"(arg0), "S"(arg1), "d"(arg2)
        : "rcx", "r11", "memory");

    return result;
}

static inline void sys_write(int fd, const char *buffer, usize len) {
    (void)syscall3(SYS_WRITE, (usize)fd, (usize)buffer, len);
}

static inline void sys_exit(int code) {
    (void)syscall3(SYS_EXIT, (usize)code, 0, 0);

    for (;;) {
        __asm__ volatile("pause");
    }
}

void _start(void) {
    static const char message[] = "Hello from user space!\n";

    sys_write(STDOUT, message, sizeof(message) - 1);
    sys_exit(0);
}
