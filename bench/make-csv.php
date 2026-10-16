<?php
// php bench/make-csv.php ROWS PATH: g (50 string groups), x (int, ~2% null), y (float).
[$_, $rows, $path] = $argv + [null, 1_000_000, 'bench/data.csv'];
mt_srand(42);
$out = fopen($path, 'w');
fwrite($out, "g,x,y\n");
for ($i = 0; $i < (int) $rows; $i++) {
    $x = mt_rand(1, 50) === 1 ? '' : mt_rand(0, 1000);
    fwrite($out, sprintf("g%02d,%s,%.4f\n", mt_rand(0, 49), $x, mt_rand() / mt_getrandmax() * 100));
}
fclose($out);
