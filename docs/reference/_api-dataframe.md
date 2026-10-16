### `Phpolars\DataFrame`

```php
__toString(): string
approxNUnique(): Phpolars\DataFrame
bottomK(int $k, mixed $by, mixed $reverse = null): Phpolars\DataFrame
cast(mixed $dtypes, ?bool $strict = null): Phpolars\DataFrame
clear(?int $n = null): Phpolars\DataFrame
clone(): Phpolars\DataFrame
collectSchema(): array
columns(): array
corr(): Phpolars\DataFrame
count(): Phpolars\DataFrame
describe(?array $percentiles = null): Phpolars\DataFrame
drop(mixed ...$columns): Phpolars\DataFrame
dropNans(mixed $subset = null): Phpolars\DataFrame
dropNulls(mixed $subset = null): Phpolars\DataFrame
dtypes(): array
equals(Phpolars\DataFrame $other, ?bool $nullEqual = null): bool
estimatedSize(): int
explode(mixed ...$columns): Phpolars\DataFrame
extend(Phpolars\DataFrame $other): Phpolars\DataFrame
fillNan(mixed $value): Phpolars\DataFrame
fillNull(mixed $value = null, ?string $strategy = null, ?int $limit = null): Phpolars\DataFrame
filter(mixed ...$predicates): Phpolars\DataFrame
first(): Phpolars\DataFrame
gather(mixed $indices): Phpolars\DataFrame
gatherEvery(int $n, ?int $offset = null): Phpolars\DataFrame
getColumn(string $name): Phpolars\Series
getColumnIndex(string $name): int
getColumns(): array
glimpse(?int $maxItemsPerColumn = null): string
groupBy(mixed $by, ?bool $maintainOrder = null): Phpolars\GroupBy
groupByDynamic(mixed $indexColumn, string $every, ?string $period = null, ?string $offset = null, ?bool $includeBoundaries = null, ?string $closed = null, ?string $label = null, mixed $groupBy = null, ?string $startBy = null): Phpolars\GroupBy
head(?int $n = null): Phpolars\DataFrame
height(): int
hstack(mixed $columns): Phpolars\DataFrame
insertColumn(int $index, Phpolars\Series $column): Phpolars\DataFrame
interpolate(): Phpolars\DataFrame
isDuplicated(): Phpolars\Series
isEmpty(): bool
isUnique(): Phpolars\Series
item(?int $row = null, mixed $column = null): mixed
iterRows(?bool $named = null): array
iterSlices(?int $nRows = null): array
join(mixed $other, mixed $on = null, ?string $how = null, mixed $leftOn = null, mixed $rightOn = null, ?string $suffix = null, ?bool $coalesce = null, ?string $maintainOrder = null, ?bool $nullsEqual = null): Phpolars\DataFrame
joinAsof(mixed $other, mixed $on = null, mixed $leftOn = null, mixed $rightOn = null, mixed $by = null, mixed $byLeft = null, mixed $byRight = null, ?string $strategy = null, ?string $suffix = null, mixed $tolerance = null, ?bool $allowExactMatches = null, ?bool $checkSortedness = null, ?bool $coalesce = null): Phpolars\DataFrame
joinWhere(mixed $other, mixed ...$predicates): Phpolars\DataFrame
last(): Phpolars\DataFrame
lazy(): Phpolars\LazyFrame
limit(?int $n = null): Phpolars\DataFrame
mapRows(mixed $function): Phpolars\DataFrame
max(): Phpolars\DataFrame
maxHorizontal(): Phpolars\Series
mean(): Phpolars\DataFrame
meanHorizontal(?bool $ignoreNulls = null): Phpolars\Series
median(): Phpolars\DataFrame
mergeSorted(mixed $other, mixed $key, ?bool $maintainOrder = null): Phpolars\DataFrame
min(): Phpolars\DataFrame
minHorizontal(): Phpolars\Series
nChunks(): int
nUnique(): Phpolars\DataFrame
nullCount(): Phpolars\DataFrame
partitionBy(mixed $by, ?bool $maintainOrder = null, ?bool $includeKey = null, ?bool $asDict = null): array
pivot(mixed $on, mixed $index = null, mixed $values = null, mixed $aggregateFunction = null, ?bool $sortColumns = null, ?string $separator = null, ?bool $maintainOrder = null): Phpolars\DataFrame
product(): Phpolars\DataFrame
quantile(float $quantile, ?string $interpolation = null): Phpolars\DataFrame
rechunk(): Phpolars\DataFrame
remove(mixed ...$predicates): Phpolars\DataFrame
rename(array $mapping, ?bool $strict = null): Phpolars\DataFrame
replaceColumn(int $index, Phpolars\Series $column): Phpolars\DataFrame
reverse(): Phpolars\DataFrame
rolling(mixed $indexColumn, string $period, ?string $offset = null, ?string $closed = null, mixed $groupBy = null): Phpolars\GroupBy
row(int $index, ?bool $named = null): mixed
rows(?bool $named = null): array
rowsByKey(string $key, ?bool $named = null, ?bool $includeKey = null, ?bool $unique = null): array
sample(?int $n = null, ?float $fraction = null, ?bool $withReplacement = null, ?bool $shuffle = null, ?int $seed = null): Phpolars\DataFrame
schema(): array
select(mixed ...$exprs): Phpolars\DataFrame
selectSeq(mixed ...$exprs): Phpolars\DataFrame
shape(): array
shift(?int $n = null, mixed $fillValue = null): Phpolars\DataFrame
shrinkToFit(): Phpolars\DataFrame
slice(int $offset, ?int $length = null): Phpolars\DataFrame
sort(mixed $by, mixed $descending = null, mixed $nullsLast = null, ?bool $maintainOrder = null): Phpolars\DataFrame
sql(string $query, ?string $tableName = null): Phpolars\DataFrame
static fromDict(array $data, ?array $schema = null): Phpolars\DataFrame
static fromDicts(array $data, ?array $schema = null): Phpolars\DataFrame
static fromPhibble(mixed $table): Phpolars\DataFrame
static fromRows(array $data, ?array $schema = null): Phpolars\DataFrame
std(?int $ddof = null): Phpolars\DataFrame
sum(): Phpolars\DataFrame
sumHorizontal(?bool $ignoreNulls = null): Phpolars\Series
tail(?int $n = null): Phpolars\DataFrame
toColumns(): array
toDict(?bool $asSeries = null): array
toDicts(): array
toDummies(mixed $columns = null, ?string $separator = null, ?bool $dropFirst = null, ?bool $dropNulls = null): Phpolars\DataFrame
toPhibble(): mixed
toSeries(?int $index = null): Phpolars\Series
toStruct(?string $name = null): Phpolars\Series
topK(int $k, mixed $by, mixed $reverse = null): Phpolars\DataFrame
transpose(?bool $includeHeader = null, ?string $headerName = null, mixed $columnNames = null): Phpolars\DataFrame
unique(mixed $subset = null, ?string $keep = null, ?bool $maintainOrder = null): Phpolars\DataFrame
unnest(mixed ...$columns): Phpolars\DataFrame
unpivot(mixed $on = null, mixed $index = null, ?string $variableName = null, ?string $valueName = null): Phpolars\DataFrame
update(mixed $other, mixed $on = null, ?string $how = null, mixed $leftOn = null, mixed $rightOn = null, ?bool $includeNulls = null, ?string $maintainOrder = null): Phpolars\DataFrame
upsample(string $timeColumn, string $every, mixed $groupBy = null, ?bool $maintainOrder = null): Phpolars\DataFrame
var(?int $ddof = null): Phpolars\DataFrame
vstack(Phpolars\DataFrame $other): Phpolars\DataFrame
width(): int
withColumns(mixed ...$exprs): Phpolars\DataFrame
withColumnsSeq(mixed ...$exprs): Phpolars\DataFrame
withRowIndex(?string $name = null, ?int $offset = null): Phpolars\DataFrame
writeCsv(string $file, ?bool $includeHeader = null, ?string $separator = null, ?string $nullValue = null, ?string $quoteChar = null, ?string $dateFormat = null, ?string $datetimeFormat = null, ?string $timeFormat = null, ?bool $floatScientific = null, ?int $floatPrecision = null, ?bool $decimalComma = null, ?string $lineTerminator = null, ?string $quoteStyle = null, ?bool $includeBom = null): void
writeDatabase(string $tableName, mixed $connection, ?string $ifTableExists = null): int
writeIpc(string $file): void
writeJson(string $file): void
writeNdjson(string $file): void
writeParquet(string $file, ?string $compression = null, ?int $compressionLevel = null, ?bool $statistics = null, ?int $rowGroupSize = null): void
```

