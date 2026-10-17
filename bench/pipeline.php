<?php
// php bench/pipeline.php CSV [phibble-autoload]
// read_csv -> filter x > 500 -> group_by g -> mean(y), count(x) -> toDicts, best of 5.
use function Phpolars\col;

$csv = $argv[1] ?? __DIR__ . '/data.csv';
if (isset($argv[2])) {
    require $argv[2];
}

function best(callable $f, int $n = 5): array {
    $times = [];
    for ($i = 0; $i < $n; $i++) {
        gc_collect_cycles();
        $t = hrtime(true);
        $out = $f();
        $times[] = (hrtime(true) - $t) / 1e6;
    }
    return [min($times), $out];
}

$runs = [];
if (extension_loaded('php_polars')) {
    $runs['phpolars'] = fn () => Phpolars\read_csv($csv)
        ->filter(col('x')->gt(500))
        ->groupBy('g')
        ->agg(col('y')->mean()->alias('mean_y'), col('x')->count()->alias('n'))
        ->sort('g')
        ->toDicts();
}
// Plain PHP arrays: a floor for any pure-PHP frame.
$runs['plain-php'] = function () use ($csv) {
    $in = fopen($csv, 'r');
    fgetcsv($in, escape: '');
    $acc = [];
    while (($r = fgetcsv($in, escape: '')) !== false) {
        if ($r[1] === '' || (int) $r[1] <= 500) {
            continue;
        }
        $acc[$r[0]][0] = ($acc[$r[0]][0] ?? 0) + (float) $r[2];
        $acc[$r[0]][1] = ($acc[$r[0]][1] ?? 0) + 1;
    }
    ksort($acc);
    $rows = [];
    foreach ($acc as $g => [$sum, $n]) {
        $rows[] = ['g' => $g, 'mean_y' => $sum / $n, 'n' => $n];
    }
    return $rows;
};
if (class_exists(\Phibble\DataFrame::class)) {
    $runs['phibble'] = fn () => \Phibble\DataFrame::fromCsv($csv)
        ->filter(fn ($r) => $r->x > 500)
        ->groupBy(['g'])
        ->summarise([
            'mean_y' => fn ($t) => array_sum($t->column('y')) / $t->nrow(),
            'n' => fn ($t) => $t->nrow(),
        ])
        ->arrange(['g'])
        ->toRows();
}

$results = [];
foreach ($runs as $name => $f) {
    [$ms, $rows] = best($f);
    $results[$name] = $rows;
    printf("%-10s %8.1f ms  (%d groups, peak PHP mem %.1f MB)\n", $name, $ms, count($rows), memory_get_peak_usage(true) / 1e6);
}
$b = $results['plain-php'];
foreach ($results as $name => $a) {
    $same = count($a) === count($b);
    foreach ($a as $i => $r) {
        $same = $same && $r['g'] === $b[$i]['g'] && $r['n'] === $b[$i]['n'] && abs($r['mean_y'] - $b[$i]['mean_y']) < 1e-9;
    }
    printf("%-10s %s\n", $name, $same ? 'matches plain-php' : 'DIFFERS from plain-php');
}
