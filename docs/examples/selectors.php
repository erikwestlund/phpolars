<?php
use Phpolars\cs;

$sales = read_csv('sales.csv', tryParseDates: true)->head(3);

echo $sales->select(cs\numeric()->sub(cs\by_name('id'))), "\n";
echo $sales->select(cs\string()->or(cs\date())), "\n";
echo $sales->withColumns(cs\float()->round(0)->name()->suffix('_rounded'))->select(cs\ends_with('price', '_rounded'));
