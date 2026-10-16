<?php
$sales = read_csv('sales.csv', tryParseDates: true)->filter(col('store')->eq('S1'))->head(8);

echo $sales->select(
    'date',
    'product',
    'units',
    col('units')->sum()->over('product')->alias('product_total'),
    col('units')->cumSum()->over('product', orderBy: 'date')->alias('running'),
    col('units')->shift()->over('product', orderBy: 'date')->alias('previous'),
    col('units')->rank('dense', descending: true)->alias('rank'),
);
