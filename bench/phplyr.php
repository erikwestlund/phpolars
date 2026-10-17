<?php

// phplyr pipelines on three phibble backends: the in-memory DataFrame, PolarsTable,
// and SqlTable over SQLite. Times run through collect(); loading data is not timed.
//
//   php -d memory_limit=-1 -d extension=.../libphp_polars.so bench/phplyr.php PHPLYR_DIR ROWS [REPEATS]
//
// PHPLYR_DIR is a phplyr checkout with its dependencies installed, on a phibble
// that includes the Polars backend (Phibble\Polars\PolarsTable, phibble v0.2+).

declare(strict_types=1);

[$_, $phplyr, $rows, $repeats] = $argv + [null, null, '100000', '3'];
if ($phplyr === null) {
    fwrite(STDERR, "usage: php bench/phplyr.php PHPLYR_DIR ROWS [REPEATS]\n");
    exit(2);
}
require $phplyr.'/vendor/autoload.php';

use Phibble\Contracts\Table;
use Phibble\DataFrame;
use Phibble\Sql\SqlTable;
use Phibble\Polars\PolarsTable;

use function Phplyr\{arrange, col, cumsum, desc, filter, group_by, lag, left_join, mean, min_rank, mutate, n, sum, summarise};

$rows = (int) $rows;
$repeats = (int) $repeats;

// Data: sales (store 100 values, product 1,000, day 1..365, units with ~2% NA
// and ~5% zero, price) and a 100-row store lookup with 5 regions.
mt_srand(7);
$sales = ['id' => [], 'store' => [], 'product' => [], 'day' => [], 'units' => [], 'price' => []];
for ($i = 0; $i < $rows; $i++) {
    $sales['id'][] = $i + 1;
    $sales['store'][] = mt_rand(1, 100);
    $sales['product'][] = mt_rand(1, 1000);
    $sales['day'][] = mt_rand(1, 365);
    $r = mt_rand(1, 100);
    $sales['units'][] = $r <= 2 ? null : ($r <= 7 ? 0 : mt_rand(1, 20));
    $sales['price'][] = round(mt_rand(100, 10000) / 100, 2);
}
$stores = ['store' => range(1, 100), 'region' => array_map(fn ($s) => 'region-'.(($s - 1) % 5 + 1), range(1, 100))];

$frameSales = DataFrame::fromColumns($sales);
$frameStores = DataFrame::fromColumns($stores);

$pdo = new PDO('sqlite::memory:');
$pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$pdo->exec('CREATE TABLE sales (id INTEGER, store INTEGER, product INTEGER, day INTEGER, units INTEGER, price REAL)');
$pdo->exec('CREATE TABLE stores (store INTEGER, region TEXT)');
$pdo->beginTransaction();
$insert = $pdo->prepare('INSERT INTO sales VALUES (?, ?, ?, ?, ?, ?)');
for ($i = 0; $i < $rows; $i++) {
    $insert->execute([$sales['id'][$i], $sales['store'][$i], $sales['product'][$i], $sales['day'][$i], $sales['units'][$i], $sales['price'][$i]]);
}
$insert = $pdo->prepare('INSERT INTO stores VALUES (?, ?)');
foreach ($stores['store'] as $i => $s) {
    $insert->execute([$s, $stores['region'][$i]]);
}
$pdo->commit();

$backends = [
    'DataFrame' => [fn () => $frameSales, fn () => $frameStores],
    'PolarsTable' => [fn () => PolarsTable::fromTable($frameSales), fn () => PolarsTable::fromTable($frameStores)],
    'SqlTable' => [fn () => SqlTable::table($pdo, 'sales'), fn () => SqlTable::table($pdo, 'stores')],
];
// Wrap once outside the timing, so only the pipeline is measured.
$tables = array_map(fn (array $b) => [$b[0](), $b[1]()], $backends);
unset($sales);

$pipelines = [
    'A: filter, join, grouped summary, ranked window' => fn (Table $s, Table $st) => $s
        |> filter(col('units')->gt(0))
        |> left_join($st, by: 'store')
        |> group_by('region', 'product')
        |> summarise(n: n(), revenue: sum(col('units')->mul(col('price'))), avg_price: mean('price'))
        |> group_by('region')
        |> mutate(rank: min_rank(desc('revenue')))
        |> filter(col('rank')->le(10))
        |> arrange('region', 'rank'),
    'B: per-group windows over every row' => fn (Table $s, Table $st) => $s
        |> group_by('store')
        |> mutate(prev: lag('units', order_by: 'day'), running: cumsum('units'), share: col('units')->div(sum('units', na_rm: true))),
    'C: as A, with a PHP closure in filter()' => fn (Table $s, Table $st) => $s
        |> filter(fn ($r) => $r->units > 0)
        |> left_join($st, by: 'store')
        |> group_by('region', 'product')
        |> summarise(n: n(), revenue: sum(col('units')->mul(col('price'))), avg_price: mean('price'))
        |> group_by('region')
        |> mutate(rank: min_rank(desc('revenue')))
        |> filter(col('rank')->le(10))
        |> arrange('region', 'rank'),
];

/** Rows with floats at 9 significant digits, sorted, so float noise and row order don't count. */
function canonical(array $rows): string
{
    $rows = array_map(fn (array $r) => array_map(fn ($v) => is_float($v) ? sprintf('%.9g', $v) : $v, $r), $rows);
    $lines = array_map('json_encode', $rows);
    sort($lines);

    return md5(implode("\n", $lines));
}

printf("%s rows, best of %d, PHP %s, Polars threads %d\n\n", number_format($rows), $repeats, PHP_VERSION, Phpolars\thread_pool_size());
$results = [];
foreach ($pipelines as $name => $pipeline) {
    echo $name, "\n";
    $reference = null;
    foreach ($tables as $backend => [$sales, $stores]) {
        $best = INF;
        $out = null;
        $fallbacks = PolarsTable::$fallbacks;
        for ($k = 0; $k < $repeats; $k++) {
            gc_collect_cycles();
            $t = hrtime(true);
            try {
                $out = $pipeline($sales, $stores)->collect();
            } catch (Throwable $e) {
                $out = $e;
                break;
            }
            $best = min($best, (hrtime(true) - $t) / 1e9);
        }
        if ($out instanceof Throwable) {
            printf("  %-12s failed: %s\n", $backend, $out->getMessage());
            continue;
        }
        $hash = canonical($out->toRows());
        $reference ??= $hash;
        $note = $backend === 'PolarsTable' ? sprintf(', %d fallbacks per run', (PolarsTable::$fallbacks - $fallbacks) / $repeats) : '';
        printf("  %-12s %8.3f s  (%d rows%s)%s\n", $backend, $best, $out->nrow(), $note, $hash === $reference ? '' : '  RESULT DIFFERS');
        $results[$name][$backend] = $best;
    }
    echo "\n";
}
echo json_encode(['rows' => $rows, 'results' => $results]), "\n";
