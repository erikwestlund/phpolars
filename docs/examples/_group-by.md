```php
$sales = read_csv('sales.csv');

echo $sales->groupBy(['store', 'product'], maintainOrder: true)
    ->agg(
        len()->alias('rows'),
        col('units')->sum(),
        col('units')->nullCount()->alias('missing_units'),
        col('price')->max()->alias('top_price'),
    )
    ->sort(['store', 'product'])
    ->head(6), "\n";

// Shorthands aggregate every non-key column.
echo $sales->select('store', 'units', 'price')->groupBy('store')->mean()->sort('store');
```

```{.text .output}
shape: (6, 6)
┌───────┬─────────┬──────┬───────┬───────────────┬───────────┐
│ store ┆ product ┆ rows ┆ units ┆ missing_units ┆ top_price │
│ ---   ┆ ---     ┆ ---  ┆ ---   ┆ ---           ┆ ---       │
│ str   ┆ str     ┆ u32  ┆ i64   ┆ u32           ┆ f64       │
╞═══════╪═════════╪══════╪═══════╪═══════════════╪═══════════╡
│ S1    ┆ apple   ┆ 17   ┆ 98    ┆ 0             ┆ 7.23      │
│ S1    ┆ banana  ┆ 15   ┆ 81    ┆ 0             ┆ 8.98      │
│ S1    ┆ cherry  ┆ 12   ┆ 90    ┆ 0             ┆ 8.07      │
│ S1    ┆ date    ┆ 13   ┆ 109   ┆ 0             ┆ 8.57      │
│ S2    ┆ apple   ┆ 11   ┆ 47    ┆ 0             ┆ 8.23      │
│ S2    ┆ banana  ┆ 14   ┆ 69    ┆ 1             ┆ 6.92      │
└───────┴─────────┴──────┴───────┴───────────────┴───────────┘
shape: (4, 3)
┌───────┬──────────┬──────────┐
│ store ┆ units    ┆ price    │
│ ---   ┆ ---      ┆ ---      │
│ str   ┆ f64      ┆ f64      │
╞═══════╪══════════╪══════════╡
│ S1    ┆ 6.631579 ┆ 4.010702 │
│ S2    ┆ 5.282609 ┆ 4.237843 │
│ S3    ┆ 6.282609 ┆ 4.877347 │
│ S4    ┆ 4.883721 ┆ 4.074186 │
└───────┴──────────┴──────────┘
```
