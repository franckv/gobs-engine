use std::env;
use std::fs;
use std::path::PathBuf;

use fs_extra::dir::copy;
use fs_extra::dir::CopyOptions;

use naga::back::spv;
use naga::front::glsl;
use naga::valid::Validator;
use naga::ShaderStage;

const SHADERS_DIR: &str = "shaders";
const ASSETS_DIR: &str = "assets";

#[allow(unused_macros)]
macro_rules! debug {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    copy_files(ASSETS_DIR);
    copy_files(SHADERS_DIR);
    compile_shaders(SHADERS_DIR);
}

fn copy_files(path: &str) {
    println!("cargo:rerun-if-changed={}/", path);

    let out_dir = env::var("OUT_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();

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

    copy(path, target, &copy_options).unwrap();
}

fn compile_shaders(path: &str) {
    println!("cargo:rerun-if-changed={}/", path);

    for f in fs::read_dir(path).unwrap() {
        let f = f.unwrap();
        if !f.file_type().unwrap().is_file() {
            continue;
        }

        let file = f.path();
        let file_name = file.to_str().unwrap();

        let out = format!("{}.spv", file_name);

        if std::path::Path::new(&out).exists() {
            continue;
        }

        let stage = match f.path().extension().unwrap().to_string_lossy().as_ref() {
            "comp" => ShaderStage::Compute,
            "vert" => ShaderStage::Vertex,
            "frag" => ShaderStage::Fragment,
            _ => continue,
        };

        debug!("Input: {}", file_name);
        debug!("Output: {}", out);

        let content = fs::read_to_string(f.path()).unwrap();

        let mut front = glsl::Frontend::default();
        let module = front.parse(&glsl::Options::from(stage), &content).unwrap();
        let info = Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();

        let mut data = Vec::new();
        let mut options = spv::Options::default();
        options
            .flags
            .remove(spv::WriterFlags::ADJUST_COORDINATE_SPACE);
        let mut writer = spv::Writer::new(&options).unwrap();
        writer
            .write(&module, &info, None, &None, &mut data)
            .expect("Failed to write shader");

        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * std::mem::size_of::<u32>(),
            )
        };
        fs::write(&out, bytes).unwrap();
    }
}
