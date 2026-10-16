<?php
$dir = sys_get_temp_dir().'/phpolars-docs';
@mkdir($dir);
$sales = read_csv('sales.csv', tryParseDates: true);

$sales->writeParquet("$dir/sales.parquet");
$sales->writeNdjson("$dir/sales.ndjson");
$sales->lazy()->filter(col('store')->eq('S2'))->sinkCsv("$dir/s2.csv");

echo read_parquet("$dir/sales.parquet")->equals($sales) ? "parquet round-trip ok\n" : "parquet differs\n";
echo scan_parquet("$dir/sales.parquet")->select(col('units')->sum())->collect()->item(), " units\n";
echo read_csv("$dir/s2.csv")->height(), " rows for S2\n";
echo read_ndjson("$dir/sales.ndjson")->schema()['date'], "\n";
