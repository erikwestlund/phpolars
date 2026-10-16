### `Phpolars\Expr`

```php
__toString(): string
abs(): Phpolars\Expr
add(mixed $other): Phpolars\Expr
alias(string $name): Phpolars\Expr
all(?bool $ignoreNulls = null): Phpolars\Expr
and(mixed $other): Phpolars\Expr
any(?bool $ignoreNulls = null): Phpolars\Expr
append(mixed $other, ?bool $upcast = null): Phpolars\Expr
approxNUnique(): Phpolars\Expr
arccos(): Phpolars\Expr
arccosh(): Phpolars\Expr
arcsin(): Phpolars\Expr
arcsinh(): Phpolars\Expr
arctan(): Phpolars\Expr
arctanh(): Phpolars\Expr
argMax(): Phpolars\Expr
argMin(): Phpolars\Expr
argSort(?bool $descending = null, ?bool $nullsLast = null): Phpolars\Expr
argUnique(): Phpolars\Expr
arr(): Phpolars\ArrayNameSpace
backwardFill(?int $limit = null): Phpolars\Expr
bin(): Phpolars\BinaryNameSpace
bitwiseAnd(): Phpolars\Expr
bitwiseCountOnes(): Phpolars\Expr
bitwiseCountZeros(): Phpolars\Expr
bitwiseLeadingOnes(): Phpolars\Expr
bitwiseLeadingZeros(): Phpolars\Expr
bitwiseOr(): Phpolars\Expr
bitwiseTrailingOnes(): Phpolars\Expr
bitwiseTrailingZeros(): Phpolars\Expr
bitwiseXor(): Phpolars\Expr
bottomK(?int $k = null): Phpolars\Expr
cast(string $dtype, ?bool $strict = null): Phpolars\Expr
cat(): Phpolars\CatNameSpace
cbrt(): Phpolars\Expr
ceil(): Phpolars\Expr
clip(mixed $lowerBound = null, mixed $upperBound = null): Phpolars\Expr
cos(): Phpolars\Expr
cosh(): Phpolars\Expr
cot(): Phpolars\Expr
count(): Phpolars\Expr
cumCount(?bool $reverse = null): Phpolars\Expr
cumMax(?bool $reverse = null): Phpolars\Expr
cumMin(?bool $reverse = null): Phpolars\Expr
cumProd(?bool $reverse = null): Phpolars\Expr
cumSum(?bool $reverse = null): Phpolars\Expr
cumulativeEval(mixed $expr, ?int $minPeriods = null): Phpolars\Expr
cut(mixed $breaks, ?array $labels = null, ?bool $leftClosed = null, ?bool $includeBreaks = null): Phpolars\Expr
degrees(): Phpolars\Expr
diff(?int $n = null, ?string $nullBehavior = null): Phpolars\Expr
div(mixed $other): Phpolars\Expr
dot(mixed $other): Phpolars\Expr
dropNans(): Phpolars\Expr
dropNulls(): Phpolars\Expr
dt(): Phpolars\DtNameSpace
entropy(?float $base = null, ?bool $normalize = null): Phpolars\Expr
eq(mixed $other): Phpolars\Expr
eqMissing(mixed $other): Phpolars\Expr
ewmMean(?float $com = null, ?float $span = null, ?float $halfLife = null, ?float $alpha = null, ?bool $adjust = null, ?int $minPeriods = null, ?bool $ignoreNulls = null): Phpolars\Expr
ewmMeanBy(mixed $by, string $halfLife): Phpolars\Expr
ewmStd(?float $com = null, ?float $span = null, ?float $halfLife = null, ?float $alpha = null, ?bool $adjust = null, ?bool $bias = null, ?int $minPeriods = null, ?bool $ignoreNulls = null): Phpolars\Expr
ewmVar(?float $com = null, ?float $span = null, ?float $halfLife = null, ?float $alpha = null, ?bool $adjust = null, ?bool $bias = null, ?int $minPeriods = null, ?bool $ignoreNulls = null): Phpolars\Expr
exclude(mixed ...$columns): Phpolars\Expr
exp(): Phpolars\Expr
explode(): Phpolars\Expr
fillNan(mixed $value): Phpolars\Expr
fillNull(mixed $value = null, ?string $strategy = null, ?int $limit = null): Phpolars\Expr
filter(mixed $predicate): Phpolars\Expr
first(?bool $ignoreNulls = null): Phpolars\Expr
floor(): Phpolars\Expr
floordiv(mixed $other): Phpolars\Expr
forwardFill(?int $limit = null): Phpolars\Expr
gather(mixed $indices): Phpolars\Expr
gatherEvery(int $n, ?int $offset = null): Phpolars\Expr
ge(mixed $other): Phpolars\Expr
get(mixed $index, ?bool $nullOnOob = null): Phpolars\Expr
gt(mixed $other): Phpolars\Expr
hasNulls(): Phpolars\Expr
head(?int $n = null): Phpolars\Expr
hist(mixed $bins = null, ?int $binCount = null, ?bool $includeCategory = null, ?bool $includeBreakpoint = null): Phpolars\Expr
implode(): Phpolars\Expr
indexOf(mixed $element): Phpolars\Expr
interpolate(?string $method = null): Phpolars\Expr
interpolateBy(mixed $by): Phpolars\Expr
isBetween(mixed $lower, mixed $upper, ?string $closed = null): Phpolars\Expr
isClose(mixed $other, ?float $absTol = null, ?float $relTol = null, ?bool $nansEqual = null): Phpolars\Expr
isDuplicated(): Phpolars\Expr
isFinite(): Phpolars\Expr
isFirstDistinct(): Phpolars\Expr
isIn(mixed $other, ?bool $nullsEqual = null): Phpolars\Expr
isInfinite(): Phpolars\Expr
isLastDistinct(): Phpolars\Expr
isNan(): Phpolars\Expr
isNotNan(): Phpolars\Expr
isNotNull(): Phpolars\Expr
isNull(): Phpolars\Expr
isUnique(): Phpolars\Expr
item(?bool $allowEmpty = null): Phpolars\Expr
kurtosis(?bool $fisher = null, ?bool $bias = null): Phpolars\Expr
last(?bool $ignoreNulls = null): Phpolars\Expr
le(mixed $other): Phpolars\Expr
len(): Phpolars\Expr
limit(?int $n = null): Phpolars\Expr
list(): Phpolars\ListNameSpace
log(?float $base = null): Phpolars\Expr
log10(): Phpolars\Expr
log1P(): Phpolars\Expr
lowerBound(): Phpolars\Expr
lt(mixed $other): Phpolars\Expr
max(): Phpolars\Expr
maxBy(mixed $by): Phpolars\Expr
mean(): Phpolars\Expr
median(): Phpolars\Expr
min(): Phpolars\Expr
minBy(mixed $by): Phpolars\Expr
mod(mixed $other): Phpolars\Expr
mode(): Phpolars\Expr
mul(mixed $other): Phpolars\Expr
nUnique(): Phpolars\Expr
name(): Phpolars\NameNameSpace
nanMax(): Phpolars\Expr
nanMin(): Phpolars\Expr
ne(mixed $other): Phpolars\Expr
neMissing(mixed $other): Phpolars\Expr
neg(): Phpolars\Expr
not(): Phpolars\Expr
nullCount(): Phpolars\Expr
or(mixed $other): Phpolars\Expr
over(mixed $partitionBy, mixed $orderBy = null, ?bool $descending = null, ?bool $nullsLast = null, ?string $mappingStrategy = null): Phpolars\Expr
pctChange(?int $n = null): Phpolars\Expr
peakMax(): Phpolars\Expr
peakMin(): Phpolars\Expr
pow(mixed $exponent): Phpolars\Expr
product(): Phpolars\Expr
qcut(mixed $quantiles, ?array $labels = null, ?bool $leftClosed = null, ?bool $allowDuplicates = null, ?bool $includeBreaks = null): Phpolars\Expr
quantile(mixed $quantile, ?string $interpolation = null): Phpolars\Expr
radians(): Phpolars\Expr
rank(?string $method = null, ?bool $descending = null, ?int $seed = null): Phpolars\Expr
repeatBy(mixed $by): Phpolars\Expr
replace(mixed $old, mixed $new = null): Phpolars\Expr
replaceStrict(mixed $old, mixed $new = null, mixed $default = null, ?string $returnDtype = null): Phpolars\Expr
reverse(): Phpolars\Expr
rle(): Phpolars\Expr
rleId(): Phpolars\Expr
rollingKurtosis(int $windowSize, ?bool $fisher = null, ?bool $bias = null, ?int $minPeriods = null, ?bool $center = null): Phpolars\Expr
rollingMax(int $windowSize, ?int $minPeriods = null, ?bool $center = null): Phpolars\Expr
rollingMaxBy(mixed $by, string $windowSize, ?int $minPeriods = null, ?string $closed = null): Phpolars\Expr
rollingMean(int $windowSize, ?int $minPeriods = null, ?bool $center = null): Phpolars\Expr
rollingMeanBy(mixed $by, string $windowSize, ?int $minPeriods = null, ?string $closed = null): Phpolars\Expr
rollingMedian(int $windowSize, ?int $minPeriods = null, ?bool $center = null): Phpolars\Expr
rollingMedianBy(mixed $by, string $windowSize, ?int $minPeriods = null, ?string $closed = null): Phpolars\Expr
rollingMin(int $windowSize, ?int $minPeriods = null, ?bool $center = null): Phpolars\Expr
rollingMinBy(mixed $by, string $windowSize, ?int $minPeriods = null, ?string $closed = null): Phpolars\Expr
rollingQuantile(float $quantile, int $windowSize, ?string $interpolation = null, ?int $minPeriods = null, ?bool $center = null): Phpolars\Expr
rollingQuantileBy(mixed $by, string $windowSize, float $quantile, ?string $interpolation = null, ?int $minPeriods = null, ?string $closed = null): Phpolars\Expr
rollingSkew(int $windowSize, ?bool $bias = null, ?int $minPeriods = null, ?bool $center = null): Phpolars\Expr
rollingStd(int $windowSize, ?int $minPeriods = null, ?bool $center = null): Phpolars\Expr
rollingStdBy(mixed $by, string $windowSize, ?int $minPeriods = null, ?string $closed = null, ?int $ddof = null): Phpolars\Expr
rollingSum(int $windowSize, ?int $minPeriods = null, ?bool $center = null): Phpolars\Expr
rollingSumBy(mixed $by, string $windowSize, ?int $minPeriods = null, ?string $closed = null): Phpolars\Expr
rollingVar(int $windowSize, ?int $minPeriods = null, ?bool $center = null): Phpolars\Expr
rollingVarBy(mixed $by, string $windowSize, ?int $minPeriods = null, ?string $closed = null, ?int $ddof = null): Phpolars\Expr
round(?int $decimals = null, ?string $mode = null): Phpolars\Expr
roundSigFigs(int $digits): Phpolars\Expr
sample(?int $n = null, ?float $fraction = null, ?bool $withReplacement = null, ?bool $shuffle = null, ?int $seed = null): Phpolars\Expr
searchSorted(mixed $element, ?string $side = null, ?bool $descending = null): Phpolars\Expr
shift(?int $n = null, mixed $fillValue = null): Phpolars\Expr
shuffle(?int $seed = null): Phpolars\Expr
sign(): Phpolars\Expr
sin(): Phpolars\Expr
sinh(): Phpolars\Expr
skew(?bool $bias = null): Phpolars\Expr
slice(int $offset, ?int $length = null): Phpolars\Expr
sort(?bool $descending = null, ?bool $nullsLast = null): Phpolars\Expr
sortBy(mixed $by, mixed $descending = null, mixed $nullsLast = null): Phpolars\Expr
sqrt(): Phpolars\Expr
std(?int $ddof = null): Phpolars\Expr
str(): Phpolars\StrNameSpace
struct(): Phpolars\StructNameSpace
sub(mixed $other): Phpolars\Expr
sum(): Phpolars\Expr
tail(?int $n = null): Phpolars\Expr
tan(): Phpolars\Expr
tanh(): Phpolars\Expr
toPhysical(): Phpolars\Expr
topK(?int $k = null): Phpolars\Expr
truediv(mixed $other): Phpolars\Expr
unique(?bool $maintainOrder = null): Phpolars\Expr
uniqueCounts(): Phpolars\Expr
upperBound(): Phpolars\Expr
valueCounts(?bool $sort = null, ?string $name = null, ?bool $normalize = null): Phpolars\Expr
var(?int $ddof = null): Phpolars\Expr
xor(mixed $other): Phpolars\Expr
```

