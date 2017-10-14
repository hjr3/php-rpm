#!/bin/bash

set -e

VERSION=$1

if [ ! -f "$HOME/$VERSION/libs/libphp7.so" ]; then
  cd $HOME
  rm -fr $VERSION
  curl -o $VERSION.tar.gz http://php.net/distributions/$VERSION.tar.gz
  tar xzf $VERSION.tar.gz
  cd $VERSION
  sed -e 's/void zend_signal_startup/ZEND_API void zend_signal_startup/g' -ibk Zend/zend_signal.c Zend/zend_signal.h
  ./configure --enable-debug --enable-embed=shared --enable-maintainer-zts
  make
  cd $HOME
else
  echo 'Using cached directory.'
fi
