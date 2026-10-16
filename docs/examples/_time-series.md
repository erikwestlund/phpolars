```php
$daily = read_csv('sales.csv', tryParseDates: true)
    ->groupBy('date')->agg(col('units')->sum())
    ->sort('date');

// Weeks starting on Monday.
echo $daily->groupByDynamic('date', every: '1w')->agg(col('units')->sum()->alias('week_units'))->head(3), "\n";

// The seven days ending at each row.
echo $daily->withColumns(col('units')->rollingSumBy('date', '7d')->alias('last_7d'))->head(4), "\n";

// The list price in force on each day.
$prices = new DataFrame(['from' => [new DateTimeImmutable('2024-01-01'), new DateTimeImmutable('2024-01-05')], 'list_price' => [5.0, 5.5]])
    ->cast(['from' => 'Date']);
echo $daily->head(4)->joinAsof($prices, leftOn: 'date', rightOn: 'from');
```

```{.text .output}
shape: (3, 2)
┌────────────┬────────────┐
│ date       ┆ week_units │
│ ---        ┆ ---        │
│ date       ┆ i64        │
╞════════════╪════════════╡
│ 2024-01-01 ┆ 109        │
│ 2024-01-08 ┆ 30         │
│ 2024-01-15 ┆ 71         │
└────────────┴────────────┘
shape: (4, 3)
┌────────────┬───────┬─────────┐
│ date       ┆ units ┆ last_7d │
│ ---        ┆ ---   ┆ ---     │
│ date       ┆ i64   ┆ i64     │
╞════════════╪═══════╪═════════╡
│ 2024-01-01 ┆ 15    ┆ 15      │
│ 2024-01-02 ┆ 23    ┆ 38      │
│ 2024-01-03 ┆ 6     ┆ 44      │
│ 2024-01-04 ┆ 19    ┆ 63      │
└────────────┴───────┴─────────┘
shape: (4, 4)
┌────────────┬───────┬────────────┬────────────┐
│ date       ┆ units ┆ from       ┆ list_price │
│ ---        ┆ ---   ┆ ---        ┆ ---        │
│ date       ┆ i64   ┆ date       ┆ f64        │
╞════════════╪═══════╪════════════╪════════════╡
│ 2024-01-01 ┆ 15    ┆ 2024-01-01 ┆ 5.0        │
│ 2024-01-02 ┆ 23    ┆ 2024-01-01 ┆ 5.0        │
│ 2024-01-03 ┆ 6     ┆ 2024-01-01 ┆ 5.0        │
│ 2024-01-04 ┆ 19    ┆ 2024-01-01 ┆ 5.0        │
└────────────┴───────┴────────────┴────────────┘
```
