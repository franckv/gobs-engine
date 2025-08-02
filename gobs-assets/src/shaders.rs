use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

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

        tracing::debug!("Shader (glsl): {file_name} -> {spv_out}");

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("cmd")
                .arg("/C")
                .arg(format!("glslangValidator.exe -V {file_name} -o {spv_out}"))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={output:?}");
            }

            let output = Command::new("cmd")
                .arg("/C")
                .arg(format!("spirv-dis.exe {spv_out} -o {asm_out}"))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={output:?}");
            }
        }
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("sh")
                .arg("-c")
                .arg(format!("glslangValidator -V {file_name} -o {spv_out}"))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={output:?}");
            }

            let output = Command::new("sh")
                .arg("-c")
                .arg(format!("spirv-dis {spv_out} -o {asm_out}"))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={output:?}");
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

        tracing::debug!("Shader (slang): {} -> {}", file_name, spv_out);

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("cmd")
                .arg("/C")
                .arg(format!("slangc.exe {file_name} -target spirv -o {spv_out}"))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={output:?}");
            }

            let output = Command::new("cmd")
                .arg("/C")
                .arg(format!(
                    "slangc.exe {file_name} -target spirv-asm -o {asm_out}"
                ))
                .output()
                .expect("Error disassembling shader");

            if !output.status.success() {
                panic!("Disassemble status={output:?}");
            }
        }
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("sh")
                .arg("-c")
                .arg(format!("slangc {file_name} -target spirv -o {spv_out}"))
                .output()
                .expect("Error compiling shader");

            if !output.status.success() {
                panic!("Compile status={output:?}");
            }

            let output = Command::new("sh")
                .arg("-c")
                .arg(format!("slangc {file_name} -target spirv-asm -o {asm_out}"))
                .output()
                .expect("Error disassembling shader");

            if !output.status.success() {
                panic!("Disassemble status={output:?}");
            }
        }
    }

    Ok(())
}
