<?php
require __DIR__ . '/harness.php';

use Phpolars\DataFrame;
use function Phpolars\{col, read_csv, scan_csv, read_parquet, scan_parquet, read_ipc, scan_ipc, read_ndjson, scan_ndjson, read_json};

$dir = sys_get_temp_dir() . '/phpolars-io-' . getmypid();
@mkdir($dir);
$df = new DataFrame(['id' => [1, 2, 3], 'name' => ['a', null, 'c'], 'score' => [1.5, 2.5, null], 'ok' => [true, false, null]]);

// CSV
file_put_contents("$dir/in.csv", "id;name;score\n1;a;1.5\n2;NA;2.5\n3;c;\n");
$csv = read_csv("$dir/in.csv", separator: ';', nullValues: 'NA');
same('read_csv', $csv->toDicts()[1], ['id' => 2, 'name' => null, 'score' => 2.5]);
same('read_csv schemaOverrides', read_csv("$dir/in.csv", separator: ';', schemaOverrides: ['id' => 'Float64'])->dtypes()[0], 'Float64');
same('read_csv columns', read_csv("$dir/in.csv", separator: ';', columns: ['score'])->columns(), ['score']);
same('read_csv nRows', read_csv("$dir/in.csv", separator: ';', nRows: 2)->height(), 2);
same('scan_csv lazy', scan_csv("$dir/in.csv", separator: ';')->filter(col('id')->gt(1))->collect()->height(), 2);
$df->writeCsv("$dir/out.csv");
same('writeCsv round-trip', read_csv("$dir/out.csv")->equals($df), true);
$df->writeCsv("$dir/out2.csv", separator: "\t", nullValue: 'NULL');
same('writeCsv options', explode("\n", file_get_contents("$dir/out2.csv"))[2], "2\tNULL\t2.5\tfalse");
throws('read_csv missing file', fn () => read_csv("$dir/nope.csv"), Phpolars\PolarsError::class);

// Parquet, IPC, NDJSON, JSON
$df->writeParquet("$dir/out.parquet");
same('parquet round-trip', read_parquet("$dir/out.parquet")->equals($df), true);
same('read_parquet columns', read_parquet("$dir/out.parquet", columns: ['id', 'ok'])->columns(), ['id', 'ok']);
same('scan_parquet pushdown', scan_parquet("$dir/out.parquet")->select(col('score')->sum())->collect()->item(), 4.0);
$df->writeIpc("$dir/out.arrow");
same('ipc round-trip', read_ipc("$dir/out.arrow")->equals($df), true);
same('scan_ipc nRows', scan_ipc("$dir/out.arrow", nRows: 1)->collect()->height(), 1);
$df->writeNdjson("$dir/out.ndjson");
same('ndjson lines', count(file("$dir/out.ndjson")), 3);
same('ndjson round-trip', read_ndjson("$dir/out.ndjson")->equals($df), true);
same('scan_ndjson', scan_ndjson("$dir/out.ndjson")->select('name')->collect()->getColumn('name')->toList(), ['a', null, 'c']);
$df->writeJson("$dir/out.json");
same('json is an array of rows', json_decode(file_get_contents("$dir/out.json"), true)[0], ['id' => 1, 'name' => 'a', 'score' => 1.5, 'ok' => true]);
same('json round-trip', read_json("$dir/out.json")->equals($df), true);

// Lazy sinks
$df->lazy()->filter(col('id')->ge(2))->sinkParquet("$dir/sink.parquet");
same('sinkParquet', read_parquet("$dir/sink.parquet")->height(), 2);
$df->lazy()->sinkCsv("$dir/sink.csv");
same('sinkCsv', read_csv("$dir/sink.csv")->height(), 3);
$df->lazy()->sinkIpc("$dir/sink.arrow");
same('sinkIpc', read_ipc("$dir/sink.arrow")->width(), 4);
$df->lazy()->sinkNdjson("$dir/sink.ndjson");
same('sinkNdjson', read_ndjson("$dir/sink.ndjson")->height(), 3);

array_map('unlink', glob("$dir/*"));
rmdir($dir);
