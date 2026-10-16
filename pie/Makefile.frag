# Use the workspace target so direct Cargo and PIE builds can share compilation.
POLARS_CARGO_TARGET_DIR = $(top_srcdir)/../target
POLARS_ARTIFACT = $(POLARS_CARGO_TARGET_DIR)/release/libphp_polars.$(POLARS_DYLIB_SUFFIX)
POLARS_MODULE = $(top_builddir)/modules/php_polars.so

.PHONY: polars-rust-build polars-artifact-check install-polars-rust
all: polars-rust-build

# make install may inherit an `all` prerequisite from phpize. Never compile as
# root: require an artifact already built by the unprivileged make step.
ifneq ($(filter install install-modules install-polars-rust,$(MAKECMDGOALS)),)
polars-rust-build: polars-artifact-check
else
polars-rust-build:
	PHP="$(POLARS_PHP)" PHP_CONFIG="$(PHP_CONFIG)" CARGO_TARGET_DIR="$(POLARS_CARGO_TARGET_DIR)" "$(CARGO)" build --manifest-path "$(top_srcdir)/../Cargo.toml" --lib --release --locked
	@mkdir -p "$(top_builddir)/modules"
	cp "$(POLARS_ARTIFACT)" "$(POLARS_MODULE)"
endif

polars-artifact-check:
	@test -f "$(POLARS_MODULE)" || { echo 'Build the extension with make before running make install.' >&2; exit 1; }

install: install-polars-rust
install-polars-rust: polars-artifact-check
	@mkdir -p "$(INSTALL_ROOT)$(EXTENSION_DIR)"
	$(INSTALL) -m 755 "$(POLARS_MODULE)" "$(INSTALL_ROOT)$(EXTENSION_DIR)/php_polars.so"
