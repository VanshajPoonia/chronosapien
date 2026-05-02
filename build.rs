use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let kernel = PathBuf::from(env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());
    let image = manifest_dir
        .join("target")
        .join("x86_64-unknown-none")
        .join("debug")
        .join("chronosapien-bios.img");

    std::fs::create_dir_all(image.parent().unwrap()).unwrap();

    let mut boot_config = bootloader::BootConfig::default();
    boot_config.frame_buffer.minimum_framebuffer_width = Some(1024);
    boot_config.frame_buffer.minimum_framebuffer_height = Some(768);
    boot_config.frame_buffer_logging = false;

    let mut boot = bootloader::BiosBoot::new(&kernel);
    boot.set_boot_config(&boot_config);
    boot.create_disk_image(&image).unwrap();

    println!("cargo:rustc-env=CHRONOSAPIEN_BIOS_IMAGE={}", image.display());
    println!("cargo:rerun-if-changed=kernel/src");
    println!("cargo:rerun-if-changed=kernel/Cargo.toml");
}
