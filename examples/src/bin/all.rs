use std::fs;
use std::process::Command;

const EXAMPLES_DIR: &str = "examples/src/bin/";
const NAME: &str = "all";

fn main() {
    let examples: Vec<String> = fs::read_dir(EXAMPLES_DIR)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .path()
                .file_stem()
                .map(|s| s.to_string_lossy().replace(EXAMPLES_DIR, ""))
        })
        .filter(|entry| entry != NAME)
        .collect();

    for example in examples {
        println!("example: {:?}", example);

        let status = Command::new("cargo")
            .args(["run", "--bin", &example])
            .status()
            .unwrap();

        if !status.success() {
            panic!("status: {:?}", status);
        }

        println!("status: {:?}", status);
    }
}
