<?php
$sales = read_csv('sales.csv');

echo $sales
    ->select(
        'id',
        'product',
        col('units')->fillNull(0),
        ['revenue' => col('units')->mul(col('price'))->round(2)],
        when(col('price')->ge(5))->then(lit('premium'))->otherwise(lit('standard'))->alias('tier'),
    )
    ->head(5);