### `Phpolars\LazyFrame`

```php
__toString(): string
approxNUnique(): Phpolars\LazyFrame
bottomK(int $k, mixed $by, mixed $reverse = null): Phpolars\LazyFrame
cache(): Phpolars\LazyFrame
cast(mixed $dtypes, ?bool $strict = null): Phpolars\LazyFrame
clear(?int $n = null): Phpolars\LazyFrame
clone(): Phpolars\LazyFrame
collect(): Phpolars\DataFrame
collectSchema(): array
columns(): array
count(): Phpolars\LazyFrame
describe(?array $percentiles = null): Phpolars\DataFrame
drop(mixed ...$columns): Phpolars\LazyFrame
dropNans(mixed $subset = null): Phpolars\LazyFrame
dropNulls(mixed $subset = null): Phpolars\LazyFrame
dtypes(): array
explain(?bool $optimized = null): string
explode(mixed ...$columns): Phpolars\LazyFrame
fetch(?int $n = null): Phpolars\DataFrame
fillNan(mixed $value): Phpolars\LazyFrame
fillNull(mixed $value = null, ?string $strategy = null, ?int $limit = null): Phpolars\LazyFrame
filter(mixed ...$predicates): Phpolars\LazyFrame
first(): Phpolars\LazyFrame
gather(mixed $indices): Phpolars\LazyFrame
gatherEvery(int $n, ?int $offset = null): Phpolars\LazyFrame
groupBy(mixed $by, ?bool $maintainOrder = null): Phpolars\LazyGroupBy
groupByDynamic(mixed $indexColumn, string $every, ?string $period = null, ?string $offset = null, ?bool $includeBoundaries = null, ?string $closed = null, ?string $label = null, mixed $groupBy = null, ?string $startBy = null): Phpolars\LazyGroupBy
head(?int $n = null): Phpolars\LazyFrame
interpolate(): Phpolars\LazyFrame
join(mixed $other, mixed $on = null, ?string $how = null, mixed $leftOn = null, mixed $rightOn = null, ?string $suffix = null, ?bool $coalesce = null, ?string $maintainOrder = null, ?bool $nullsEqual = null): Phpolars\LazyFrame
joinAsof(mixed $other, mixed $on = null, mixed $leftOn = null, mixed $rightOn = null, mixed $by = null, mixed $byLeft = null, mixed $byRight = null, ?string $strategy = null, ?string $suffix = null, mixed $tolerance = null, ?bool $allowExactMatches = null, ?bool $checkSortedness = null, ?bool $coalesce = null): Phpolars\LazyFrame
joinWhere(mixed $other, mixed ...$predicates): Phpolars\LazyFrame
last(): Phpolars\LazyFrame
lazy(): Phpolars\LazyFrame
limit(?int $n = null): Phpolars\LazyFrame
max(): Phpolars\LazyFrame
mean(): Phpolars\LazyFrame
median(): Phpolars\LazyFrame
mergeSorted(mixed $other, mixed $key, ?bool $maintainOrder = null): Phpolars\LazyFrame
min(): Phpolars\LazyFrame
nUnique(): Phpolars\LazyFrame
nullCount(): Phpolars\LazyFrame
product(): Phpolars\LazyFrame
profile(): array
quantile(float $quantile, ?string $interpolation = null): Phpolars\LazyFrame
remove(mixed ...$predicates): Phpolars\LazyFrame
rename(array $mapping, ?bool $strict = null): Phpolars\LazyFrame
reverse(): Phpolars\LazyFrame
rolling(mixed $indexColumn, string $period, ?string $offset = null, ?string $closed = null, mixed $groupBy = null): Phpolars\LazyGroupBy
schema(): array
select(mixed ...$exprs): Phpolars\LazyFrame
selectSeq(mixed ...$exprs): Phpolars\LazyFrame
shift(?int $n = null, mixed $fillValue = null): Phpolars\LazyFrame
sinkCsv(string $file, ?bool $includeHeader = null, ?string $separator = null, ?string $nullValue = null, ?string $quoteChar = null, ?string $dateFormat = null, ?string $datetimeFormat = null, ?string $timeFormat = null, ?bool $floatScientific = null, ?int $floatPrecision = null, ?bool $decimalComma = null, ?string $lineTerminator = null, ?string $quoteStyle = null, ?bool $includeBom = null): void
sinkIpc(string $file): void
sinkNdjson(string $file): void
sinkParquet(string $file, ?string $compression = null, ?int $compressionLevel = null, ?bool $statistics = null, ?int $rowGroupSize = null): void
slice(int $offset, ?int $length = null): Phpolars\LazyFrame
sort(mixed $by, mixed $descending = null, mixed $nullsLast = null, ?bool $maintainOrder = null): Phpolars\LazyFrame
sql(string $query, ?string $tableName = null): Phpolars\LazyFrame
std(?int $ddof = null): Phpolars\LazyFrame
sum(): Phpolars\LazyFrame
tail(?int $n = null): Phpolars\LazyFrame
topK(int $k, mixed $by, mixed $reverse = null): Phpolars\LazyFrame
unique(mixed $subset = null, ?string $keep = null, ?bool $maintainOrder = null): Phpolars\LazyFrame
unnest(mixed ...$columns): Phpolars\LazyFrame
unpivot(mixed $on = null, mixed $index = null, ?string $variableName = null, ?string $valueName = null): Phpolars\LazyFrame
update(mixed $other, mixed $on = null, ?string $how = null, mixed $leftOn = null, mixed $rightOn = null, ?bool $includeNulls = null, ?string $maintainOrder = null): Phpolars\LazyFrame
var(?int $ddof = null): Phpolars\LazyFrame
width(): int
withColumns(mixed ...$exprs): Phpolars\LazyFrame
withColumnsSeq(mixed ...$exprs): Phpolars\LazyFrame
withRowIndex(?string $name = null, ?int $offset = null): Phpolars\LazyFrame
```

