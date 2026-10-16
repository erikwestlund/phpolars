<?php
$sales = read_csv('sales.csv', tryParseDates: true);

$top = $sales
    ->filter(col('units')->gt(0))
    ->withColumns(col('units')->mul(col('price'))->alias('revenue'))
    ->groupBy('product', maintainOrder: true)
    ->agg(col('revenue')->sum()->round(2), ['orders' => len()], col('price')->mean()->round(2)->alias('avg_price'))
    ->sort('revenue', descending: true);

echo $top;
