<?php
require dirname(getcwd(), 2).'/phibble/vendor/autoload.php';

use Phibble\Frame;
use Phpolars\Phibble\PolarsTable;
use function Phibble\Expr\{col as pcol, mean, n};

$table = PolarsTable::fromPolars(read_csv('sales.csv'));

$out = $table
    ->groupBy(['store'])
    ->filter(pcol('units')->gt(mean(pcol('units'), na_rm: true)))
    ->summarise(['orders' => n(), 'avg_units' => mean(pcol('units'))]);

echo get_class($out), "\n";
echo $out->collect(), "\n";
echo PolarsTable::$fallbacks, " fallbacks to an in-memory Frame\n";
