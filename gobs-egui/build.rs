use std::env;
use std::path::PathBuf;

use fs_extra::dir::CopyOptions;
use fs_extra::dir::copy;

const RESOURCES_DIR: &str = "resources";

include!("../gobs-assets/src/shaders.rs");

#[allow(unused_macros)]
macro_rules! debug {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    println!("cargo:rerun-if-changed={RESOURCES_DIR}/");

    copy_files(RESOURCES_DIR, RESOURCES_DIR);
}

fn copy_files(path: &str, dest: &str) {
    let out_dir = env::var("OUT_DIR").unwrap();

    let mut target = PathBuf::from(out_dir);

    for _ in 0..3 {
        target = target.parent().unwrap().to_path_buf();
    }

    target = target.join(dest);

    debug!("Target {target:?}");

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy_options.content_only = true;

    copy(path, target, &copy_options).unwrap();
}
