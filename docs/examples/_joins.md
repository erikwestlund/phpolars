```php
$sales = read_csv('sales.csv')->select('id', 'store', 'units')->head(6);
$stores = read_csv('stores.csv')->filter(col('store')->ne('S2'));

echo $sales->join($stores, on: 'store', how: 'left', maintainOrder: 'left'), "\n";
echo $sales->join($stores, on: 'store', how: 'anti'), "\n";
echo $stores->rename(['store' => 'shop'])->join($sales, leftOn: 'shop', rightOn: 'store', how: 'inner')->height(), " matched rows\n";
```

```{.text .output}
shape: (6, 5)
┌─────┬───────┬───────┬──────────┬────────┐
│ id  ┆ store ┆ units ┆ city     ┆ region │
│ --- ┆ ---   ┆ ---   ┆ ---      ┆ ---    │
│ i64 ┆ str   ┆ i64   ┆ str      ┆ str    │
╞═════╪═══════╪═══════╪══════════╪════════╡
│ 1   ┆ S4    ┆ 10    ┆ Portland ┆ West   │
│ 2   ┆ S4    ┆ 0     ┆ Portland ┆ West   │
│ 3   ┆ S2    ┆ 2     ┆ null     ┆ null   │
│ 4   ┆ S3    ┆ 12    ┆ Atlanta  ┆ East   │
│ 5   ┆ S4    ┆ 1     ┆ Portland ┆ West   │
│ 6   ┆ S2    ┆ 6     ┆ null     ┆ null   │
└─────┴───────┴───────┴──────────┴────────┘
shape: (2, 3)
┌─────┬───────┬───────┐
│ id  ┆ store ┆ units │
│ --- ┆ ---   ┆ ---   │
│ i64 ┆ str   ┆ i64   │
╞═════╪═══════╪═══════╡
│ 3   ┆ S2    ┆ 2     │
│ 6   ┆ S2    ┆ 6     │
└─────┴───────┴───────┘
4 matched rows
```
