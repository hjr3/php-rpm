# PHP Rust Process Manager (php-rpm)

[![Build Status](https://travis-ci.org/hjr3/php-rpm.svg?branch=master)](https://travis-ci.org/hjr3/php-rpm)

An _experimental_ process manager that uses Rust's hyper library to act as a frontend for PHP.

## Problem

The standard way to run PHP is to use nginx + php_fpm. Not only is this a pain to setup, but FastCGI is not very fast.

## Solution

Embed PHP into Rust so that hyper can accept the HTTP request, spawn a thread in which we pass that information off to a PHP script and then have hyper return the HTTP response.

## Installation

This package depends on PHP. The default location for the PHP includes is in `/usr/include/php`. You can set an environment variable `PHP_INCLUDE_DIR` to override this. The default location for `libphp7` is in `/usr/lib`. You can set an environment variable `PHP_LIB_DIR` to override this. For details on how to install or compile PHP, see [php-sys/README.md](php-sys/README.md). Please note that in order to safely spawn threads with PHP, `--enable-maintainer-zts` must be used.

The code uses bindgen to dynamically build bindings for PHP 7. If you want to compile against a static `libphp7`, then specify `PHP_LINK_STATIC=1`. Using `cargo build` should give you a working binary.

## Usage

Depending on the location of `libphp7` you may need to provide `LD_LIBRARY_PATH`. The first argument to the program is the document root. Currently the index is hardcoded to `index.php`. Example: `PHP_LIB_DIR="/path/to/lib" PHP_INCLUDE_DIR="/path/to/include" LD_LIBRARY_PATH="/path/to/lib" cargo run -- tests/`. This will send requests to a script in `./tests/index.php`.

The `tests/index.php` is the entry point of the PHP program. A hyper server listens for new requests and dispatches each request to the PHP engine, which executes `tests/index.php`. The response headers and body are sent back through hyper and then to the client.

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
