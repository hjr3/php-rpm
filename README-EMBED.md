Some basic instructions on how to install PHP so you can embed it into Rust.

Note: I embed a static library. If you want to do embed PHP with a shared library, then use `--enable-embed=shared`.

### Mac OS X

I had to use brew to install bison. I believe autoconf and other tools were either already installed or provided by Mac OS X. Brew installed some modified version of libiconv which confused PHP. I also had some problems, so I stopped building xml related stuff. To build I had to do:

```
$ ./genfiles
$ ./buildconf --force
$ PATH="/usr/local/opt/bison/bin:$PATH" ./configure --enable-debug --enable-embed=static --without-iconv
$ PATH="/usr/local/opt/bison/bin:$PATH" make
$ PATH="/usr/local/opt/bison/bin:$PATH" make test
```

### Linux

```
$ ./genfiles
$ ./buildconf --force
$ PATH="/usr/local/opt/bison/bin:$PATH" ./configure --enable-debug --enable-embed=static
$ PATH="/usr/local/opt/bison/bin:$PATH" make
$ PATH="/usr/local/opt/bison/bin:$PATH" make test
```
