use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
    process,
};

use bindgen::Builder;

fn main() {
    process::Command::new("make")
        .current_dir("../initrd")
        .spawn()
        .unwrap();

    // Generate Rust bindings to the C headers we expose
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let include_dir = crate_dir.join("include/kernel");

    let bindings = bindgen::builder()
        .add_headers(&include_dir)
        .unwrap()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_arg("-ffreestanding")
        .ctypes_prefix("crate::uapi")
        .generate()
        .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}

trait BuilderExt: Sized {
    fn add_headers(self, path: &Path) -> Result<Self, Box<dyn Error>>;
}

impl BuilderExt for Builder {
    fn add_headers(mut self, path: &Path) -> Result<Builder, Box<dyn Error>> {
        if path.is_file() {
            Ok(self.header(path.display().to_string()))
        } else if path.is_dir() {
            for entry in path.read_dir()? {
                self = self.add_headers(&entry?.path())?
            }
            Ok(self)
        } else {
            Err("found something that is not a file or a dir".into())
        }
    }
}