### `Phpolars\When`

```php
then(mixed $value): Phpolars\Then
```

### `Phpolars\Then`

```php
alias(string $name): Phpolars\Expr
otherwise(mixed $value): Phpolars\Expr
when(mixed ...$condition): Phpolars\When
```

### `Phpolars\StrNameSpace`

```php
contains(mixed $pattern, ?bool $literal = null, ?bool $strict = null): mixed
containsAny(mixed $patterns, ?bool $asciiCaseInsensitive = null): mixed
countMatches(mixed $pattern, ?bool $literal = null): mixed
decode(string $encoding, ?bool $strict = null): mixed
encode(string $encoding): mixed
endsWith(mixed $suffix): mixed
escapeRegex(): mixed
extract(mixed $pattern, ?int $groupIndex = null): mixed
extractAll(mixed $pattern): mixed
extractGroups(string $pattern): mixed
extractMany(mixed $patterns, ?bool $asciiCaseInsensitive = null, ?bool $overlapping = null, ?bool $leftmost = null): mixed
find(mixed $pattern, ?bool $literal = null, ?bool $strict = null): mixed
findMany(mixed $patterns, ?bool $asciiCaseInsensitive = null, ?bool $overlapping = null, ?bool $leftmost = null): mixed
head(int $n): mixed
join(?string $delimiter = null, ?bool $ignoreNulls = null): mixed
jsonDecode(string $dtype): mixed
jsonPathMatch(mixed $jsonPath): mixed
lenBytes(): mixed
lenChars(): mixed
normalize(?string $form = null): mixed
padEnd(mixed $length, ?string $fillChar = null): mixed
padStart(mixed $length, ?string $fillChar = null): mixed
replace(mixed $pattern, mixed $value, ?bool $literal = null, ?int $n = null): mixed
replaceAll(mixed $pattern, mixed $value, ?bool $literal = null): mixed
replaceMany(mixed $patterns, mixed $replaceWith = null, ?bool $asciiCaseInsensitive = null, ?bool $leftmost = null): mixed
reverse(): mixed
slice(mixed $offset, mixed $length = null): mixed
split(mixed $by, ?bool $inclusive = null, ?bool $literal = null): mixed
splitExact(mixed $by, int $n, ?bool $inclusive = null): mixed
splitn(mixed $by, int $n): mixed
startsWith(mixed $prefix): mixed
stripChars(?string $characters = null): mixed
stripCharsEnd(?string $characters = null): mixed
stripCharsStart(?string $characters = null): mixed
stripPrefix(mixed $prefix): mixed
stripSuffix(mixed $suffix): mixed
strptime(string $dtype, ?string $format = null, ?bool $strict = null, ?bool $exact = null): mixed
tail(int $n): mixed
toDate(?string $format = null, ?bool $strict = null, ?bool $exact = null): mixed
toDatetime(?string $format = null, ?string $timeUnit = null, ?string $timeZone = null, ?bool $strict = null, ?bool $exact = null): mixed
toDecimal(int $scale): mixed
toInteger(?int $base = null, ?string $dtype = null, ?bool $strict = null): mixed
toLowercase(): mixed
toTime(?string $format = null, ?bool $strict = null): mixed
toUppercase(): mixed
zfill(int $length): mixed
```

