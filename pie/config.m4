dnl PIE uses phpize/configure/make; the extension itself is compiled by Cargo.
PHP_ARG_ENABLE([php-polars], [whether to enable Polars data frames],
  [AS_HELP_STRING([--enable-php-polars], [Build the Rust php_polars extension])], [yes])

if test "$PHP_PHP_POLARS" != "no"; then
  AC_PATH_PROG([CARGO], [cargo], [])
  if test -z "$CARGO"; then
    AC_MSG_ERROR([cargo is required; install a stable Rust toolchain first])
  fi
  if test "$PHP_THREAD_SAFETY" = "yes"; then
    AC_MSG_ERROR([php_polars supports non-thread-safe PHP only])
  fi
  POLARS_PHP_VERSION=`$PHP_CONFIG --vernum`
  if test "$POLARS_PHP_VERSION" -lt 80500 || test "$POLARS_PHP_VERSION" -ge 80600; then
    AC_MSG_ERROR([php_polars targets PHP 8.5])
  fi
  POLARS_PHP=`$PHP_CONFIG --php-binary`
  AS_CASE([$host_os],
    [linux*], [POLARS_DYLIB_SUFFIX=so],
    [darwin*], [POLARS_DYLIB_SUFFIX=dylib],
    [AC_MSG_ERROR([php_polars supports Linux and macOS only])])
  PHP_SUBST([CARGO])
  PHP_SUBST([POLARS_PHP])
  PHP_SUBST([POLARS_DYLIB_SUFFIX])
  PHP_ADD_MAKEFILE_FRAGMENT([$srcdir/Makefile.frag])
fi
