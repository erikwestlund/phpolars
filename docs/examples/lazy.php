<?php
$query = scan_csv('sales.csv')
    ->filter(col('units')->gt(5))
    ->groupBy('store')
    ->agg(col('units')->sum())
    ->sort('store');

echo $query->explain(), "\n\n";
echo $query->collect();
