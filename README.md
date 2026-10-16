# phpolars

phpolars brings [Polars](https://pola.rs) data frames to PHP as a native extension written in Rust with [ext-php-rs](https://github.com/davidcole1340/ext-php-rs). The API follows py-polars: the same classes and methods, with camelCase method names and snake_case functions.

```php
use Phpolars\DataFrame;
use function Phpolars\{col, lit, when};

$df = Phpolars\read_csv('sales.csv');

$summary = $df
    ->filter(col('amount')->gt(0))
    ->withColumns(when(col('region')->isIn(['N', 'S']))->then(lit('core'))->otherwise(lit('other'))->alias('tier'))
    ->groupBy(['tier', 'region'], maintainOrder: true)
    ->agg(col('amount')->sum()->alias('total'), ['orders' => Phpolars\len()])
    ->sort('total', descending: true);

print_r($summary->toDicts());
```

phpolars is part of a family of PHP 8.5 packages for data work. [phibble](https://github.com/erikwestlund/phibble) is a pure-PHP data frame, [phplyr](https://github.com/erikwestlund/phplyr) provides dplyr verbs over phibble tables, and phpolars is the optional fast backend. `DataFrame::fromPhibble()` and `toPhibble()` move data between the two.

## Requirements

- PHP 8.5, non-thread-safe (NTS) build, on Linux or macOS
- A stable Rust toolchain and libclang to build

## Installing

With [PIE](https://github.com/php/pie):

```sh
pie install erikwestlund/phpolars
```

From a checkout, build with Cargo and load the library directly:

```sh
PHP=$(which php) PHP_CONFIG=$(which php-config) cargo build --release
php -d extension=$PWD/target/release/libphp_polars.so script.php   # .dylib on macOS
```

On macOS with Homebrew, set `LIBCLANG_PATH=$(brew --prefix llvm)/lib`. If several PHP versions are installed, point `PHP` and `PHP_CONFIG` at the PHP 8.5 binaries, because ext-php-rs builds against the headers of that PHP.

## API conventions

- Methods use the py-polars name in camelCase: `with_columns` is `withColumns`, `group_by` is `groupBy`, and `to_dicts` is `toDicts`.
- Functions use the py-polars name in snake_case in the `Phpolars` namespace: `col`, `lit`, `when`, `concat`, `read_csv`, `scan_parquet`, and `sum_horizontal`.
- Named parameters use camelCase: `join($other, leftOn: 'id', rightOn: 'key', how: 'left')`.
- Where py-polars takes `*exprs`, the method is variadic. Each argument is a column name, an `Expr`, a list of those, or an associative array `['alias' => Expr]` in place of py's `**named_exprs`.
- PHP does not allow parameters after a variadic one. Methods that need both take a single argument that accepts a name, an `Expr`, or a list: `groupBy(['a', 'b'], maintainOrder: true)` and `over('g', orderBy: 'x')`.
- Expression namespaces are methods: `col('s')->str()->contains('x')`, `col('t')->dt()->year()`, `col('v')->list()->len()`, and `col('p')->struct()->field('x')`. The same namespaces exist on `Series` and return a `Series`.
- Strings in comparisons and arithmetic are literals (`col('g')->eq('a')`). Strings passed where py expects an expression input, such as `select()`, `then()`, and `groupBy()`, name columns. Use `lit('text')` for a string value in those places.
- Data types use the py-polars names as strings: `'Int64'`, `'Float64'`, `'String'`, `'Date'`, `'Datetime(ms, UTC)'`, and `'List(Int64)'`. `Phpolars\DataType` has the same names as constants, such as `DataType::Int64`.
- Defaults follow py-polars. `groupBy()` and `unique()` do not keep row order unless `maintainOrder: true` is passed.

## Values

Null is PHP `null`. Values convert as follows.

| Polars | PHP |
| --- | --- |
| integer types | `int` (UInt64 values beyond `PHP_INT_MAX` become `float`) |
| Float32, Float64 | `float` |
| Boolean | `bool` |
| String, Categorical | `string` |
| Date, Datetime | `DateTimeImmutable` (naive values in UTC, zoned values in their zone) |
| Time | `string`, `"HH:MM:SS.ffffff"` |
| Duration | `int` in the column's time unit |
| Decimal | `string` |
| List, Array | list |
| Struct | associative array |

From PHP, each column gets one inferred type. Integers widen to floats, nulls fit any type, `DateTimeInterface` values become naive `Datetime(us)` on their wall-clock time (the zone is not kept; use `dt()->replaceTimeZone()` to attach one), lists become `List`, and associative arrays become `Struct`. Any other mix of types raises `SchemaError`. Pass a `schema` to set types explicitly.

## Errors

Polars errors are thrown as subclasses of `Phpolars\PolarsError` named after the py-polars exceptions in `Phpolars\Exception`: `ColumnNotFoundError`, `ComputeError`, `DuplicateError`, `InvalidOperationError`, `NoDataError`, `OutOfBoundsError`, `SchemaError`, `SchemaFieldNotFoundError`, `ShapeError`, and `StructFieldNotFoundError`. Invalid arguments from PHP raise `ValueError`.

## PHP-FPM and threads

Polars runs queries on a thread pool. PHP-FPM workers are separate processes, so a pool in every worker multiplies quickly. The INI setting `php_polars.max_threads` caps the pool size per process. It defaults to 2 and can only be set in `php.ini`, because the pool starts once per process. An explicit `POLARS_MAX_THREADS` environment variable takes precedence. `Phpolars\thread_pool_size()` reports the size in effect.

PHP closures never run on Polars threads. Every phpolars operation is built from expressions that Polars evaluates natively.

Polars allocates memory with its own allocator, so that memory does not count against PHP's `memory_limit`. Reading a 1M-row CSV raises the process's resident memory to about 83 MB while `memory_get_peak_usage()` reports 2 MB. Size FPM pools by process memory, not by `memory_limit`.

## Performance

The benchmark in `bench/pipeline.php` reads a CSV, filters it, groups and aggregates it, and returns PHP arrays. It runs each implementation five times and reports the fastest run. On a 1M-row, 16 MB CSV on an Apple M5 Ultra:

| Implementation | Time |
| --- | --- |
| phpolars, 1 thread | 49 ms |
| phpolars, 2 threads (default) | 27 ms |
| phpolars, 4 threads | 17 ms |
| plain PHP arrays (`fgetcsv` loop) | 342 ms |

All implementations produce the same result. Run it with:

```sh
php bench/make-csv.php 1000000 bench/data.csv
php -d extension=... bench/pipeline.php bench/data.csv [path/to/phibble/vendor/autoload.php]
```

## Binary size

Polars is large. Release builds use the `dist` profile, which applies fat LTO, one codegen unit, and symbol stripping. Optional Cargo features gate the formats and data that not every installation needs:

| Feature | Provides |
| --- | --- |
| `parquet` | `read_parquet`, `scan_parquet`, `writeParquet`, `sinkParquet` |
| `json` | `read_json`, `read_ndjson`, `writeJson`, `writeNdjson` |
| `ipc` | Arrow IPC (Feather) reading and writing; also enables `parquet`, which Polars 0.55's IPC reader needs |
| `fmt` | the table layout used by `__toString()` |
| `sql` | `SQLContext`, `DataFrame::sql()`, `LazyFrame::sql()`, and `Phpolars\sql()` |

All features are on by default. Build a smaller extension with, for example, `cargo build --profile dist --no-default-features --features parquet`.

Sizes of the stripped macOS arm64 library with fat LTO and one codegen unit:

<!-- size-table -->
| Build | opt-level | Size | Benchmark, 1 / 4 threads |
| --- | --- | --- | --- |
| all features (`dist` default) | s | 64.5 MB | 47 / 23 ms |
| without `sql` | s | 60.4 MB | |
| v0.1.0, all features, `release` profile (no LTO) | 3 | 117 MB | |
| v0.1.0, all features | 3 | 57.3 MB | 44 / 23 ms |
| v0.1.0, all features (`dist` default) | s | 40.2 MB | 45 / 21 ms |
| v0.1.0, all features | z | 23.9 MB | 86 / 33 ms |
| v0.1.0, without `fmt` | 3 | 57.2 MB | |
| v0.1.0, without `json` | 3 | 56.8 MB | |
| v0.1.0, without `ipc` | 3 | 56.9 MB | |
| v0.1.0, without `parquet` and `ipc` | 3 | 52.4 MB | |
| v0.1.0, CSV only (no optional features) | 3 | 51.8 MB | |
| v0.1.0, CSV only (no optional features) | s | 35.9 MB | |
<!-- /size-table -->

The rows marked v0.1.0 were measured before the py-polars API was filled in: time series windows and asof joins, the extra string, binary, and array kernels, and SQL. Those additions take the default build from 40.2 MB to 64.5 MB; SQL is 4.1 MB of that and can be left out with `--no-default-features --features parquet,json,ipc,fmt`. The other optional features cost little: at v0.1.0 they added about 5 MB together. Most of the size is the Polars query engine, expression set, and data types that the core API needs. The `dist` profile uses `opt-level = "s"`, which is 30% smaller than `opt-level = 3` and as fast on the benchmark above (the `read_csv` → `filter` → `groupBy` pipeline on 1M rows). `opt-level = "z"` halves the size again but doubles single-threaded time. A clean `dist` build takes 4–5 minutes on an Apple M5 Ultra.

## Development

```sh
scripts/test                       # build and run the test files
PHIBBLE=../phibble/vendor/autoload.php scripts/test   # also round-trip through phibble
php -d extension=... scripts/stubs.php > stubs/phpolars.php   # regenerate IDE stubs
```

`stubs/phpolars.php` holds signatures for IDEs and static analysis. It is generated from the extension's reflection data.

## License

MIT
