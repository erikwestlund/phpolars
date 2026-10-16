```php
$sales = read_csv('sales.csv', tryParseDates: true)->filter(col('store')->eq('S1'))->head(8);

echo $sales->select(
    'date',
    'product',
    'units',
    col('units')->sum()->over('product')->alias('product_total'),
    col('units')->cumSum()->over('product', orderBy: 'date')->alias('running'),
    col('units')->shift()->over('product', orderBy: 'date')->alias('previous'),
    col('units')->rank('dense', descending: true)->alias('rank'),
);
```

```{.text .output}
shape: (8, 7)
┌────────────┬─────────┬───────┬───────────────┬─────────┬──────────┬──────┐
│ date       ┆ product ┆ units ┆ product_total ┆ running ┆ previous ┆ rank │
│ ---        ┆ ---     ┆ ---   ┆ ---           ┆ ---     ┆ ---      ┆ ---  │
│ date       ┆ str     ┆ i64   ┆ i64           ┆ i64     ┆ i64      ┆ u32  │
╞════════════╪═════════╪═══════╪═══════════════╪═════════╪══════════╪══════╡
│ 2024-01-04 ┆ banana  ┆ 10    ┆ 23            ┆ 11      ┆ 1        ┆ 3    │
│ 2024-02-02 ┆ banana  ┆ 6     ┆ 23            ┆ 17      ┆ 10       ┆ 5    │
│ 2024-01-22 ┆ apple   ┆ 5     ┆ 16            ┆ 5       ┆ null     ┆ 6    │
│ 2024-01-23 ┆ apple   ┆ 11    ┆ 16            ┆ 16      ┆ 5        ┆ 2    │
│ 2024-01-03 ┆ banana  ┆ 1     ┆ 23            ┆ 1       ┆ null     ┆ 7    │
│ 2024-03-05 ┆ date    ┆ 7     ┆ 19            ┆ 19      ┆ 12       ┆ 4    │
│ 2024-03-22 ┆ banana  ┆ 6     ┆ 23            ┆ 23      ┆ 6        ┆ 5    │
│ 2024-02-03 ┆ date    ┆ 12    ┆ 19            ┆ 12      ┆ null     ┆ 1    │
└────────────┴─────────┴───────┴───────────────┴─────────┴──────────┴──────┘
```
