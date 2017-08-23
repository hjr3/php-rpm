extern crate gcc;

use std::env;
use std::path::PathBuf;

fn main() {
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

    gcc::Build::new()
        .file("src/shim.c")
        .include(&include_dir)
        .include(&format!("{}/TSRM", include_dir.display()))
        .include(&format!("{}/Zend", include_dir.display()))
        .include(&format!("{}/main", include_dir.display()))
        .compile("foo");
}
