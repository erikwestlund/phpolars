```php
use Phpolars\cs;

$sales = read_csv('sales.csv', tryParseDates: true)->head(3);

echo $sales->select(cs\numeric()->sub(cs\by_name('id'))), "\n";
echo $sales->select(cs\string()->or(cs\date())), "\n";
echo $sales->withColumns(cs\float()->round(0)->name()->suffix('_rounded'))->select(cs\ends_with('price', '_rounded'));
```

```{.text .output}
shape: (3, 2)
┌───────┬───────┐
│ units ┆ price │
│ ---   ┆ ---   │
│ i64   ┆ f64   │
╞═══════╪═══════╡
│ 10    ┆ 7.03  │
│ 0     ┆ 2.46  │
│ 2     ┆ 3.84  │
└───────┴───────┘
shape: (3, 3)
┌────────────┬───────┬─────────┐
│ date       ┆ store ┆ product │
│ ---        ┆ ---   ┆ ---     │
│ date       ┆ str   ┆ str     │
╞════════════╪═══════╪═════════╡
│ 2024-03-22 ┆ S4    ┆ banana  │
│ 2024-02-11 ┆ S4    ┆ cherry  │
│ 2024-01-06 ┆ S2    ┆ apple   │
└────────────┴───────┴─────────┘
shape: (3, 2)
┌───────┬───────────────┐
│ price ┆ price_rounded │
│ ---   ┆ ---           │
│ f64   ┆ f64           │
╞═══════╪═══════════════╡
│ 7.03  ┆ 7.0           │
│ 2.46  ┆ 2.0           │
│ 3.84  ┆ 4.0           │
└───────┴───────────────┘
```
