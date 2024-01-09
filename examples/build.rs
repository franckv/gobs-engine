use naga::back::spv;
use naga::front::glsl;
use naga::valid::Validator;
use naga::ShaderStage;

use std::fs;

const SHADERS_DIR: &str = "shaders";

#[allow(unused_macros)]
macro_rules! debug {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    println!("cargo:rerun-if-changed={}/", SHADERS_DIR);

    for f in fs::read_dir(SHADERS_DIR).unwrap() {
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
