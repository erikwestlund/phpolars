### `Phpolars\DataFrame`

```php
__toString(): string
cast(mixed $dtypes, ?bool $strict = null): Phpolars\DataFrame
clone(): Phpolars\DataFrame
collectSchema(): array
columns(): array
count(): Phpolars\DataFrame
describe(?array $percentiles = null): Phpolars\DataFrame
drop(mixed ...$columns): Phpolars\DataFrame
dropNulls(mixed $subset = null): Phpolars\DataFrame
dtypes(): array
equals(Phpolars\DataFrame $other, ?bool $nullEqual = null): bool
estimatedSize(): int
explode(mixed ...$columns): Phpolars\DataFrame
fillNan(mixed $value): Phpolars\DataFrame
fillNull(mixed $value = null, ?string $strategy = null, ?int $limit = null): Phpolars\DataFrame
filter(mixed ...$predicates): Phpolars\DataFrame
first(): Phpolars\DataFrame
getColumn(string $name): Phpolars\Series
getColumns(): array
groupBy(mixed $by, ?bool $maintainOrder = null): Phpolars\GroupBy
head(?int $n = null): Phpolars\DataFrame
height(): int
hstack(mixed $columns): Phpolars\DataFrame
isEmpty(): bool
item(?int $row = null, mixed $column = null): mixed
join(mixed $other, mixed $on = null, ?string $how = null, mixed $leftOn = null, mixed $rightOn = null, ?string $suffix = null, ?bool $coalesce = null, ?string $maintainOrder = null, ?bool $nullsEqual = null): Phpolars\DataFrame
joinWhere(mixed $other, mixed ...$predicates): Phpolars\DataFrame
last(): Phpolars\DataFrame
lazy(): Phpolars\LazyFrame
limit(?int $n = null): Phpolars\DataFrame
max(): Phpolars\DataFrame
mean(): Phpolars\DataFrame
median(): Phpolars\DataFrame
min(): Phpolars\DataFrame
nUnique(): Phpolars\DataFrame
nullCount(): Phpolars\DataFrame
pivot(mixed $on, mixed $index = null, mixed $values = null, mixed $aggregateFunction = null, ?bool $sortColumns = null, ?string $separator = null, ?bool $maintainOrder = null): Phpolars\DataFrame
rename(array $mapping, ?bool $strict = null): Phpolars\DataFrame
reverse(): Phpolars\DataFrame
row(int $index, ?bool $named = null): mixed
rows(?bool $named = null): array
sample(?int $n = null, ?float $fraction = null, ?bool $withReplacement = null, ?bool $shuffle = null, ?int $seed = null): Phpolars\DataFrame
schema(): array
select(mixed ...$exprs): Phpolars\DataFrame
shape(): array
slice(int $offset, ?int $length = null): Phpolars\DataFrame
sort(mixed $by, mixed $descending = null, mixed $nullsLast = null, ?bool $maintainOrder = null): Phpolars\DataFrame
static fromDict(array $data, ?array $schema = null): Phpolars\DataFrame
static fromDicts(array $data, ?array $schema = null): Phpolars\DataFrame
static fromPhibble(mixed $table): Phpolars\DataFrame
static fromRows(array $data, ?array $schema = null): Phpolars\DataFrame
std(?int $ddof = null): Phpolars\DataFrame
sum(): Phpolars\DataFrame
tail(?int $n = null): Phpolars\DataFrame
toColumns(): array
toDict(?bool $asSeries = null): array
toDicts(): array
toPhibble(): mixed
toSeries(?int $index = null): Phpolars\Series
transpose(?bool $includeHeader = null, ?string $headerName = null, mixed $columnNames = null): Phpolars\DataFrame
unique(mixed $subset = null, ?string $keep = null, ?bool $maintainOrder = null): Phpolars\DataFrame
unnest(mixed ...$columns): Phpolars\DataFrame
unpivot(mixed $on = null, mixed $index = null, ?string $variableName = null, ?string $valueName = null): Phpolars\DataFrame
var(?int $ddof = null): Phpolars\DataFrame
vstack(Phpolars\DataFrame $other): Phpolars\DataFrame
width(): int
withColumns(mixed ...$exprs): Phpolars\DataFrame
withRowIndex(?string $name = null, ?int $offset = null): Phpolars\DataFrame
writeCsv(string $file, ?bool $includeHeader = null, ?string $separator = null, ?string $nullValue = null): void
writeIpc(string $file): void
writeJson(string $file): void
writeNdjson(string $file): void
writeParquet(string $file): void
```

