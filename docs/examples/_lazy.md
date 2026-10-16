```php
$query = scan_csv('sales.csv')
    ->filter(col('units')->gt(5))
    ->groupBy('store')
    ->agg(col('units')->sum())
    ->sort('store');

echo $query->explain(), "\n\n";
echo $query->collect();
```

```{.text .output}
SORT BY [col("store")]
  AGGREGATE[maintain_order: false]
    [col("units").sum()] BY [col("store")]
    FROM
    Csv SCAN [sales.csv]
    PROJECT 2/6 COLUMNS
    SELECTION: col("units") > 5
    ESTIMATED ROWS: 201

shape: (4, 2)
┌───────┬───────┐
│ store ┆ units │
│ ---   ┆ ---   │
│ str   ┆ i64   │
╞═══════╪═══════╡
│ S1    ┆ 335   │
│ S2    ┆ 199   │
│ S3    ┆ 230   │
│ S4    ┆ 136   │
└───────┴───────┘
```
