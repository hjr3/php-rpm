# php-sys

Bindings to php.

Note: I have only tested this without ZTS.

## PHP

In order to compile php-sys, we need development headers and the libphp7 library. That library may come in the form of `libphp7.so` or `libphp7.a` depending on how you install/compile PHP.

### From Package

   * For Ubuntu, please refer to the [.travis.yml](../.travis.yml) _install_ section for the commands.
   * For Mac OS X, I could not find a set of packages that worked.

### From Source

Some basic instructions on how to install PHP so you can embed it into Rust.

#### Mac OS X

I had to use brew to install bison. I believe autoconf and other tools were either already installed or provided by Mac OS X. Brew installed some modified version of libiconv which confused PHP. I also had some problems, so I stopped building xml related stuff. To build I had to do:

```
$ ./genfiles
$ ./buildconf --force
$ PATH="/usr/local/opt/bison/bin:$PATH" ./configure --enable-debug --enable-embed=static --without-iconv --disable-libxml --disable-dom --disable-xml --disable-simplexml --disable-xmlwriter --disable-xmlreader --without-pear
$ PATH="/usr/local/opt/bison/bin:$PATH" make
$ PATH="/usr/local/opt/bison/bin:$PATH" make test
```

Note: I embed a static library on Mac OS X. If you want to do embed PHP with a shared library, then use `--enable-embed=shared`.

#### Linux

Here are the dependencies needed (in apt-get form):

```bash
$ apt-get install git make gcc libxml2-dev autoconf bison valgrind clang re2c
```

```
$ ./genfiles
$ ./buildconf --force
$ ./configure --enable-debug --enable-embed=shared
$ make
$ make test
```
