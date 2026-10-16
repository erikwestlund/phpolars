```php
$sales = read_csv('sales.csv', tryParseDates: true);

$top = $sales
    ->filter(col('units')->gt(0))
    ->withColumns(col('units')->mul(col('price'))->alias('revenue'))
    ->groupBy('product', maintainOrder: true)
    ->agg(col('revenue')->sum()->round(2), ['orders' => len()], col('price')->mean()->round(2)->alias('avg_price'))
    ->sort('revenue', descending: true);

echo $top;
```

```{.text .output}
shape: (4, 4)
┌─────────┬─────────┬────────┬───────────┐
│ product ┆ revenue ┆ orders ┆ avg_price │
│ ---     ┆ ---     ┆ ---    ┆ ---       │
│ str     ┆ f64     ┆ u32    ┆ f64       │
╞═════════╪═════════╪════════╪═══════════╡
│ cherry  ┆ 1307.38 ┆ 44     ┆ 4.09      │
│ banana  ┆ 1285.76 ┆ 48     ┆ 4.35      │
│ apple   ┆ 1227.68 ┆ 42     ┆ 4.38      │
│ date    ┆ 1153.58 ┆ 41     ┆ 4.46      │
└─────────┴─────────┴────────┴───────────┘
```
