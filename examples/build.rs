use gobs_build::{compile_glsl_shaders, compile_slang_shaders, copy_files};

const SHADERS_GLSL_DIR: &str = "shaders/glsl";
const SHADERS_SLANG_DIR: &str = "shaders/slang";
const SHADERS_SLANG_INCLUDE_DIR: &str = "../gobs-assets/shaders/slang/include";
const SHADERS_OUT_DIR: &str = "shaders/spv";
const SHADERS_ASM_DIR: &str = "shaders/spvasm";
const ASSETS_DIR: &str = "assets";
const RESOURCES_DIR: &str = "resources";
const SHADERS_DIR: &str = "shaders";

fn main() {
    println!("cargo:rerun-if-changed={SHADERS_GLSL_DIR}/");
    println!("cargo:rerun-if-changed={SHADERS_SLANG_DIR}/");
    println!("cargo:rerun-if-changed={SHADERS_SLANG_INCLUDE_DIR}/");
    println!("cargo:rerun-if-changed={ASSETS_DIR}/");
    println!("cargo:rerun-if-changed={RESOURCES_DIR}/");

    compile_glsl_shaders(SHADERS_GLSL_DIR, SHADERS_OUT_DIR, SHADERS_ASM_DIR)
        .expect("Compile shaders");
    compile_slang_shaders(
        SHADERS_SLANG_DIR,
        SHADERS_SLANG_INCLUDE_DIR,
        SHADERS_OUT_DIR,
        SHADERS_ASM_DIR,
    )
    .expect("Compile shaders");
    copy_files(ASSETS_DIR, ASSETS_DIR);
    copy_files(RESOURCES_DIR, RESOURCES_DIR);
    copy_files(SHADERS_OUT_DIR, SHADERS_DIR);
}
