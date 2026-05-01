//! Heap-only single-note app.

use alloc::string::String;
use core::cell::UnsafeCell;

struct NoteSlot(UnsafeCell<Option<String>>);

unsafe impl Sync for NoteSlot {}

static NOTE: NoteSlot = NoteSlot(UnsafeCell::new(None));

pub fn run(args: &str) {
    let args = args.trim();

    // Notes are intentionally one-line and heap-only for this milestone.
    if args.is_empty() {
        crate::println!("Usage: notes <text> | notes read");
        return;
    }

    if args == "read" {
        read_note();
        return;
    }

    store_note(args);
}

fn read_note() {
    // SAFETY: Notes are accessed only from the single shell command loop.
    let note = unsafe { &*NOTE.0.get() };

    match note {
        Some(text) => crate::println!("{}", text),
        None => crate::println!("No note stored."),
    }
}

fn store_note(text: &str) {
    // SAFETY: Notes are accessed only from the single shell command loop.
    let note = unsafe { &mut *NOTE.0.get() };

    *note = Some(String::from(text));
    crate::println!("Note stored.");
}
