//! Tiny heap-backed in-memory filesystem.

use alloc::string::String;
use alloc::vec::Vec;
use core::cell::UnsafeCell;

pub const MAX_FILENAME_LEN: usize = 32;

struct File {
    name: String,
    content: String,
}

struct FileList(UnsafeCell<Vec<File>>);

unsafe impl Sync for FileList {}

static FILES: FileList = FileList(UnsafeCell::new(Vec::new()));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FsError {
    EmptyName,
    InvalidName,
    NameTooLong,
    NotFound,
}

/// Visits files in insertion order. Returns false when the filesystem is empty.
pub fn list(mut visit: impl FnMut(&str)) -> bool {
    // SAFETY: The in-memory filesystem is accessed only from the single shell
    // command loop for this milestone.
    let files: &'static Vec<File> = unsafe { &*FILES.0.get() };

    for file in files {
        visit(&file.name);
    }

    !files.is_empty()
}

/// Returns a borrowed file body from the global heap-backed file list.
pub fn read(name: &str) -> Result<&'static str, FsError> {
    validate_name(name)?;

    // SAFETY: Files are mutated only by the single shell command loop. Returned
    // borrows remain valid until that same loop overwrites or removes the file.
    let files: &'static Vec<File> = unsafe { &*FILES.0.get() };

    files
        .iter()
        .find(|file| file.name == name)
        .map(|file| file.content.as_str())
        .ok_or(FsError::NotFound)
}

pub fn write(name: &str, content: &str) -> Result<(), FsError> {
    validate_name(name)?;

    // SAFETY: The shell is single-threaded, so there is no concurrent access to
    // this heap-backed file list.
    let files = unsafe { &mut *FILES.0.get() };

    if let Some(file) = files.iter_mut().find(|file| file.name == name) {
        file.content = String::from(content);
    } else {
        files.push(File {
            name: String::from(name),
            content: String::from(content),
        });
    }

    crate::serial_println!("[CHRONO] fs: write {}", name);
    Ok(())
}

pub fn remove(name: &str) -> Result<(), FsError> {
    validate_name(name)?;

    // SAFETY: The shell is single-threaded, so there is no concurrent access to
    // this heap-backed file list.
    let files = unsafe { &mut *FILES.0.get() };

    let Some(index) = files.iter().position(|file| file.name == name) else {
        return Err(FsError::NotFound);
    };

    files.remove(index);
    crate::serial_println!("[CHRONO] fs: rm {}", name);
    Ok(())
}

fn validate_name(name: &str) -> Result<(), FsError> {
    if name.is_empty() {
        return Err(FsError::EmptyName);
    }

    if name.len() > MAX_FILENAME_LEN {
        return Err(FsError::NameTooLong);
    }

    if name.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return Err(FsError::InvalidName);
    }

    Ok(())
}
