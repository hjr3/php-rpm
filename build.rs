extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main () {
    let default_lib_dir = PathBuf::from("/usr/lib");
    let default_include_dir = PathBuf::from("/usr/include/php");

    let lib_dir = env::var_os("PHP_LIB_DIR").map(PathBuf::from).unwrap_or(default_lib_dir);
    let include_dir = env::var_os("PHP_INCLUDE_DIR").map(PathBuf::from).unwrap_or(default_include_dir);

    if !lib_dir.exists() {
        panic!(
            "PHP library directory does not exist: {}",
            lib_dir.to_string_lossy()
        );
    }

    if !include_dir.exists() {
        panic!(
            "PHP include directory does not exist: {}",
            include_dir.to_string_lossy()
        );
    }

    println!("cargo:rustc-link-lib=php7");
    println!("cargo:rustc-link-search=native={}", lib_dir.to_string_lossy());

    let includes = ["/", "/TSRM", "/Zend", "/main"].iter().map(|d| {
        format!("-I{}{}", include_dir.to_string_lossy(), d)
    }).collect::<Vec<String>>();

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(includes)
        .hide_type("FP_NAN")
        .hide_type("FP_INFINITE")
        .hide_type("FP_ZERO")
        .hide_type("FP_SUBNORMAL")
        .hide_type("FP_NORMAL")
        .hide_type("max_align_t")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
