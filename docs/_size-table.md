| Build | opt-level | Size | Benchmark, 1 / 4 threads |
| --- | --- | --- | --- |
| all features, `release` profile (no LTO) | 3 | 117 MB | |
| all features | 3 | 57.3 MB | 44 / 23 ms |
| all features (`dist` default) | s | 40.2 MB | 45 / 21 ms |
| all features | z | 23.9 MB | 86 / 33 ms |
| without `fmt` | 3 | 57.2 MB | |
| without `json` | 3 | 56.8 MB | |
| without `ipc` | 3 | 56.9 MB | |
| without `parquet` and `ipc` | 3 | 52.4 MB | |
| CSV only (no optional features) | 3 | 51.8 MB | |
| CSV only (no optional features) | s | 35.9 MB | |
The optional features cost little: all of them together add about 5 MB. Most of the size is the Polars query engine, expression set, and data types that the core API needs. The `dist` profile uses `opt-level = "s"`, which is 30% smaller than `opt-level = 3` and as fast on the benchmark above (the `read_csv` → `filter` → `groupBy` pipeline on 1M rows). `opt-level = "z"` halves the size again but doubles single-threaded time. A clean `dist` build takes 4–5 minutes on an Apple M5 Ultra.
