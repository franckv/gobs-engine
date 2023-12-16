use std::env;
use std::path::PathBuf;

use fs_extra::dir::copy;
use fs_extra::dir::CopyOptions;

#[allow(unused_macros)]
macro_rules! debug {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();

    println!("cargo:rerun-if-changed=shaders/");

    let mut target = PathBuf::from(out_dir);

    let mut found = false;

    while !found {
        if target.ends_with(&profile) {
            found = true;
        } else {
            target = target.parent().unwrap().to_path_buf();
        }
    }

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;

    copy("shaders", target, &copy_options).unwrap();
}
