use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use fs_extra::dir::copy;
use fs_extra::dir::CopyOptions;

const SHADERS_IN_DIR: &str = "shaders/in";
const SHADERS_OUT_DIR: &str = "shaders";
const ASSETS_DIR: &str = "assets";

#[allow(unused_macros)]
macro_rules! debug {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    println!("cargo:rerun-if-changed={}/", SHADERS_IN_DIR);
    println!("cargo:rerun-if-changed={}/", ASSETS_DIR);

    compile_shaders(SHADERS_IN_DIR, SHADERS_OUT_DIR);
    copy_files(ASSETS_DIR);
    copy_files(SHADERS_OUT_DIR);
}

fn copy_files(path: &str) {
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

fn compile_shaders(path_in: &str, path_out: &str) {
    for f in fs::read_dir(path_in).unwrap() {
        let f = f.unwrap();
        if !f.file_type().unwrap().is_file() {
            continue;
        }

        let file = f.path();
        let file_name = file.to_str().unwrap();

        let out = format!("{}.spv", file_name.replace(path_in, path_out));

        match f.path().extension().unwrap().to_string_lossy().as_ref() {
            "comp" | "vert" | "frag" => (),
            _ => continue,
        };

        debug!("Shader: {} -> {}", file_name, out);

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("cmd")
                .arg("/C")
                .arg(&format!("glslangValidator.exe -V {} -o {}", file_name, out))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={:?}", output);
            }
        }
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("sh")
                .arg("-c")
                .arg(&format!("glslangValidator -V {} -o {}", file_name, out))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={:?}", output);
            }
        }
    }
}
