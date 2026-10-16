### `Phpolars\Series`

```php
__call(string $name, array $arguments): mixed
__toString(): string
abs(): Phpolars\Series
add(mixed $other): Phpolars\Series
alias(string $name): Phpolars\Series
all(?bool $ignoreNulls = null): mixed
and(mixed $other): Phpolars\Series
any(?bool $ignoreNulls = null): mixed
append(Phpolars\Series $other): Phpolars\Series
argMax(): mixed
argMin(): mixed
argSort(?bool $descending = null, ?bool $nullsLast = null): Phpolars\Series
argTrue(): Phpolars\Series
arr(): Phpolars\ArrayNameSpace
bin(): Phpolars\BinaryNameSpace
cast(string $dtype, ?bool $strict = null): Phpolars\Series
cat(): Phpolars\CatNameSpace
chunkLengths(): array
clear(?int $n = null): Phpolars\Series
clip(mixed $lowerBound = null, mixed $upperBound = null): Phpolars\Series
count(): int
cumMax(?bool $reverse = null): Phpolars\Series
cumMin(?bool $reverse = null): Phpolars\Series
cumProd(?bool $reverse = null): Phpolars\Series
cumSum(?bool $reverse = null): Phpolars\Series
describe(?array $percentiles = null): Phpolars\DataFrame
diff(?int $n = null): Phpolars\Series
div(mixed $other): Phpolars\Series
dropNulls(): Phpolars\Series
dt(): Phpolars\DtNameSpace
dtype(): string
eq(mixed $other): Phpolars\Series
equals(Phpolars\Series $other): bool
estimatedSize(): int
exp(): Phpolars\Series
extend(Phpolars\Series $other): Phpolars\Series
fillNan(mixed $value): Phpolars\Series
fillNull(mixed $value = null, ?string $strategy = null, ?int $limit = null): Phpolars\Series
filter(mixed $predicate): Phpolars\Series
first(): mixed
floordiv(mixed $other): Phpolars\Series
gather(mixed $indices): Phpolars\Series
ge(mixed $other): Phpolars\Series
get(int $index): mixed
gt(mixed $other): Phpolars\Series
hasNulls(): bool
head(?int $n = null): Phpolars\Series
isBetween(mixed $lower, mixed $upper, ?string $closed = null): Phpolars\Series
isDuplicated(): Phpolars\Series
isEmpty(): bool
isIn(mixed $other, ?bool $nullsEqual = null): Phpolars\Series
isNan(): Phpolars\Series
isNotNull(): Phpolars\Series
isNull(): Phpolars\Series
isSorted(?bool $descending = null, ?bool $nullsLast = null): bool
isUnique(): Phpolars\Series
item(): mixed
last(): mixed
le(mixed $other): Phpolars\Series
len(): int
list(): Phpolars\ListNameSpace
log(?float $base = null): Phpolars\Series
lt(mixed $other): Phpolars\Series
max(): mixed
mean(): mixed
median(): mixed
min(): mixed
mod(mixed $other): Phpolars\Series
mul(mixed $other): Phpolars\Series
nChunks(): int
nUnique(): mixed
name(): string
ne(mixed $other): Phpolars\Series
newFromIndex(int $index, int $length): Phpolars\Series
not(): Phpolars\Series
nullCount(): int
or(mixed $other): Phpolars\Series
pctChange(?int $n = null): Phpolars\Series
pow(mixed $exponent): Phpolars\Series
product(): mixed
quantile(float $quantile, ?string $interpolation = null): mixed
rank(?string $method = null, ?bool $descending = null, ?int $seed = null): Phpolars\Series
rechunk(): Phpolars\Series
rename(string $name): Phpolars\Series
reverse(): Phpolars\Series
round(?int $decimals = null): Phpolars\Series
sample(?int $n = null, ?float $fraction = null, ?bool $withReplacement = null, ?bool $shuffle = null, ?int $seed = null): Phpolars\Series
scatter(mixed $indices, mixed $values): Phpolars\Series
set(Phpolars\Series $filter, mixed $value): Phpolars\Series
shape(): array
shift(?int $n = null): Phpolars\Series
shrinkToFit(): Phpolars\Series
slice(int $offset, ?int $length = null): Phpolars\Series
sort(?bool $descending = null, ?bool $nullsLast = null): Phpolars\Series
sqrt(): Phpolars\Series
std(?int $ddof = null): mixed
str(): Phpolars\StrNameSpace
struct(): Phpolars\StructNameSpace
sub(mixed $other): Phpolars\Series
sum(): mixed
tail(?int $n = null): Phpolars\Series
toDummies(?string $separator = null, ?bool $dropFirst = null, ?bool $dropNulls = null): Phpolars\DataFrame
toFrame(?string $name = null): Phpolars\DataFrame
toList(): array
truediv(mixed $other): Phpolars\Series
unique(?bool $maintainOrder = null): Phpolars\Series
valueCounts(?bool $sort = null, ?string $name = null, ?bool $normalize = null): Phpolars\DataFrame
var(?int $ddof = null): mixed
zipWith(Phpolars\Series $mask, Phpolars\Series $other): Phpolars\Series
```