### `Phpolars\DtNameSpace`

```php
addBusinessDays(mixed $n, ?array $weekMask = null, mixed $holidays = null, ?string $roll = null): mixed
baseUtcOffset(): mixed
castTimeUnit(string $timeUnit): mixed
century(): mixed
combine(mixed $time, ?string $timeUnit = null): mixed
convertTimeZone(string $timeZone): mixed
date(): mixed
day(): mixed
daysInMonth(): mixed
dstOffset(): mixed
epoch(?string $timeUnit = null): mixed
hour(): mixed
isBusinessDay(?array $weekMask = null, mixed $holidays = null): mixed
isLeapYear(): mixed
isoYear(): mixed
microsecond(): mixed
millennium(): mixed
millisecond(): mixed
minute(): mixed
month(): mixed
monthEnd(): mixed
monthStart(): mixed
nanosecond(): mixed
offsetBy(string $by): mixed
ordinalDay(): mixed
quarter(): mixed
replace(mixed $year = null, mixed $month = null, mixed $day = null, mixed $hour = null, mixed $minute = null, mixed $second = null, mixed $microsecond = null, ?string $ambiguous = null): mixed
replaceTimeZone(?string $timeZone = null, ?string $ambiguous = null, ?string $nonExistent = null): mixed
round(string $every): mixed
second(): mixed
strftime(string $format): mixed
time(): mixed
timestamp(?string $timeUnit = null): mixed
toString(?string $format = null): mixed
totalDays(?bool $fractional = null): mixed
totalHours(?bool $fractional = null): mixed
totalMicroseconds(?bool $fractional = null): mixed
totalMilliseconds(?bool $fractional = null): mixed
totalMinutes(?bool $fractional = null): mixed
totalNanoseconds(?bool $fractional = null): mixed
totalSeconds(?bool $fractional = null): mixed
truncate(string $every): mixed
week(): mixed
weekday(): mixed
withTimeUnit(string $timeUnit): mixed
year(): mixed
```

