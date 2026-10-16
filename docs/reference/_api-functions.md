```php
all(): Phpolars\Expr
all_horizontal(mixed ...$exprs): Phpolars\Expr
any_horizontal(mixed ...$exprs): Phpolars\Expr
approx_n_unique(mixed ...$names): Phpolars\Expr
arg_sort_by(mixed $exprs, mixed $descending = null, mixed $nullsLast = null, ?bool $maintainOrder = null): Phpolars\Expr
arg_where(mixed $condition): Phpolars\Expr
business_day_count(mixed $start, mixed $end, ?array $weekMask = null, mixed $holidays = null): Phpolars\Expr
coalesce(mixed ...$exprs): Phpolars\Expr
col(mixed ...$names): Phpolars\Expr
concat(array $items, ?string $how = null): mixed
concat_arr(mixed ...$exprs): Phpolars\Expr
concat_list(mixed ...$exprs): Phpolars\Expr
concat_str(mixed $exprs, ?string $separator = null, ?bool $ignoreNulls = null): Phpolars\Expr
corr(mixed $a, mixed $b, ?string $method = null, ?bool $propagateNans = null): Phpolars\Expr
count(mixed ...$names): Phpolars\Expr
cov(mixed $a, mixed $b, ?int $ddof = null): Phpolars\Expr
date(mixed $year, mixed $month, mixed $day): Phpolars\Expr
date_range(mixed $start = null, mixed $end = null, ?string $interval = null, ?int $numSamples = null, ?string $closed = null): Phpolars\Expr
date_ranges(mixed $start = null, mixed $end = null, ?string $interval = null, ?string $closed = null): Phpolars\Expr
datetime(mixed $year, mixed $month, mixed $day, mixed $hour = null, mixed $minute = null, mixed $second = null, mixed $microsecond = null, ?string $timeUnit = null, ?string $timeZone = null): Phpolars\Expr
datetime_range(mixed $start = null, mixed $end = null, ?string $interval = null, ?int $numSamples = null, ?string $closed = null, ?string $timeUnit = null, ?string $timeZone = null): Phpolars\Expr
datetime_ranges(mixed $start = null, mixed $end = null, ?string $interval = null, ?string $closed = null, ?string $timeUnit = null, ?string $timeZone = null): Phpolars\Expr
duration(mixed $weeks = null, mixed $days = null, mixed $hours = null, mixed $minutes = null, mixed $seconds = null, mixed $milliseconds = null, mixed $microseconds = null, mixed $nanoseconds = null, ?string $timeUnit = null): Phpolars\Expr
element(): Phpolars\Expr
exclude(mixed ...$names): Phpolars\Expr
field(mixed $name): Phpolars\Expr
first(mixed ...$names): Phpolars\Expr
format(string $fString, mixed ...$args): Phpolars\Expr
from_epoch(mixed $column, ?string $timeUnit = null): Phpolars\Expr
implode(mixed ...$names): Phpolars\Expr
int_range(mixed $start, mixed $end = null, ?int $step = null): Phpolars\Expr
int_ranges(mixed $start, mixed $end = null, mixed $step = null): Phpolars\Expr
last(mixed ...$names): Phpolars\Expr
len(): Phpolars\Expr
linear_space(mixed $start, mixed $end, mixed $numSamples, ?string $closed = null): Phpolars\Expr
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
polars_version(): string
quantile(mixed $column, float $quantile, ?string $interpolation = null): Phpolars\Expr
repeat(mixed $value, mixed $n, ?string $dtype = null): Phpolars\Expr
select(mixed ...$exprs): Phpolars\DataFrame
sql(string $query, ?array $frames = null, ?bool $eager = null): mixed
std(mixed $column, ?int $ddof = null): Phpolars\Expr
struct_(mixed ...$exprs): Phpolars\Expr
sum(mixed ...$names): Phpolars\Expr
sum_horizontal(mixed ...$exprs): Phpolars\Expr
thread_pool_size(): int
var_(mixed $column, ?int $ddof = null): Phpolars\Expr
version(): string
when(mixed ...$conditions): Phpolars\When
```
