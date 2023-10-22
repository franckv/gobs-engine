use std::env;
use std::fs;
use std::path::PathBuf;

macro_rules! debug {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let mut target = PathBuf::from(out_dir);

    let mut found = false;

    while !found {
        if target.ends_with(&profile) {
            found = true;
        } else {
            target = target.parent().unwrap().to_path_buf();
        }
    }

    debug!("=> target = {:?}", target);
    debug!("=> project = {:?}", project_dir);

    for dir in ["shaders", "assets"] {
        let src_path = PathBuf::from(&project_dir).join(dir);
        let dst_path = PathBuf::from(&target).join(dir);

        debug!("Copy {:?} to {:?}", src_path, dst_path);

        let _ = fs::create_dir_all(&dst_path);

        for file in fs::read_dir(src_path).unwrap() {
            let file = file.unwrap();
            let file_name = file.file_name();
            if file.file_type().unwrap().is_file() {
                debug!("Copy {:?}", file_name);
                fs::copy(file.path(), &dst_path.join(file_name)).unwrap();
            }
        }
    }
}
