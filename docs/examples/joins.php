<?php
$sales = read_csv('sales.csv')->select('id', 'store', 'units')->head(6);
$stores = read_csv('stores.csv')->filter(col('store')->ne('S2'));

echo $sales->join($stores, on: 'store', how: 'left', maintainOrder: 'left'), "\n";
echo $sales->join($stores, on: 'store', how: 'anti'), "\n";
echo $stores->rename(['store' => 'shop'])->join($sales, leftOn: 'shop', rightOn: 'store', how: 'inner')->height(), " matched rows\n";
