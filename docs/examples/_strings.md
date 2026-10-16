```php
$people = new DataFrame(['name' => ['  Ada Lovelace', 'grace hopper ', null, 'Alan Turing'], 'email' => ['ada@x.org', 'grace@navy.mil', 'n/a', 'alan@cam.ac.uk']]);

echo $people->select(
    col('name')->str()->stripChars()->str()->toUppercase()->alias('upper'),
    col('name')->str()->stripChars()->str()->split(' ')->list()->first()->alias('first'),
    col('email')->str()->extract('@(.+)$')->alias('domain'),
    col('email')->str()->contains('.org', literal: true)->alias('org'),
    col('email')->str()->lenChars()->alias('len'),
);
```

```{.text .output}
shape: (4, 5)
┌──────────────┬───────┬───────────┬───────┬─────┐
│ upper        ┆ first ┆ domain    ┆ org   ┆ len │
│ ---          ┆ ---   ┆ ---       ┆ ---   ┆ --- │
│ str          ┆ str   ┆ str       ┆ bool  ┆ u32 │
╞══════════════╪═══════╪═══════════╪═══════╪═════╡
│ ADA LOVELACE ┆ Ada   ┆ x.org     ┆ true  ┆ 9   │
│ GRACE HOPPER ┆ grace ┆ navy.mil  ┆ false ┆ 14  │
│ null         ┆ null  ┆ null      ┆ false ┆ 3   │
│ ALAN TURING  ┆ Alan  ┆ cam.ac.uk ┆ false ┆ 14  │
└──────────────┴───────┴───────────┴───────┴─────┘
```
