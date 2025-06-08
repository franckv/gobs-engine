use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use fs_extra::dir::CopyOptions;
use fs_extra::dir::copy;

const SHADERS_GLSL_DIR: &str = "shaders/glsl";
const SHADERS_SLANG_DIR: &str = "shaders/slang";
const SHADERS_OUT_DIR: &str = "shaders/spv";
const ASSETS_DIR: &str = "assets";
const SHADERS_DIR: &str = "shaders";

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

    compile_glsl_shaders(SHADERS_GLSL_DIR, SHADERS_OUT_DIR).expect("Compile shaders");
    compile_slang_shaders(SHADERS_SLANG_DIR, SHADERS_OUT_DIR).expect("Compile shaders");
    copy_files(ASSETS_DIR, ASSETS_DIR);
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

fn compile_glsl_shaders(path_in: &str, path_out: &str) -> Result<(), io::Error> {
    for f in fs::read_dir(path_in).unwrap() {
        let f = f?;
        if !f.file_type()?.is_file() {
            continue;
        }

        let file = f.path();
        let file_name = file.to_str().unwrap();

        let out = format!("{}.spv", file_name.replace(path_in, path_out));

        match f.path().extension().unwrap().to_string_lossy().as_ref() {
            "comp" | "vert" | "frag" => (),
            _ => continue,
        };

        debug!("Shader (glsl): {} -> {}", file_name, out);

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
                .arg(format!("glslangValidator -V {} -o {}", file_name, out))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={:?}", output);
            }
        }
    }

    Ok(())
}

fn compile_slang_shaders(path_in: &str, path_out: &str) -> Result<(), io::Error> {
    for f in fs::read_dir(path_in).unwrap() {
        let f = f?;
        if !f.file_type()?.is_file() {
            continue;
        }

        let file = f.path();
        let file_name = file.to_str().unwrap();

        let file_stem = Path::with_extension(&file, "");

        let out = format!(
            "{}.spv",
            file_stem.to_str().unwrap().replace(path_in, path_out)
        );

        match f.path().extension().unwrap().to_string_lossy().as_ref() {
            "slang" => (),
            _ => continue,
        };

        debug!("Shader (slang): {} -> {}", file_name, out);

        #[cfg(target_os = "windows")]
        {
            unimplemented!();
        }
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("sh")
                .arg("-c")
                .arg(format!("slangc {} -target spirv -o {}", file_name, out))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={:?}", output);
            }
        }
    }

    Ok(())
}
