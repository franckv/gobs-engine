use gobs_build::copy_files;

const RESOURCES_DIR: &str = "resources";

fn main() {
    println!("cargo:rerun-if-changed={RESOURCES_DIR}/");

    copy_files(RESOURCES_DIR, RESOURCES_DIR);
}
