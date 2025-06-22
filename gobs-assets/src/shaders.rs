use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use tracing::debug;

pub fn compile_glsl_shaders(path_in: &str, path_out: &str, asm_out: &str) -> Result<(), io::Error> {
    for f in fs::read_dir(path_in)? {
        let f = f?;
        if !f.file_type()?.is_file() {
            continue;
        }

        let file = f.path();
        let file_name = file.to_str().unwrap();

        let spv_out = format!("{}.spv", file_name.replace(path_in, path_out));
        let asm_out = format!("{}.spvasm", file_name.replace(path_in, asm_out));

        match f.path().extension().unwrap().to_string_lossy().as_ref() {
            "comp" | "vert" | "frag" => (),
            _ => continue,
        };

        debug!("Shader (glsl): {} -> {}", file_name, spv_out);

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("cmd")
                .arg("/C")
                .arg(format!(
                    "glslangValidator.exe -V {} -o {}",
                    file_name, spv_out
                ))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={:?}", output);
            }

            let output = Command::new("cmd")
                .arg("/C")
                .arg(format!("spirv-dis.exe {} -o {}", spv_out, asm_out))
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
                .arg(format!("glslangValidator -V {} -o {}", file_name, spv_out))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={:?}", output);
            }

            let output = Command::new("sh")
                .arg("-c")
                .arg(format!("spirv-dis {} -o {}", spv_out, asm_out))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={:?}", output);
            }
        }
    }

    Ok(())
}

pub fn compile_slang_shaders(
    path_in: &str,
    path_out: &str,
    asm_out: &str,
) -> Result<(), io::Error> {
    for f in fs::read_dir(path_in).unwrap() {
        let f = f?;
        if !f.file_type()?.is_file() {
            continue;
        }

        let file = f.path();
        let file_name = file.to_str().unwrap();

        let file_stem = Path::with_extension(&file, "");

        let spv_out = format!(
            "{}.spv",
            file_stem.to_str().unwrap().replace(path_in, path_out)
        );

        let asm_out = format!(
            "{}.spvasm",
            file_stem.to_str().unwrap().replace(path_in, asm_out)
        );

        match f.path().extension().unwrap().to_string_lossy().as_ref() {
            "slang" => (),
            _ => continue,
        };

        debug!("Shader (slang): {} -> {}", file_name, spv_out);

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("cmd")
                .arg("/C")
                .arg(format!(
                    "slangc.exe {} -target spirv -o {}",
                    file_name, spv_out
                ))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={:?}", output);
            }

            let output = Command::new("cmd")
                .arg("/C")
                .arg(format!(
                    "slangc.exe {} -target spirv-asm -o {}",
                    file_name, asm_out
                ))
                .output()
                .expect("Error disassembling shader");

            if !output.status.success() {
                panic!("Disassemble status={:?}", output);
            }
        }
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("sh")
                .arg("-c")
                .arg(format!("slangc {} -target spirv -o {}", file_name, spv_out))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={:?}", output);
            }

            let output = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "slangc {} -target spirv-asm -o {}",
                    file_name, asm_out
                ))
                .output()
                .expect("Error disassembling shader");

            if !output.status.success() {
                panic!("Disassemble status={:?}", output);
            }
        }
    }

    Ok(())
}
