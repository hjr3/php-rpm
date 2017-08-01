extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main () {
    println!("cargo:rustc-link-lib=php7");
    println!("cargo:rustc-link-search=native=/home/herman/projects/php-src/libs");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/home/herman/projects/php-src")
        .clang_arg("-I/home/herman/projects/php-src/TSRM")
        .clang_arg("-I/home/herman/projects/php-src/Zend")
        .clang_arg("-I/home/herman/projects/php-src/main")
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
