<?php
$sales = read_csv('sales.csv', tryParseDates: true)->select('date')->head(4);

echo $sales->withColumns(
    col('date')->dt()->year()->alias('year'),
    col('date')->dt()->month()->alias('month'),
    col('date')->dt()->weekday()->alias('weekday'),
    col('date')->dt()->strftime('%b %d')->alias('label'),
    col('date')->dt()->truncate('1mo')->alias('month_start'),
), "\n";

// Date and Datetime values come back as DateTimeImmutable.
$first = $sales->item(0, 'date');
echo get_class($first), ' ', $first->format('Y-m-d'), "\n";
