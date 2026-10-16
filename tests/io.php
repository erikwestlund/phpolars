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

// Reader and writer options
file_put_contents("$dir/opts.csv", "# a comment\nx|y\n1|'a|b'\n2|c,d\n3|\n");
$o = read_csv("$dir/opts.csv", separator: '|', commentPrefix: '#', quoteChar: "'", missingUtf8IsEmptyString: true);
same('read_csv commentPrefix and quoteChar', $o->getColumn('y')->toList(), ['a|b', 'c,d', '']);
same('read_csv rowIndexName', read_csv("$dir/in.csv", separator: ';', rowIndexName: 'row', rowIndexOffset: 10)->getColumn('row')->toList(), [10, 11, 12]);
same('read_csv newColumns', read_csv("$dir/in.csv", separator: ';', newColumns: ['a', 'b', 'c'])->columns(), ['a', 'b', 'c']);
same('read_csv schema', read_csv("$dir/in.csv", separator: ';', schema: ['id' => 'Int32', 'name' => 'String', 'score' => 'Float32'])->dtypes(), ['Int32', 'String', 'Float32']);
same('read_csv skipRowsAfterHeader', read_csv("$dir/in.csv", separator: ';', skipRowsAfterHeader: 2)->getColumn('id')->toList(), [3]);
file_put_contents("$dir/ragged.csv", "a,b\n1,2\n3,4,5\n");
throws('ragged lines raise', fn () => read_csv("$dir/ragged.csv"), Phpolars\PolarsError::class);
same('truncateRaggedLines', read_csv("$dir/ragged.csv", truncateRaggedLines: true)->height(), 2);
file_put_contents("$dir/comma.csv", "a;b\n1,5;x\n");
same('decimalComma', read_csv("$dir/comma.csv", separator: ';', decimalComma: true)->item(0, 'a'), 1.5);
file_put_contents("$dir/empty.csv", "");
same('raiseIfEmpty: false', read_csv("$dir/empty.csv", raiseIfEmpty: false)->height(), 0);
$df->writeCsv("$dir/w.csv", quoteStyle: 'always', lineTerminator: "\r\n", floatPrecision: 2);
same('writeCsv quoteStyle and floatPrecision', explode("\r\n", file_get_contents("$dir/w.csv"))[1], '"1","a","1.50","true"');
$dates = new DataFrame(['d' => [new DateTimeImmutable('2024-03-09')]]);
$dates->cast(['d' => 'Date'])->writeCsv("$dir/d.csv", dateFormat: '%d/%m/%Y');
same('writeCsv dateFormat', trim(explode("\n", file_get_contents("$dir/d.csv"))[1]), '09/03/2024');
$big = new DataFrame(['v' => range(1, 5000)]);
$big->writeParquet("$dir/gz.parquet", compression: 'gzip', compressionLevel: 9, rowGroupSize: 1000);
same('writeParquet gzip', read_parquet("$dir/gz.parquet")->equals($big), true);
$big->writeParquet("$dir/plain.parquet", compression: 'uncompressed', statistics: false);
same('writeParquet uncompressed', read_parquet("$dir/plain.parquet")->height(), 5000);
throws('bad compression', fn () => $big->writeParquet("$dir/x.parquet", compression: 'zip'), ValueError::class, 'compression must be');
throws('bad zstd level', fn () => $big->writeParquet("$dir/x.parquet", compressionLevel: 99), Phpolars\PolarsError::class);
same('read_parquet rowIndexName', read_parquet("$dir/out.parquet", rowIndexName: 'i')->columns()[0], 'i');
@mkdir("$dir/parts");
$df->head(1)->writeParquet("$dir/parts/a.parquet");
$df->tail(2)->writeParquet("$dir/parts/b.parquet");
same('read_parquet glob', read_parquet("$dir/parts/*.parquet")->height(), 3);
same('includeFilePaths', count(array_unique(read_parquet("$dir/parts/*.parquet", includeFilePaths: 'file')->getColumn('file')->toList())), 2);
same('read_ipc columns', read_ipc("$dir/out.arrow", columns: ['name'])->columns(), ['name']);
same('read_ipc rowIndexName', read_ipc("$dir/out.arrow", rowIndexName: 'i')->columns()[0], 'i');
same('read_ndjson schemaOverrides', read_ndjson("$dir/out.ndjson", schemaOverrides: ['id' => 'Float64'])->dtypes()[0], 'Float64');
same('read_ndjson columns', read_ndjson("$dir/out.ndjson", columns: ['ok'])->columns(), ['ok']);
same('read_json schema', read_json("$dir/out.json", schema: ['id' => 'Int16', 'name' => 'String', 'score' => 'Float64', 'ok' => 'Boolean'])->dtypes()[0], 'Int16');

