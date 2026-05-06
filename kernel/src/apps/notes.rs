//! Tiny note app backed by the shared filesystem facade.

const NOTE_FILE: &str = "note.txt";

pub fn run(args: &str) {
    let args = args.trim();

    // Notes are intentionally one-line for this milestone.
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
    match crate::fs::read(NOTE_FILE) {
        Ok(text) => crate::println!("{}", text),
        Err(_) => crate::println!("No note stored."),
    }
}

fn store_note(text: &str) {
    match crate::fs::write(NOTE_FILE, text) {
        Ok(()) => crate::println!("Note stored."),
        Err(_) => crate::println!("Could not store note."),
    }
}
