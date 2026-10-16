# Changelog

## 0.1.0

First release of phpolars: Polars 0.55.2 data frames for PHP 8.5 as a native extension, following the py-polars API.

- `DataFrame` and `LazyFrame` with py-polars' verbs: `select`, `withColumns`, `filter`, `sort`, `groupBy`/`agg`, `join` (inner, left, right, full, semi, anti, cross, with `nullsEqual` and `maintainOrder`), `joinWhere`, `unique`, `pivot`, `unpivot`, `explode`, `unnest`, `concat`, `describe`, `sample`, `transpose`, and frame aggregates.
- Expressions with arithmetic, three-valued logic, aggregations, `over()` windows with `orderBy`, cumulative and rolling functions, ranks, `when`/`then`/`otherwise`, and the `str`, `dt`, `list`, `struct`, and `name` namespaces. `Series` shares the expression methods.
- CSV, Parquet, JSON, NDJSON, and Arrow IPC reading, lazy scans, writing, and streaming sinks. Parquet, JSON, IPC, and table printing are Cargo features.
- py-polars data type names and exception classes, and a value mapping to PHP (dates as `DateTimeImmutable`, lists, structs).
- `php_polars.max_threads` caps the Polars thread pool per PHP-FPM worker.
- 169 cases are compared against py-polars 1.43.2, and all match.
- `erikwestlund/phpolars-phibble` (the `phibble/` directory): `PolarsTable`, a phibble `Table` backed by Polars, so phplyr pipelines run in Polars with dplyr semantics, including `join_by()` inequality and rolling joins, tidyr pivot specs, and the stringr and lubridate expression functions. Closures fall back to phibble's in-memory `Frame`. It passes phibble's dplyr, tidyr, stringr, and lubridate reference fixtures.
- PIE packaging, a Docker install-and-FPM check, and prebuilt Linux binaries (amd64 and arm64, PHP 8.5 NTS, Debian Bookworm) with dependency notices.
- Documentation: https://erikwestlund.github.io/phpolars/
