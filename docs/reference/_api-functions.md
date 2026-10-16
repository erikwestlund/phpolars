```php
all(): Phpolars\Expr
all_horizontal(mixed ...$exprs): Phpolars\Expr
any_horizontal(mixed ...$exprs): Phpolars\Expr
coalesce(mixed ...$exprs): Phpolars\Expr
col(mixed ...$names): Phpolars\Expr
concat(array $items, ?string $how = null): mixed
concat_list(mixed ...$exprs): Phpolars\Expr
concat_str(mixed $exprs, ?string $separator = null, ?bool $ignoreNulls = null): Phpolars\Expr
count(mixed ...$names): Phpolars\Expr
element(): Phpolars\Expr
exclude(mixed ...$names): Phpolars\Expr
first(mixed ...$names): Phpolars\Expr
int_range(mixed $start, mixed $end = null, ?int $step = null): Phpolars\Expr
last(mixed ...$names): Phpolars\Expr
len(): Phpolars\Expr
lit(mixed $value, ?string $dtype = null): Phpolars\Expr
max(mixed ...$names): Phpolars\Expr
max_horizontal(mixed ...$exprs): Phpolars\Expr
mean(mixed ...$names): Phpolars\Expr
mean_horizontal(mixed ...$exprs): Phpolars\Expr
median(mixed ...$names): Phpolars\Expr
min(mixed ...$names): Phpolars\Expr
min_horizontal(mixed ...$exprs): Phpolars\Expr
n_unique(mixed ...$names): Phpolars\Expr
struct_(mixed ...$exprs): Phpolars\Expr
sum(mixed ...$names): Phpolars\Expr
sum_horizontal(mixed ...$exprs): Phpolars\Expr
thread_pool_size(): int
version(): string
when(mixed ...$conditions): Phpolars\When
```
