extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main () {

    println!("cargo:rerun-if-env-changed=PHP_LIB_DIR");
    println!("cargo:rerun-if-env-changed=PHP_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=PHP_LINK_STATIC");

    let default_lib_dir = PathBuf::from("/usr/lib");
    let default_include_dir = PathBuf::from("/usr/include/php");
    let default_link_static = false;

    let lib_dir = env::var_os("PHP_LIB_DIR").map(PathBuf::from).unwrap_or(default_lib_dir);
    let include_dir = env::var_os("PHP_INCLUDE_DIR").map(PathBuf::from).unwrap_or(default_include_dir);
    let link_static = env::var_os("PHP_LINK_STATIC").map(|_| true).unwrap_or(default_link_static);

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

    let link_type = if link_static {
        "=static"
    } else {
        ""
    };

    println!("cargo:rustc-link-lib{}=php7", link_type);
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
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
