```php
$sales = read_csv('sales.csv');

echo $sales
    ->select(
        'id',
        'product',
        col('units')->fillNull(0),
        ['revenue' => col('units')->mul(col('price'))->round(2)],
        when(col('price')->ge(5))->then(lit('premium'))->otherwise(lit('standard'))->alias('tier'),
    )
    ->head(5);
```

```{.text .output}
shape: (5, 5)
┌─────┬─────────┬───────┬─────────┬──────────┐
│ id  ┆ product ┆ units ┆ revenue ┆ tier     │
│ --- ┆ ---     ┆ ---   ┆ ---     ┆ ---      │
│ i64 ┆ str     ┆ i64   ┆ f64     ┆ str      │
╞═════╪═════════╪═══════╪═════════╪══════════╡
│ 1   ┆ banana  ┆ 10    ┆ 70.3    ┆ premium  │
│ 2   ┆ cherry  ┆ 0     ┆ 0.0     ┆ standard │
│ 3   ┆ apple   ┆ 2     ┆ 7.68    ┆ standard │
│ 4   ┆ apple   ┆ 12    ┆ 88.08   ┆ premium  │
│ 5   ┆ apple   ┆ 1     ┆ 6.05    ┆ premium  │
└─────┴─────────┴───────┴─────────┴──────────┘
```
