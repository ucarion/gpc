extern crate bindgen;
extern crate gcc;

use std::env;
use std::path::PathBuf;

fn main() {
    gcc::Build::new().file("src/gpc.c").compile("gpc");

    let bindings = bindgen::Builder::default()
        .header("src/gpc.h")
        .generate()
        .expect("Unable to generate GPC bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Unable to write out bindings to file");
}
