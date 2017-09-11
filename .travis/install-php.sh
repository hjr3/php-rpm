#!/bin/bash

set -e

if [ ! -f "$HOME/php-src/libs/libphp7.so" ]; then
  cd $HOME
  rm -fr php-src
  curl -o php-7.1.8.tar.gz http://us1.php.net/distributions/php-7.1.8.tar.gz
  tar xzf php-7.1.8.tar.gz
  mv php-7.1.8 php-src
  cd php-src
  sed -e 's/void zend_signal_startup/ZEND_API void zend_signal_startup/g' -ibk Zend/zend_signal.c Zend/zend_signal.h
  ./configure --enable-debug --enable-embed=shared --enable-maintainer-zts
  make
  cd $HOME
else
  echo 'Using cached directory.'
fi
