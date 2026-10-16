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
