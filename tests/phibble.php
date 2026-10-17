<?php
// php -d extension=... tests/phibble.php PHIBBLE_AUTOLOAD: round-trips through Phibble\DataFrame.
require __DIR__ . '/harness.php';

use Phibble\DataFrame as PhibbleDataFrame;
use Phpolars\DataFrame;

require $argv[1];

$columns = ['g' => ['a', 'b', null], 'x' => [1, null, 3], 'y' => [1.5, 2.0, null], 'ok' => [true, false, null]];
$df = new DataFrame($columns);
$frame = $df->toPhibble();
same('toPhibble returns a phibble DataFrame', $frame instanceof PhibbleDataFrame, true);
same('toPhibble keeps values', $frame->toColumns(), $columns);
same('fromPhibble round-trip', DataFrame::fromPhibble($frame)->toDict(asSeries: false), $columns);
same('fromPhibble after a phibble verb', DataFrame::fromPhibble($frame->filter(fn ($r) => $r->x !== null))->getColumn('g')->toList(), ['a', null]);
