<?php
// php -d extension=... tests/dataframe.php; exits non-zero on the first failure.
use Phpolars\DataFrame;
use function Phpolars\{col, lit};

function check(string $name, mixed $actual, mixed $expected): void {
    if ($actual !== $expected) {
        fwrite(STDERR, "FAIL $name\nexpected: " . var_export($expected, true) . "\nactual:   " . var_export($actual, true) . "\n");
        exit(1);
    }
    echo "ok $name\n";
}

function throws(string $name, callable $f, string $message): void {
    try {
        $f();
    } catch (Exception $e) {
        check($name, str_contains($e->getMessage(), $message), true);
        return;
    }
    check($name, 'no exception', $message);
}

$csv = tempnam(sys_get_temp_dir(), 'phpolars') . '.csv';
file_put_contents($csv, "g,x,y\na,1,2.5\nb,2,\na,3,1.5\nc,,4.0\n");
$df = DataFrame::readCsv($csv);
unlink($csv);

check('columns', $df->columns(), ['g', 'x', 'y']);
check('shape', [$df->height(), $df->width()], [4, 3]);
check('types round-trip', $df->toRows()[1], ['g' => 'b', 'x' => 2, 'y' => null]);
check('filter, null is false', $df->filter(col('x')->gte(2))->toColumns()['g'], ['b', 'a']);
check('filter and', $df->filter(col('x')->gt(1)->and(col('y')->lt(lit(2))))->toColumns()['g'], ['a']);
check('isNull', $df->filter(col('y')->isNull())->toColumns()['g'], ['b']);
check('sort desc, nulls last', $df->sort('y', true)->toColumns()['y'], [4.0, 2.5, 1.5, null]);
check('sort stable multi-key', $df->sort(['g', 'x'], [false, true])->toColumns()['x'], [3, 1, 2, null]);
check('select order', $df->select(['y', 'g'])->columns(), ['y', 'g']);
check('head', $df->head(2)->height(), 2);
check(
    'groupBy first appearance',
    $df->groupBy('g')->agg([
        col('x')->sum()->alias('sx'),
        col('y')->mean()->alias('my'),
        col('x')->count()->alias('n'),
        col('y')->max()->alias('hi'),
    ])->toRows(),
    [
        ['g' => 'a', 'sx' => 4, 'my' => 2.0, 'n' => 2, 'hi' => 2.5],
        ['g' => 'b', 'sx' => 2, 'my' => null, 'n' => 1, 'hi' => null],
        ['g' => 'c', 'sx' => 0, 'my' => 4.0, 'n' => 0, 'hi' => 4.0],
    ],
);
check('withColumns', $df->withColumns(['z' => col('x')->mul(10)->add(col('y'))])->toColumns()['z'], [12.5, null, 31.5, null]);

$rows = [['a' => 1, 'b' => 'x', 'c' => true], ['a' => null, 'b' => null, 'c' => false], ['a' => 2.5, 'b' => 'z', 'c' => null]];
check('fromRows widens int to float', DataFrame::fromRows($rows)->toRows(), [
    ['a' => 1.0, 'b' => 'x', 'c' => true],
    ['a' => null, 'b' => null, 'c' => false],
    ['a' => 2.5, 'b' => 'z', 'c' => null],
]);
check('fromColumns', DataFrame::fromColumns(['n' => [1, 2], 'e' => [null, null]])->toColumns(), ['n' => [1, 2], 'e' => [null, null]]);
check('empty frame', DataFrame::fromRows([])->height(), 0);

throws('unknown column', fn () => $df->select(['nope']), "column 'nope' not found");
throws('mixed types', fn () => DataFrame::fromColumns(['a' => [1, 'x']]), 'mixes incompatible types');
throws('missing file', fn () => DataFrame::readCsv('/nonexistent.csv'), 'nonexistent');
throws('non-expr agg', fn () => $df->groupBy('g')->agg(['x']), 'Phpolars\\Expr');
check('threads capped by INI', Phpolars\threads(), (int) (getenv('POLARS_MAX_THREADS') ?: ini_get('php_polars.max_threads')));
