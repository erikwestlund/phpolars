<?php
// Regression tests for the v0.1.0 review fixes.
require __DIR__.'/harness.php';

use Phpolars\{DataFrame, Series};
use Phpolars\Exception\SchemaError;
use function Phpolars\col;

// Run first: an eager operation must not start Polars' pool before the
// php_polars.max_threads cap applies (it used to start one thread per core).
$big = new DataFrame(['a' => range(1, 50000), 'b' => range(1, 50000)]);
$big->vstack($big)->transpose();
same('thread cap holds after an eager operation', Phpolars\thread_pool_size(), (int) (getenv('POLARS_MAX_THREADS') ?: ini_get('php_polars.max_threads')));

$df = new DataFrame(['x' => [1, 2, 3, 4], 's' => ['ab', 'cd', 'ef', 'gh']]);

// Negative head/tail keep all but |n| rows, as in py-polars.
same('head(-1)', $df->head(-1)->getColumn('x')->toList(), [1, 2, 3]);
same('tail(-1)', $df->tail(-1)->getColumn('x')->toList(), [2, 3, 4]);
same('lazy head(-3)', $df->lazy()->head(-3)->collect()->getColumn('x')->toList(), [1]);
same('series head(-1)', (new Series('a', [1, 2, 3]))->head(-1)->toList(), [1, 2]);
same('series tail(-1)', (new Series('a', [1, 2, 3]))->tail(-1)->toList(), [2, 3]);
same('head(-10) on 4 rows', $df->head(-10)->height(), 0);

// Counts that used to wrap to huge unsigned values or panic now raise ValueError.
$bad = [
    'gatherEvery(0)' => fn () => $df->select(col('x')->gatherEvery(0)),
    'rollingSum(0)' => fn () => $df->select(col('x')->rollingSum(0)),
    'rollingSum(minPeriods: -1)' => fn () => $df->select(col('x')->rollingSum(2, minPeriods: -1)),
    'std(-1)' => fn () => $df->select(col('x')->std(-1)),
    'std(300)' => fn () => $df->select(col('x')->std(300)),
    'var(256)' => fn () => $df->var(256),
    'round(-1)' => fn () => $df->select(col('x')->round(-1)),
    'series round(-2)' => fn () => (new Series('a', [1.5]))->round(-2),
    'zfill(-1)' => fn () => $df->select(col('s')->str()->zfill(-1)),
    'splitn(0)' => fn () => $df->select(col('s')->str()->splitn('b', 0)),
    'extract(group -1)' => fn () => $df->select(col('s')->str()->extract('(a)', -1)),
    'expr head(-1)' => fn () => $df->select(col('x')->head(-1)),
    'topK(-1)' => fn () => $df->select(col('x')->topK(-1)),
    'expr sample(-1)' => fn () => $df->select(col('x')->sample(-1)),
    'groupBy head(-1)' => fn () => $df->groupBy('x')->head(-1),
    'withRowIndex(offset: -5)' => fn () => $df->withRowIndex('i', -5),
    'sample(-2)' => fn () => $df->sample(-2),
    'slice length -1' => fn () => $df->slice(1, -1),
    'series slice length -1' => fn () => (new Series('a', [1]))->slice(0, -1),
    'fillNull limit -1' => fn () => $df->select(col('x')->forwardFill(-1)),
];
foreach ($bad as $name => $f) {
    throws($name, $f, ValueError::class);
}

// Mixed kinds inside nested values raise SchemaError instead of becoming strings.
throws('list mixing strings and numbers', fn () => new DataFrame(['l' => [['x'], [1]]]), SchemaError::class, 'mixes');
throws('struct field mixing kinds', fn () => new DataFrame(['l' => [[['a' => 1]], [['a' => 'z']]]]), SchemaError::class, 'at [].a');
same('lists still widen int to float', (new DataFrame(['l' => [[1, 2.5], [3]]]))->dtypes(), ['List(Float64)']);
same('struct fields keep their own types', (new DataFrame(['p' => [['a' => 1, 'b' => 'x'], ['a' => 2.5, 'b' => null]]]))->dtypes(), ['Struct(a: Float64, b: String)']);
