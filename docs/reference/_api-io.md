```php
read_csv(string $source, ?bool $hasHeader = null, ?string $separator = null, mixed $nullValues = null, ?array $schemaOverrides = null, ?int $nRows = null, ?int $skipRows = null, ?int $inferSchemaLength = null, ?bool $tryParseDates = null, mixed $columns = null): Phpolars\DataFrame
read_ipc(string $source, ?int $nRows = null): Phpolars\DataFrame
read_json(string $source): Phpolars\DataFrame
read_ndjson(string $source, ?int $nRows = null, ?int $inferSchemaLength = null): Phpolars\DataFrame
read_parquet(string $source, mixed $columns = null, ?int $nRows = null): Phpolars\DataFrame
scan_csv(string $source, ?bool $hasHeader = null, ?string $separator = null, mixed $nullValues = null, ?array $schemaOverrides = null, ?int $nRows = null, ?int $skipRows = null, ?int $inferSchemaLength = null, ?bool $tryParseDates = null): Phpolars\LazyFrame
scan_ipc(string $source, ?int $nRows = null): Phpolars\LazyFrame
scan_ndjson(string $source, ?int $nRows = null, ?int $inferSchemaLength = null): Phpolars\LazyFrame
scan_parquet(string $source, ?int $nRows = null): Phpolars\LazyFrame
```