### `Phpolars\GroupBy`

```php
agg(mixed ...$aggs): Phpolars\DataFrame
all(): Phpolars\DataFrame
count(): Phpolars\DataFrame
first(): Phpolars\DataFrame
getIterator(): Phpolars\GroupIterator
having(mixed ...$predicates): Phpolars\GroupBy
head(?int $n = null): Phpolars\DataFrame
last(): Phpolars\DataFrame
len(?string $name = null): Phpolars\DataFrame
max(): Phpolars\DataFrame
mean(): Phpolars\DataFrame
median(): Phpolars\DataFrame
min(): Phpolars\DataFrame
nUnique(): Phpolars\DataFrame
quantile(float $quantile, ?string $interpolation = null): Phpolars\DataFrame
sum(): Phpolars\DataFrame
tail(?int $n = null): Phpolars\DataFrame
```

### `Phpolars\LazyGroupBy`

```php
agg(mixed ...$aggs): Phpolars\LazyFrame
all(): Phpolars\LazyFrame
count(): Phpolars\LazyFrame
first(): Phpolars\LazyFrame
getIterator(): Phpolars\GroupIterator
having(mixed ...$predicates): Phpolars\LazyGroupBy
head(?int $n = null): Phpolars\LazyFrame
last(): Phpolars\LazyFrame
len(?string $name = null): Phpolars\LazyFrame
max(): Phpolars\LazyFrame
mean(): Phpolars\LazyFrame
median(): Phpolars\LazyFrame
min(): Phpolars\LazyFrame
nUnique(): Phpolars\LazyFrame
quantile(float $quantile, ?string $interpolation = null): Phpolars\LazyFrame
sum(): Phpolars\LazyFrame
tail(?int $n = null): Phpolars\LazyFrame
```