### `Phpolars\ListNameSpace`

```php
agg(mixed $expr): mixed
all(?bool $ignoreNulls = null): mixed
any(?bool $ignoreNulls = null): mixed
argMax(): mixed
argMin(): mixed
concat(mixed $other): mixed
contains(mixed $item, ?bool $nullsEqual = null): mixed
countMatches(mixed $element): mixed
diff(?int $n = null): mixed
dropNulls(): mixed
eval(mixed $expr): mixed
explode(): mixed
filter(mixed $predicate): mixed
first(): mixed
gather(mixed $indices, ?bool $nullOnOob = null): mixed
gatherEvery(mixed $n, mixed $offset = null): mixed
get(mixed $index, ?bool $nullOnOob = null): mixed
head(?int $n = null): mixed
join(mixed $separator, ?bool $ignoreNulls = null): mixed
last(): mixed
len(): mixed
max(): mixed
mean(): mixed
median(): mixed
min(): mixed
nUnique(): mixed
reverse(): mixed
sample(mixed $n = null, mixed $fraction = null, ?bool $withReplacement = null, ?bool $shuffle = null, ?int $seed = null): mixed
setDifference(mixed $other): mixed
setIntersection(mixed $other): mixed
setSymmetricDifference(mixed $other): mixed
setUnion(mixed $other): mixed
shift(?int $n = null): mixed
slice(int $offset, ?int $length = null): mixed
sort(?bool $descending = null, ?bool $nullsLast = null): mixed
std(?int $ddof = null): mixed
sum(): mixed
tail(?int $n = null): mixed
toArray(int $width): mixed
toStruct(mixed $fields = null): mixed
unique(?bool $maintainOrder = null): mixed
var(?int $ddof = null): mixed
```

