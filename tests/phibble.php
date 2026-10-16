<?php
// php -d extension=... tests/phibble.php PHIBBLE_AUTOLOAD; round-trips through Phibble\Frame.
use Phibble\Frame;
use Phpolars\DataFrame;

require $argv[1];

function check(string $name, mixed $actual, mixed $expected): void {
    if ($actual !== $expected) {
        fwrite(STDERR, "FAIL $name\nexpected: " . var_export($expected, true) . "\nactual:   " . var_export($actual, true) . "\n");
        exit(1);
    }
    echo "ok $name\n";
}

$columns = ['g' => ['a', 'b', null], 'x' => [1, null, 3], 'y' => [1.5, 2.0, null], 'ok' => [true, false, null]];
$df = DataFrame::fromColumns($columns);
$frame = $df->toPhibble();
check('toPhibble returns a Frame', $frame instanceof Frame, true);
check('toPhibble keeps values', $frame->toColumns(), $columns);
check('fromPhibble round-trip', DataFrame::fromPhibble($frame)->toColumns(), $columns);
check('fromPhibble after a phibble verb', DataFrame::fromPhibble($frame->filter(fn ($r) => $r->x !== null))->toColumns()['g'], ['a', null]);
