//! Tiny spinlock for shared kernel state.
//!
//! On one CPU, disabling interrupts is enough to make many critical sections
//! non-reentrant. On SMP, another CPU can still touch the same memory at the
//! same time, so shared state needs an atomic lock too.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for SpinLock<T> {}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
    restore_interrupts: bool,
}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock_irq(&self) -> SpinLockGuard<'_, T> {
        let restore_interrupts = x86_64::instructions::interrupts::are_enabled();
        x86_64::instructions::interrupts::disable();
        self.acquire();

        SpinLockGuard {
            lock: self,
            restore_interrupts,
        }
    }

    pub unsafe fn lock_irq_raw(&self) -> (*mut T, bool) {
        let restore_interrupts = x86_64::instructions::interrupts::are_enabled();
        x86_64::instructions::interrupts::disable();
        self.acquire();

        (self.data.get(), restore_interrupts)
    }

    pub fn raw_lock_byte(&self) -> *mut u8 {
        &self.locked as *const AtomicBool as *mut u8
    }

    pub unsafe fn unlock_irq_raw(&self, restore_interrupts: bool) {
        self.locked.store(false, Ordering::Release);
        if restore_interrupts {
            x86_64::instructions::interrupts::enable();
        }
    }

    fn acquire(&self) {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }
    }
}

impl<T> core::ops::Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> core::ops::DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
        if self.restore_interrupts {
            x86_64::instructions::interrupts::enable();
        }
    }
}