### `Phpolars\StructNameSpace`

```php
field(string $name): mixed
jsonEncode(): mixed
renameFields(mixed $names): mixed
withFields(mixed ...$exprs): mixed
```

### `Phpolars\NameNameSpace`

```php
keep(): Phpolars\Expr
prefix(string $prefix): Phpolars\Expr
replace(string $pattern, string $value, ?bool $literal = null): Phpolars\Expr
suffix(string $suffix): Phpolars\Expr
toLowercase(): Phpolars\Expr
toUppercase(): Phpolars\Expr
```

### `Phpolars\CatNameSpace`

```php
endsWith(string $suffix): mixed
getCategories(): mixed
lenBytes(): mixed
lenChars(): mixed
slice(int $offset, ?int $length = null): mixed
startsWith(string $prefix): mixed
```

### `Phpolars\ArrayNameSpace`

```php
agg(mixed $expr): mixed
all(?bool $ignoreNulls = null): mixed
any(?bool $ignoreNulls = null): mixed
argMax(): mixed
argMin(): mixed
contains(mixed $item, ?bool $nullsEqual = null): mixed
countMatches(mixed $element): mixed
eval(mixed $expr, ?bool $asList = null): mixed
explode(): mixed
first(): mixed
get(mixed $index, ?bool $nullOnOob = null): mixed
head(?int $n = null, ?bool $asArray = null): mixed
join(mixed $separator, ?bool $ignoreNulls = null): mixed
last(): mixed
len(): mixed
max(): mixed
mean(): mixed
median(): mixed
min(): mixed
nUnique(): mixed
reverse(): mixed
shift(mixed $n = null): mixed
slice(mixed $offset, mixed $length = null, ?bool $asArray = null): mixed
sort(?bool $descending = null, ?bool $nullsLast = null): mixed
std(?int $ddof = null): mixed
sum(): mixed
tail(?int $n = null, ?bool $asArray = null): mixed
toList(): mixed
toStruct(): mixed
unique(?bool $maintainOrder = null): mixed
var(?int $ddof = null): mixed
```

### `Phpolars\BinaryNameSpace`

```php
contains(mixed $literal): mixed
decode(string $encoding, ?bool $strict = null): mixed
encode(string $encoding): mixed
endsWith(mixed $suffix): mixed
get(mixed $index, ?bool $nullOnOob = null): mixed
head(?int $n = null): mixed
size(?string $unit = null): mixed
slice(mixed $offset, mixed $length = null): mixed
startsWith(mixed $prefix): mixed
tail(?int $n = null): mixed
```

