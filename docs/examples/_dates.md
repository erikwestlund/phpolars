```php
$sales = read_csv('sales.csv', tryParseDates: true)->select('date')->head(4);

echo $sales->withColumns(
    col('date')->dt()->year()->alias('year'),
    col('date')->dt()->month()->alias('month'),
    col('date')->dt()->weekday()->alias('weekday'),
    col('date')->dt()->strftime('%b %d')->alias('label'),
    col('date')->dt()->truncate('1mo')->alias('month_start'),
), "\n";

// Date and Datetime values come back as DateTimeImmutable.
$first = $sales->item(0, 'date');
echo get_class($first), ' ', $first->format('Y-m-d'), "\n";
```

```{.text .output}
shape: (4, 6)
┌────────────┬──────┬───────┬─────────┬────────┬─────────────┐
│ date       ┆ year ┆ month ┆ weekday ┆ label  ┆ month_start │
│ ---        ┆ ---  ┆ ---   ┆ ---     ┆ ---    ┆ ---         │
│ date       ┆ i32  ┆ i8    ┆ i8      ┆ str    ┆ date        │
╞════════════╪══════╪═══════╪═════════╪════════╪═════════════╡
│ 2024-03-22 ┆ 2024 ┆ 3     ┆ 5       ┆ Mar 22 ┆ 2024-03-01  │
│ 2024-02-11 ┆ 2024 ┆ 2     ┆ 7       ┆ Feb 11 ┆ 2024-02-01  │
│ 2024-01-06 ┆ 2024 ┆ 1     ┆ 6       ┆ Jan 06 ┆ 2024-01-01  │
│ 2024-03-29 ┆ 2024 ┆ 3     ┆ 5       ┆ Mar 29 ┆ 2024-03-01  │
└────────────┴──────┴───────┴─────────┴────────┴─────────────┘
DateTimeImmutable 2024-03-22
```
