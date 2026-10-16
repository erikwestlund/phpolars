```php
$orders = new DataFrame([
    'order' => [1, 2, 3],
    'items' => [['apple', 'banana'], ['cherry'], []],
    'ship' => [['city' => 'Baltimore', 'days' => 2], ['city' => 'Denver', 'days' => 5], null],
]);

echo $orders, "\n";
echo $orders->select(
    'order',
    col('items')->list()->len()->alias('n_items'),
    col('items')->list()->join(', ')->alias('items'),
    col('ship')->struct()->field('city'),
), "\n";
echo $orders->explode('items'), "\n";
print_r($orders->row(0, named: true));
```

```{.text .output}
shape: (3, 3)
┌───────┬─────────────────────┬─────────────────┐
│ order ┆ items               ┆ ship            │
│ ---   ┆ ---                 ┆ ---             │
│ i64   ┆ list[str]           ┆ struct[2]       │
╞═══════╪═════════════════════╪═════════════════╡
│ 1     ┆ ["apple", "banana"] ┆ {"Baltimore",2} │
│ 2     ┆ ["cherry"]          ┆ {"Denver",5}    │
│ 3     ┆ []                  ┆ null            │
└───────┴─────────────────────┴─────────────────┘
shape: (3, 4)
┌───────┬─────────┬───────────────┬───────────┐
│ order ┆ n_items ┆ items         ┆ city      │
│ ---   ┆ ---     ┆ ---           ┆ ---       │
│ i64   ┆ u32     ┆ str           ┆ str       │
╞═══════╪═════════╪═══════════════╪═══════════╡
│ 1     ┆ 2       ┆ apple, banana ┆ Baltimore │
│ 2     ┆ 1       ┆ cherry        ┆ Denver    │
│ 3     ┆ 0       ┆               ┆ null      │
└───────┴─────────┴───────────────┴───────────┘
shape: (4, 3)
┌───────┬────────┬─────────────────┐
│ order ┆ items  ┆ ship            │
│ ---   ┆ ---    ┆ ---             │
│ i64   ┆ str    ┆ struct[2]       │
╞═══════╪════════╪═════════════════╡
│ 1     ┆ apple  ┆ {"Baltimore",2} │
│ 1     ┆ banana ┆ {"Baltimore",2} │
│ 2     ┆ cherry ┆ {"Denver",5}    │
│ 3     ┆ null   ┆ null            │
└───────┴────────┴─────────────────┘
Array
(
    [order] => 1
    [items] => Array
        (
            [0] => apple
            [1] => banana
        )

    [ship] => Array
        (
            [city] => Baltimore
            [days] => 2
        )

)
```
