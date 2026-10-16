```php
all(): Phpolars\Expr
all_horizontal(mixed ...$exprs): Phpolars\Expr
any(mixed $names, ?bool $ignoreNulls = null): Phpolars\Expr
any_horizontal(mixed ...$exprs): Phpolars\Expr
approx_n_unique(mixed ...$names): Phpolars\Expr
arctan2(mixed $y, mixed $x): Phpolars\Expr
arg_sort_by(mixed $exprs, mixed $descending = null, mixed $nullsLast = null, ?bool $maintainOrder = null): Phpolars\Expr
arg_where(mixed $condition): Phpolars\Expr
business_day_count(mixed $start, mixed $end, ?array $weekMask = null, mixed $holidays = null): Phpolars\Expr
coalesce(mixed ...$exprs): Phpolars\Expr
col(mixed ...$names): Phpolars\Expr
collect_all(array $lazyFrames): array
concat(array $items, ?string $how = null): mixed
concat_arr(mixed ...$exprs): Phpolars\Expr
concat_list(mixed ...$exprs): Phpolars\Expr
concat_str(mixed $exprs, ?string $separator = null, ?bool $ignoreNulls = null): Phpolars\Expr
corr(mixed $a, mixed $b, ?string $method = null, ?bool $propagateNans = null): Phpolars\Expr
count(mixed ...$names): Phpolars\Expr
cov(mixed $a, mixed $b, ?int $ddof = null): Phpolars\Expr
cum_count(mixed $names, ?bool $reverse = null): Phpolars\Expr
cum_sum(mixed $names, ?bool $reverse = null): Phpolars\Expr
cum_sum_horizontal(mixed ...$exprs): Phpolars\Expr
date(mixed $year, mixed $month, mixed $day): Phpolars\Expr
date_range(mixed $start = null, mixed $end = null, ?string $interval = null, ?int $numSamples = null, ?string $closed = null): Phpolars\Expr
date_ranges(mixed $start = null, mixed $end = null, ?string $interval = null, ?string $closed = null): Phpolars\Expr
datetime(mixed $year, mixed $month, mixed $day, mixed $hour = null, mixed $minute = null, mixed $second = null, mixed $microsecond = null, ?string $timeUnit = null, ?string $timeZone = null): Phpolars\Expr
datetime_range(mixed $start = null, mixed $end = null, ?string $interval = null, ?int $numSamples = null, ?string $closed = null, ?string $timeUnit = null, ?string $timeZone = null): Phpolars\Expr
datetime_ranges(mixed $start = null, mixed $end = null, ?string $interval = null, ?string $closed = null, ?string $timeUnit = null, ?string $timeZone = null): Phpolars\Expr
duration(mixed $weeks = null, mixed $days = null, mixed $hours = null, mixed $minutes = null, mixed $seconds = null, mixed $milliseconds = null, mixed $microseconds = null, mixed $nanoseconds = null, ?string $timeUnit = null): Phpolars\Expr
element(): Phpolars\Expr
escape_regex(string $s): string
exclude(mixed ...$names): Phpolars\Expr
field(mixed $name): Phpolars\Expr
first(mixed ...$names): Phpolars\Expr
format(string $fString, mixed ...$args): Phpolars\Expr
from_dict(array $data, ?array $schema = null): Phpolars\DataFrame
from_dicts(array $data, ?array $schema = null): Phpolars\DataFrame
from_epoch(mixed $column, ?string $timeUnit = null): Phpolars\Expr
from_records(array $data, ?array $schema = null): Phpolars\DataFrame
head(mixed $names, ?int $n = null): Phpolars\Expr
implode(mixed ...$names): Phpolars\Expr
int_range(mixed $start, mixed $end = null, ?int $step = null): Phpolars\Expr
int_ranges(mixed $start, mixed $end = null, mixed $step = null): Phpolars\Expr
last(mixed ...$names): Phpolars\Expr
len(): Phpolars\Expr
linear_space(mixed $start, mixed $end, mixed $numSamples, ?string $closed = null): Phpolars\Expr
linear_spaces(mixed $start, mixed $end, mixed $numSamples, ?string $closed = null, ?bool $asArray = null): Phpolars\Expr
lit(mixed $value, ?string $dtype = null): Phpolars\Expr
max(mixed ...$names): Phpolars\Expr
max_horizontal(mixed ...$exprs): Phpolars\Expr
mean(mixed ...$names): Phpolars\Expr
mean_horizontal(mixed ...$exprs): Phpolars\Expr
median(mixed ...$names): Phpolars\Expr
min(mixed ...$names): Phpolars\Expr
min_horizontal(mixed ...$exprs): Phpolars\Expr
n_unique(mixed ...$names): Phpolars\Expr
nth(mixed ...$indices): Phpolars\Expr
ones(mixed $n, ?string $dtype = null): Phpolars\Expr
polars_version(): string
quantile(mixed $column, float $quantile, ?string $interpolation = null): Phpolars\Expr
repeat(mixed $value, mixed $n, ?string $dtype = null): Phpolars\Expr
rolling_corr(mixed $a, mixed $b, int $windowSize, ?int $minPeriods = null, ?int $ddof = null): Phpolars\Expr
rolling_cov(mixed $a, mixed $b, int $windowSize, ?int $minPeriods = null, ?int $ddof = null): Phpolars\Expr
row_index(?string $name = null): Phpolars\Expr
select(mixed ...$exprs): Phpolars\DataFrame
set_random_seed(int $seed): void
sql(string $query, ?array $frames = null, ?bool $eager = null): mixed
std(mixed $column, ?int $ddof = null): Phpolars\Expr
struct_(mixed ...$exprs): Phpolars\Expr
sum(mixed ...$names): Phpolars\Expr
sum_horizontal(mixed ...$exprs): Phpolars\Expr
tail(mixed $names, ?int $n = null): Phpolars\Expr
thread_pool_size(): int
threadpool_size(): int
time(mixed $hour = null, mixed $minute = null, mixed $second = null, mixed $microsecond = null): Phpolars\Expr
time_range(mixed $start = null, mixed $end = null, ?string $interval = null, ?string $closed = null): Phpolars\Expr
time_ranges(mixed $start = null, mixed $end = null, ?string $interval = null, ?string $closed = null): Phpolars\Expr
union(array $items, ?string $how = null): mixed
var_(mixed $column, ?int $ddof = null): Phpolars\Expr
version(): string
when(mixed ...$conditions): Phpolars\When
zeros(mixed $n, ?string $dtype = null): Phpolars\Expr
```
