```php
$sales = read_csv('sales.csv');

// Several predicates are combined with and; null counts as false.
echo $sales->filter(
    col('product')->isIn(['apple', 'cherry']),
    col('units')->isBetween(5, 8),
    col('store')->ne('S1'),
)->head(5);
```

```{.text .output}
shape: (5, 6)
┌─────┬────────────┬───────┬─────────┬───────┬───────┐
│ id  ┆ date       ┆ store ┆ product ┆ units ┆ price │
│ --- ┆ ---        ┆ ---   ┆ ---     ┆ ---   ┆ ---   │
│ i64 ┆ str        ┆ str   ┆ str     ┆ i64   ┆ f64   │
╞═════╪════════════╪═══════╪═════════╪═══════╪═══════╡
│ 12  ┆ 2024-03-17 ┆ S2    ┆ apple   ┆ 8     ┆ 3.85  │
│ 16  ┆ 2024-02-24 ┆ S3    ┆ apple   ┆ 8     ┆ 5.07  │
│ 20  ┆ 2024-03-26 ┆ S4    ┆ cherry  ┆ 7     ┆ 5.72  │
│ 22  ┆ 2024-03-05 ┆ S2    ┆ apple   ┆ 5     ┆ 8.23  │
│ 24  ┆ 2024-03-26 ┆ S3    ┆ cherry  ┆ 6     ┆ 2.2   │
└─────┴────────────┴───────┴─────────┴───────┴───────┘
```
