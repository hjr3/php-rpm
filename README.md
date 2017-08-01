# PHP Rust Process Manager (php-rpm)

An _experimental_ process manager that uses Rust's hyper library to act as a frontend for PHP.

## Problem

The standard way to run PHP is to use nginx + php_fpm. Not only is this a pain to setup, but FastCGI is not very fast.

## Solution

Embed PHP into Rust so that hyper can accept the HTTP request, pass that information off to a PHP script and then have hyper return the HTTP response.

## Installation

I currently have PHP compiled in a specific directory `/home/herman/projects/php-src`. You will have to either replicate this directory or modify the code to find it in whatever directory you have it in.

The code uses bindgen to dynamically build bindings for PHP 7.1. Using `cargo build` should give you a working binary.

## Usage

Depending on the location of `libphp7.so` you may need to provide `LD_LIBRARY_PATH`. Example: `LD_LIBRARY_PATH="/path/to/lib" cargo run`

## Inspiration

I thought of this idea while working on weldr and thinking about how weldr would be used at my day job (which uses PHP).

## Related

See php_fpm.

## Thanks

A big thanks to Sara Goleman and her book _Extending and Embedding PHP_. Also thanks to the people that created bindgen.

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
