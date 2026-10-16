<?php
$sales = read_csv('sales.csv');

// Several predicates are combined with and; null counts as false.
echo $sales->filter(
    col('product')->isIn(['apple', 'cherry']),
    col('units')->isBetween(5, 8),
    col('store')->ne('S1'),
)->head(5);
