<?php
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
