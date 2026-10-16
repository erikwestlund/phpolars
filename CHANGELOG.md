# Changelog

## 0.1.0

First release of phpolars: Polars 0.55.2 data frames for PHP 8.5 as a native extension, following the py-polars API.

- `DataFrame` and `LazyFrame` with py-polars' verbs: `select`, `withColumns`, `filter`, `sort`, `groupBy`/`agg`, `join` (inner, left, right, full, semi, anti, cross, with `nullsEqual` and `maintainOrder`), `joinWhere`, `unique`, `pivot`, `unpivot`, `explode`, `unnest`, `concat`, `describe`, `sample`, `transpose`, and frame aggregates.
- Expressions with arithmetic, three-valued logic, aggregations, `over()` windows with `orderBy`, cumulative and rolling functions, ranks, `when`/`then`/`otherwise`, and the `str`, `dt`, `list`, `struct`, and `name` namespaces. `Series` shares the expression methods.
- CSV, Parquet, JSON, NDJSON, and Arrow IPC reading, lazy scans, writing, and streaming sinks. Parquet, JSON, IPC, and table printing are Cargo features.
- py-polars data type names and exception classes, and a value mapping to PHP (dates as `DateTimeImmutable`, lists, structs).
- `php_polars.max_threads` caps the Polars thread pool per PHP-FPM worker, applied at module startup.
- 169 cases are compared against py-polars 1.43.2, and all match.
- The Polars backend for phibble and phplyr, `Phibble\Polars\PolarsTable`, was developed here and ships in phibble v0.2.0. It runs phplyr pipelines in Polars with dplyr semantics and passes phibble's dplyr, tidyr, stringr, and lubridate reference fixtures.
- PIE packaging, a Docker install-and-FPM check, and prebuilt Linux binaries (amd64 and arm64, PHP 8.5 NTS, Debian Bookworm) with dependency notices.
- Documentation: https://erikwestlund.github.io/phpolars/
