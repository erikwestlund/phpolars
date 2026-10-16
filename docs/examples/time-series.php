<?php
$daily = read_csv('sales.csv', tryParseDates: true)
    ->groupBy('date')->agg(col('units')->sum())
    ->sort('date');

// Weeks starting on Monday.
echo $daily->groupByDynamic('date', every: '1w')->agg(col('units')->sum()->alias('week_units'))->head(3), "\n";

// The seven days ending at each row.
echo $daily->withColumns(col('units')->rollingSumBy('date', '7d')->alias('last_7d'))->head(4), "\n";

// The list price in force on each day.
$prices = new DataFrame(['from' => [new DateTimeImmutable('2024-01-01'), new DateTimeImmutable('2024-01-05')], 'list_price' => [5.0, 5.5]])
    ->cast(['from' => 'Date']);
echo $daily->head(4)->joinAsof($prices, leftOn: 'date', rightOn: 'from');
