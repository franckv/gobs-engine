use std::{env, path::PathBuf};

use fs_extra::dir::{CopyOptions, copy};

mod shaders;

pub use shaders::*;

#[allow(unused_macros)]
macro_rules! debug {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

pub fn copy_files(path: &str, dest: &str) {
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
