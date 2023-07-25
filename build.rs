use std::env;
use std::path::{Path, PathBuf};

fn get_output_path() -> PathBuf {
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string).join("target").join(build_type);
    return PathBuf::from(path);
}

fn main() {
    let target_dir = get_output_path();
    let src = Path::join(&env::current_dir().unwrap(), "icon.png");
    let dest = Path::join(Path::new(&target_dir), Path::new("icon.png"));
    std::fs::copy(src, dest).expect("Can't copy icon image");
}