### `Phpolars\LazyFrame`

```php
__toString(): string
cache(): Phpolars\LazyFrame
cast(mixed $dtypes, ?bool $strict = null): Phpolars\LazyFrame
collect(): Phpolars\DataFrame
collectSchema(): array
columns(): array
count(): Phpolars\LazyFrame
drop(mixed ...$columns): Phpolars\LazyFrame
dropNulls(mixed $subset = null): Phpolars\LazyFrame
explain(?bool $optimized = null): string
explode(mixed ...$columns): Phpolars\LazyFrame
fetch(?int $n = null): Phpolars\DataFrame
fillNan(mixed $value): Phpolars\LazyFrame
fillNull(mixed $value = null, ?string $strategy = null, ?int $limit = null): Phpolars\LazyFrame
filter(mixed ...$predicates): Phpolars\LazyFrame
first(): Phpolars\LazyFrame
groupBy(mixed $by, ?bool $maintainOrder = null): Phpolars\LazyGroupBy
head(?int $n = null): Phpolars\LazyFrame
join(mixed $other, mixed $on = null, ?string $how = null, mixed $leftOn = null, mixed $rightOn = null, ?string $suffix = null, ?bool $coalesce = null, ?string $maintainOrder = null, ?bool $nullsEqual = null): Phpolars\LazyFrame
joinWhere(mixed $other, mixed ...$predicates): Phpolars\LazyFrame
last(): Phpolars\LazyFrame
limit(?int $n = null): Phpolars\LazyFrame
max(): Phpolars\LazyFrame
mean(): Phpolars\LazyFrame
median(): Phpolars\LazyFrame
min(): Phpolars\LazyFrame
nUnique(): Phpolars\LazyFrame
nullCount(): Phpolars\LazyFrame
rename(array $mapping, ?bool $strict = null): Phpolars\LazyFrame
reverse(): Phpolars\LazyFrame
select(mixed ...$exprs): Phpolars\LazyFrame
sinkCsv(string $file, ?bool $includeHeader = null, ?string $separator = null, ?string $nullValue = null): void
sinkIpc(string $file): void
sinkNdjson(string $file): void
sinkParquet(string $file): void
slice(int $offset, ?int $length = null): Phpolars\LazyFrame
sort(mixed $by, mixed $descending = null, mixed $nullsLast = null, ?bool $maintainOrder = null): Phpolars\LazyFrame
std(?int $ddof = null): Phpolars\LazyFrame
sum(): Phpolars\LazyFrame
tail(?int $n = null): Phpolars\LazyFrame
unique(mixed $subset = null, ?string $keep = null, ?bool $maintainOrder = null): Phpolars\LazyFrame
unnest(mixed ...$columns): Phpolars\LazyFrame
unpivot(mixed $on = null, mixed $index = null, ?string $variableName = null, ?string $valueName = null): Phpolars\LazyFrame
var(?int $ddof = null): Phpolars\LazyFrame
withColumns(mixed ...$exprs): Phpolars\LazyFrame
withRowIndex(?string $name = null, ?int $offset = null): Phpolars\LazyFrame
```

### `Phpolars\GroupBy`

```php
agg(mixed ...$aggs): Phpolars\DataFrame
all(): Phpolars\DataFrame
count(): Phpolars\DataFrame
first(): Phpolars\DataFrame
head(?int $n = null): Phpolars\DataFrame
last(): Phpolars\DataFrame
len(?string $name = null): Phpolars\DataFrame
max(): Phpolars\DataFrame
mean(): Phpolars\DataFrame
median(): Phpolars\DataFrame
min(): Phpolars\DataFrame
nUnique(): Phpolars\DataFrame
sum(): Phpolars\DataFrame
tail(?int $n = null): Phpolars\DataFrame
```

### `Phpolars\LazyGroupBy`

```php
agg(mixed ...$aggs): Phpolars\LazyFrame
all(): Phpolars\LazyFrame
count(): Phpolars\LazyFrame
first(): Phpolars\LazyFrame
head(?int $n = null): Phpolars\LazyFrame
last(): Phpolars\LazyFrame
len(?string $name = null): Phpolars\LazyFrame
max(): Phpolars\LazyFrame
mean(): Phpolars\LazyFrame
median(): Phpolars\LazyFrame
min(): Phpolars\LazyFrame
nUnique(): Phpolars\LazyFrame
sum(): Phpolars\LazyFrame
tail(?int $n = null): Phpolars\LazyFrame
```

