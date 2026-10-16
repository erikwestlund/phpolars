# phpolars-phibble

`Phpolars\Phibble\PolarsTable` implements `Phibble\Contracts\Table` on top of a Polars `DataFrame`, so phibble and phplyr pipelines can run in Polars without changes. It needs the [php_polars](https://github.com/erikwestlund/phpolars) extension.

```php
use Phpolars\Phibble\PolarsTable;
use function Phibble\Expr\{col, mean, n};

$table = PolarsTable::fromPolars(Phpolars\read_csv('schools.csv'));
// or: PolarsTable::fromTable($phibbleFrame)

$summary = $table
    ->groupBy(['district'])
    ->filter(col('score')->gt(mean(col('score'), na_rm: true)))
    ->summarise(['n' => n(), 'avg' => mean(col('score'))]);

$frame = $summary->collect();   // a Phibble\Frame
```

## What runs in Polars

Verbs given column names or `Phibble\Expr` expressions run in Polars: `select`, `filter`, `mutate`, `arrange`, `summarise`, `distinct`, `take`, `join`, `rename`, `slice`, `head`, `groupBy`, and `eachGroup`. The translation follows dplyr semantics, the same ones phibble's in-memory `Frame` uses:

- NA propagates and logic is three-valued.
- Aggregates default to `na_rm = FALSE`, so an NA input gives an NA result.
- In a grouped `mutate()` or `filter()`, aggregates and window functions run per group.
- Groups are visited and summarised in sorted key order, with factors in level order and NA last.
- Joins follow dplyr 1.1: row order, `.x`/`.y` suffixes, key coalescing, `multiple`, `relationship`, `unmatched`, and `naMatches`.
- `round()` follows R 4: ties go to the even digit, judged against the stored double.
- `quantile()` and `iqr()` over NA with `na_rm = FALSE` raise R's error, checked before the verb runs.

PHP closures cannot run on Polars threads. A verb given a closure (`filter(fn ($r) => ...)`, `mutate(['x' => fn ($r) => ...])`, `mutateGroups()`, `filterGroups()`) and every verb on a rowwise table materializes a `Phibble\Frame`, runs there, and converts the result back to Polars. `PolarsTable::$fallbacks` counts these fallbacks.

Factor columns are held as strings in Polars. Their levels travel with the table, so group order and `collect()` keep them.

## Known differences from the in-memory Frame

- `as_string()` on doubles formats to 15 significant digits, as R does. Very large or small values can differ from R's scientific notation.
- `mean()`, `min()`, and `max()` over zero rows return NA. R and the Frame return NaN, Inf, and -Inf. The contract allows this difference.
- Regex is not used: `str_detect()`, `str_starts()`, and `str_ends()` take fixed patterns, as the contract specifies.

## Tests

The tests run phibble's own dplyr reference cases (`tests/Support/ExprCases.php` and `tests/fixtures/expr/expected.json` from the installed phibble) and its contract v1 cases against `PolarsTable`. `native()` asserts that a verb ran in Polars without a fallback.

```sh
composer install
php -d extension=/path/to/libphp_polars.so vendor/bin/pest
```
