# Changelog

## Unreleased

- Time series: `joinAsof` (backward, forward, and nearest, with `by` groups and tolerances), `groupByDynamic`, `rolling` groups, `upsample`, the `rolling*By` expressions, `rollingQuantile`, `rollingSkew`, `rollingKurtosis`, and the `ewmMean`, `ewmStd`, `ewmVar`, and `ewmMeanBy` expressions.
- Functions: `date_range`, `date_ranges`, `datetime_range`, `datetime_ranges`, `int_ranges`, `date`, `datetime`, `duration`, `from_epoch`, `arg_where`, `corr`, `cov`, `nth`, `repeat`, `linear_space`, `select`, `std`, `var_`, `quantile`, `implode`, and `approx_n_unique`. `var_` has a trailing underscore because PHP reserves `var`.
- Frame verbs: `shift`, `interpolate`, `quantile`, `product`, `approxNUnique`, `topK`, `bottomK`, `gatherEvery`, `dropNans`, `selectSeq`, and `withColumnsSeq`. DataFrame adds `isDuplicated`, `isUnique`, `partitionBy`, `toStruct`, `corr`, and `insertColumn`, and LazyFrame adds `describe`.
- Expressions: `cut`, `qcut`, `rle`, `rleId`, `searchSorted`, `peakMax`, `peakMin`, `entropy`, `hist`, `interpolateBy`, `isClose`, `argUnique`, `indexOf`, `upperBound`, `lowerBound`, `toPhysical`, `limit`, `exclude`, and the hyperbolic and degree functions.
- Namespaces: `str` adds `containsAny`, `replaceMany`, `extractMany`, `findMany`, `extractGroups`, `splitExact`, `encode`, `decode`, `toDecimal`, `normalize`, `jsonDecode`, `jsonPathMatch`, and `escapeRegex`. `dt` adds `addBusinessDays`, `isBusinessDay`, `combine`, `replace`, `withTimeUnit`, `millennium`, `century`, `daysInMonth`, `baseUtcOffset`, `dstOffset`, and `totalNanoseconds`. `list` adds `any`, `all`, `filter`, `sample`, `gatherEvery`, `toArray`, and `agg`. New `cat`, `arr`, and `bin` namespaces cover Categorical, Array, and Binary columns.
- `Series` runs any `Expr` method it does not define itself, so every expression method works on a Series. Reductions such as `skew()` return a PHP value.
- Data type strings accept `Array(Int64, 3)`, `Struct(a: Int64, b: String)`, `Int128`, and `UInt128`.
- Lists of dates and datetimes convert to PHP `DateTimeImmutable` values; some list results had come back as raw integers.
- 244 cases are compared against py-polars 1.43.2, and all match.

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