// Databases through PDO
if (extension_loaded('pdo_sqlite')) {
    $pdo = new PDO('sqlite::memory:', options: [PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION]);
    $pdo->exec('CREATE TABLE t (id INTEGER, name TEXT, score REAL)');
    $pdo->exec("INSERT INTO t VALUES (1, 'a', 1.5), (2, NULL, 2.5)");
    $db = Phpolars\read_database('SELECT * FROM t ORDER BY id', $pdo);
    same('read_database', $db->toDicts(), [['id' => 1, 'name' => 'a', 'score' => 1.5], ['id' => 2, 'name' => null, 'score' => 2.5]]);
    same('read_database params', Phpolars\read_database('SELECT name FROM t WHERE id = ?', $pdo, params: [1])->item(), 'a');
    same('read_database empty keeps columns', Phpolars\read_database('SELECT * FROM t WHERE id > 9', $pdo)->columns(), ['id', 'name', 'score']);
    same('read_database schemaOverrides', Phpolars\read_database('SELECT id FROM t', $pdo, schemaOverrides: ['id' => 'Int8'])->dtypes(), ['Int8']);
    throws('read_database bad query', fn () => Phpolars\read_database('SELECT nope FROM t', $pdo), PDOException::class);
    $out = new DataFrame(['id' => [1, 2], 'name' => ['a', null], 'ok' => [true, false], 'd' => [new DateTimeImmutable('2024-01-02'), null], 'l' => [[1, 2], []]]);
    $out = $out->cast(['d' => 'Date']);
    same('writeDatabase rows', $out->writeDatabase('w', $pdo), 2);
    same('writeDatabase round-trip', Phpolars\read_database('SELECT * FROM w', $pdo)->toDicts(), [
        ['id' => 1, 'name' => 'a', 'ok' => 1, 'd' => '2024-01-02', 'l' => '[1,2]'],
        ['id' => 2, 'name' => null, 'ok' => 0, 'd' => null, 'l' => '[]'],
    ]);
    throws('writeDatabase existing table', fn () => $out->writeDatabase('w', $pdo), ValueError::class, 'already exists');
    same('writeDatabase append', $out->writeDatabase('w', $pdo, ifTableExists: 'append'), 2);
    same('appended', Phpolars\read_database('SELECT COUNT(*) AS n FROM w', $pdo)->item(), 4);
    $out->head(1)->writeDatabase('w', $pdo, ifTableExists: 'replace');
    same('replaced', Phpolars\read_database('SELECT COUNT(*) AS n FROM w', $pdo)->item(), 1);
    $big = new DataFrame(['v' => range(1, 2500), 'w' => array_fill(0, 2500, 'x')]);
    same('writeDatabase in batches', $big->writeDatabase('big', $pdo), 2500);
    $dates = Phpolars\read_database('SELECT d FROM w', $pdo, schemaOverrides: ['d' => 'Date']);
    same('schemaOverrides parses dates', [$dates->dtypes(), $dates->item()->format('Y-m-d')], [['Date'], '2024-01-02']);
    same('batched rows arrive', Phpolars\read_database('SELECT SUM(v) AS s FROM big', $pdo)->item(), 3126250);
}

array_map('unlink', glob("$dir/parts/*"));
rmdir("$dir/parts");
array_map('unlink', glob("$dir/*"));
rmdir($dir);
