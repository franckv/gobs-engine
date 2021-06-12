use glsl_to_spirv::ShaderType;

use std::env;
use std::io::Read;
use std::fs;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    for f in fs::read_dir("shaders").unwrap() {
        let f = f.unwrap();
        if !f.file_type().unwrap().is_file() {
            continue;
        }

        let (filename, t) = match f.path().extension().unwrap().to_string_lossy().as_ref() {
            "vert" => ("vert.spv", ShaderType::Vertex),
            "frag" => ("frag.spv", ShaderType::Fragment),
            _ => continue
        };

        let content = fs::read_to_string(f.path()).unwrap();

        let compiled = glsl_to_spirv::compile(&content, t).unwrap();

        let data: Vec<u8> = compiled.bytes().filter_map(|b| b.ok()).collect();

        let path = "assets/shaders";
        let out = format!("{}/{}", path, filename);

        fs::create_dir_all(path).unwrap();
        fs::write(&out, &data).unwrap();
    }
}
