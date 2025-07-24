use std::env;
use std::path::PathBuf;

use fs_extra::dir::CopyOptions;
use fs_extra::dir::copy;

const SHADERS_GLSL_DIR: &str = "shaders/glsl";
const SHADERS_SLANG_DIR: &str = "shaders/slang";
const SHADERS_OUT_DIR: &str = "shaders/spv";
const SHADERS_ASM_DIR: &str = "shaders/spvasm";
const ASSETS_DIR: &str = "assets";
const RESOURCES_DIR: &str = "resources";
const SHADERS_DIR: &str = "shaders";

include!("../gobs-assets/src/shaders.rs");

#[allow(unused_macros)]
macro_rules! debug {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    println!("cargo:rerun-if-changed={}/", SHADERS_GLSL_DIR);
    println!("cargo:rerun-if-changed={}/", SHADERS_SLANG_DIR);
    println!("cargo:rerun-if-changed={}/", ASSETS_DIR);
    println!("cargo:rerun-if-changed={}/", RESOURCES_DIR);

    compile_glsl_shaders(SHADERS_GLSL_DIR, SHADERS_OUT_DIR, SHADERS_ASM_DIR)
        .expect("Compile shaders");
    compile_slang_shaders(SHADERS_SLANG_DIR, SHADERS_OUT_DIR, SHADERS_ASM_DIR)
        .expect("Compile shaders");
    copy_files(ASSETS_DIR, ASSETS_DIR);
    copy_files(RESOURCES_DIR, RESOURCES_DIR);
    copy_files(SHADERS_OUT_DIR, SHADERS_DIR);
}

fn copy_files(path: &str, dest: &str) {
    let out_dir = env::var("OUT_DIR").unwrap();

    let mut target = PathBuf::from(out_dir);

    for _ in 0..3 {
        target = target.parent().unwrap().to_path_buf();
    }

    target = target.join(dest);

    debug!("Target {:?}", target);

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy_options.content_only = true;

    copy(path, target, &copy_options).unwrap();
}
