<?php
require __DIR__ . '/harness.php';

use Phpolars\{DataFrame, Series};
use Phpolars\Exception\{ColumnNotFoundError, SchemaError};
use function Phpolars\{col, lit, len};

$df = new DataFrame(['g' => ['a', 'b', 'a', 'c'], 'x' => [1, 2, 3, null], 'y' => [2.5, null, 1.5, 4.0]]);

// Construction and shape
same('columns', $df->columns(), ['g', 'x', 'y']);
same('shape', $df->shape(), [4, 3]);
same('schema', $df->schema(), ['g' => 'String', 'x' => 'Int64', 'y' => 'Float64']);
same('dtypes', $df->dtypes(), ['String', 'Int64', 'Float64']);
same('from rows', (new DataFrame([['a' => 1, 'b' => 'x'], ['a' => 2, 'b' => null]]))->toDict(asSeries: false), ['a' => [1, 2], 'b' => ['x', null]]);
same('positional rows need schema', DataFrame::fromRows([[1, 'x'], [2, 'y']], schema: ['n' => 'Int32', 's' => 'String'])->schema(), ['n' => 'Int32', 's' => 'String']);
same('schema override', (new DataFrame(['a' => [1, 2]], schema: ['a' => 'Float64']))->toDict(asSeries: false), ['a' => [1.0, 2.0]]);
same('empty with schema', (new DataFrame([], ['a' => 'Int64']))->schema(), ['a' => 'Int64']);
same('int widens to float', (new DataFrame(['a' => [1, 2.5]]))->dtypes(), ['Float64']);
throws('mixed types', fn () => new DataFrame(['a' => [1, 'x']]), SchemaError::class, 'mixes incompatible types');
throws('ragged columns', fn () => new DataFrame(['a' => [1], 'b' => [1, 2]]), Phpolars\PolarsError::class);

// Output
same('toDicts', $df->toDicts()[1], ['g' => 'b', 'x' => 2, 'y' => null]);
same('rows', $df->rows()[0], ['a', 1, 2.5]);
same('row named', $df->row(-1, named: true), ['g' => 'c', 'x' => null, 'y' => 4.0]);
same('item', $df->item(2, 'x'), 3);
same('item 1x1', $df->select(col('x')->sum())->item(), 6);
same('toDict series', $df->toDict()['g'] instanceof Series, true);
same('getColumn', $df->getColumn('y')->toList(), [2.5, null, 1.5, 4.0]);
throws('getColumn missing', fn () => $df->getColumn('nope'), ColumnNotFoundError::class, 'nope');

// Verbs
same('select exprs', $df->select('g', col('x')->mul(10)->alias('x10'), ['half' => col('y')->truediv(2)])->columns(), ['g', 'x10', 'half']);
same('filter null is false', $df->filter(col('x')->ge(2))->getColumn('g')->toList(), ['b', 'a']);
same('filter several', $df->filter(col('x')->gt(0), col('g')->eq('a'))->height(), 2);
same('withColumns', $df->withColumns(col('x')->add(col('y'))->alias('s'))->getColumn('s')->toList(), [3.5, null, 4.5, null]);
same('sort desc nulls last', $df->sort('y', descending: true, nullsLast: true)->getColumn('y')->toList(), [4.0, 2.5, 1.5, null]);
same('sort multi', $df->sort(['g', 'x'], descending: [false, true])->getColumn('x')->toList(), [3, 1, 2, null]);
same('head', $df->head(2)->height(), 2);
same('tail', $df->tail(1)->getColumn('g')->toList(), ['c']);
same('slice', $df->slice(1, 2)->getColumn('g')->toList(), ['b', 'a']);
same('drop', $df->drop('x', 'y')->columns(), ['g']);
throws('drop missing', fn () => $df->drop('nope'), ColumnNotFoundError::class);
same('rename', $df->rename(['g' => 'group'])->columns(), ['group', 'x', 'y']);
same('unique', $df->select('g')->unique(maintainOrder: true)->getColumn('g')->toList(), ['a', 'b', 'c']);
same('unique keep last', $df->unique(subset: 'g', keep: 'last', maintainOrder: true)->getColumn('x')->toList(), [2, 3, null]);
same('fillNull value', $df->select('x')->fillNull(0)->getColumn('x')->toList(), [1, 2, 3, 0]);
same('fillNull strategy', $df->select('y')->fillNull(strategy: 'forward')->getColumn('y')->toList(), [2.5, 2.5, 1.5, 4.0]);
same('dropNulls', $df->dropNulls()->height(), 2);
same('dropNulls subset', $df->dropNulls('x')->height(), 3);
same('nullCount', $df->nullCount()->toDicts(), [['g' => 0, 'x' => 1, 'y' => 1]]);
same('withRowIndex', $df->withRowIndex()->getColumn('index')->toList(), [0, 1, 2, 3]);
same('cast map', $df->cast(['x' => 'Float64'])->dtypes(), ['String', 'Float64', 'Float64']);
same('reverse', $df->reverse()->getColumn('g')->toList(), ['c', 'a', 'b', 'a']);
same('frame sum', $df->select('x', 'y')->sum()->toDicts(), [['x' => 6, 'y' => 8.0]]);
same('frame mean', $df->select('x')->mean()->item(), 2.0);

// Group by
$agg = $df->groupBy('g', maintainOrder: true)->agg(col('x')->sum()->alias('sx'), ['n' => len(), 'my' => col('y')->mean()]);
same('groupBy agg', $agg->toDicts(), [
    ['g' => 'a', 'sx' => 4, 'n' => 2, 'my' => 2.0],
    ['g' => 'b', 'sx' => 2, 'n' => 1, 'my' => null],
    ['g' => 'c', 'sx' => 0, 'n' => 1, 'my' => 4.0],
]);
same('groupBy len', $df->groupBy('g', maintainOrder: true)->len()->toDicts()[0], ['g' => 'a', 'len' => 2]);
same('groupBy sum', $df->groupBy('g', maintainOrder: true)->sum()->columns(), ['g', 'x', 'y']);
same('groupBy all lists', $df->groupBy('g', maintainOrder: true)->all()->getColumn('x')->toList()[0], [1, 3]);
same('groupBy multi', $df->groupBy(['g', col('x')->isNull()->alias('xn')], maintainOrder: true)->len()->height(), 3);

// Join
$right = new DataFrame(['g' => ['a', 'c', 'd'], 'z' => [10, 30, 40]]);
same('inner join', $df->join($right, on: 'g')->sort('x')->getColumn('z')->toList(), [30, 10, 10]);
same('left join', $df->join($right, on: 'g', how: 'left', maintainOrder: 'left')->getColumn('z')->toList(), [10, null, 10, 30]);
same('full join', $df->join($right, on: 'g', how: 'full')->columns(), ['g', 'x', 'y', 'g_right', 'z']);
same('full join coalesce', $df->join($right, on: 'g', how: 'full', coalesce: true)->height(), 5);
same('semi join', $df->join($right, on: 'g', how: 'semi')->height(), 3);
same('anti join', $df->join($right, on: 'g', how: 'anti')->getColumn('g')->toList(), ['b']);
same('cross join', $df->join($right, how: 'cross')->height(), 12);
same('leftOn rightOn', $df->join($right->rename(['g' => 'key']), leftOn: 'g', rightOn: 'key')->height(), 3);
same('suffix', $df->join($right->withColumns(col('z')->alias('x')), on: 'g', suffix: '_r')->columns(), ['g', 'x', 'y', 'z', 'x_r']);

// Concat, stack, unpivot, explode
same('concat vertical', Phpolars\concat([$df, $df])->height(), 8);
same('concat horizontal', Phpolars\concat([$df->select('g'), $right->select('z')->head(4)], how: 'horizontal')->columns(), ['g', 'z']);
same('concat diagonal', Phpolars\concat([$df->select('g'), $right], how: 'diagonal')->columns(), ['g', 'z']);
same('vstack', $df->vstack($df)->height(), 8);
same('hstack', $df->hstack([new Series('k', [1, 2, 3, 4])])->columns(), ['g', 'x', 'y', 'k']);
$long = $df->select('g', 'x', 'y')->unpivot(index: 'g');
same('unpivot', [$long->columns(), $long->height()], [['g', 'variable', 'value'], 8]);
same('explode', (new DataFrame(['k' => ['a', 'b'], 'v' => [[1, 2], [3]]]))->explode('v')->getColumn('v')->toList(), [1, 2, 3]);

// Lazy
$lf = $df->lazy()->filter(col('x')->gt(1))->select('g', 'x');
same('lazy collect', $lf->collect()->toDicts(), [['g' => 'b', 'x' => 2], ['g' => 'a', 'x' => 3]]);
same('lazy schema', $lf->collectSchema(), ['g' => 'String', 'x' => 'Int64']);
same('explain mentions filter', str_contains($lf->explain(), 'FILTER') || str_contains($lf->explain(), 'SELECTION'), true);
same('lazy groupBy', $df->lazy()->groupBy('g', maintainOrder: true)->agg(col('x')->max())->collect()->getColumn('x')->toList(), [3, 2, null]);

// Equality
same('equals', $df->equals($df->clone()), true);
same('not equals', $df->equals($df->head(2)), false);
same('threads capped by INI', Phpolars\thread_pool_size(), (int) (getenv('POLARS_MAX_THREADS') ?: ini_get('php_polars.max_threads')));

// Reshaping and summaries
$w = new DataFrame(['id' => [1, 1, 2, 2], 'k' => ['a', 'b', 'a', 'b'], 'v' => [10, 20, 30, null]]);
same('pivot', $w->pivot('k', index: 'id', values: 'v')->toDicts(), [['id' => 1, 'a' => 10, 'b' => 20], ['id' => 2, 'a' => 30, 'b' => null]]);
same('pivot sum', $w->pivot('k', index: 'id', values: 'v', aggregateFunction: 'sum')->getColumn('a')->toList(), [10, 30]);
same('pivot expr agg', $w->pivot('k', index: 'id', values: 'v', aggregateFunction: Phpolars\element()->max())->getColumn('b')->toList(), [20, null]);
same('transpose', $w->select('id', 'v')->transpose(includeHeader: true)->row(1), ['v', 10, 20, 30, null]);
same('transpose names', $w->select('v')->head(2)->transpose(columnNames: ['p', 'q'])->columns(), ['p', 'q']);
same('sample n', $w->sample(3, seed: 7)->height(), 3);
same('sample fraction', $w->sample(fraction: 0.5, seed: 7)->height(), 2);
same('sample seeded', $w->sample(2, seed: 1)->toDicts(), $w->sample(2, seed: 1)->toDicts());
$d = $w->select('id', 'v')->describe();
same('describe stats', $d->getColumn('statistic')->toList(), ['count', 'null_count', 'mean', 'std', 'min', '25%', '50%', '75%', 'max']);
same('describe numeric', $d->getColumn('v')->toList(), [3.0, 1.0, 20.0, 10.0, 10.0, 20.0, 20.0, 30.0, 30.0]);
same('describe keeps numeric columns float', $w->describe()->dtypes(), ['String', 'Float64', 'String', 'Float64']);
preg_match('/^polars = \{ version = "=([^"]+)"/m', file_get_contents(__DIR__.'/../Cargo.toml'), $pin);
same('polars_version matches the Cargo pin', Phpolars\polars_version(), $pin[1] ?? null);

// Partitions, windows on LazyFrame, and argument checks
$parts = $w->partitionBy('k');
same('partitionBy list', array_map(fn ($p) => $p->height(), $parts), [2, 2]);
same('partitionBy asDict keys', array_keys($w->partitionBy('k', asDict: true)), ['a', 'b']);
same('partitionBy drops key', $w->partitionBy('k', includeKey: false)[0]->columns(), ['id', 'v']);
throws('partitionBy asDict needs one key', fn () => $w->partitionBy(['id', 'k'], asDict: true), ValueError::class, 'one key column');
$ts = new DataFrame(['t' => [new DateTimeImmutable('2024-01-01 00:00'), new DateTimeImmutable('2024-01-01 12:00'), new DateTimeImmutable('2024-01-02 06:00')], 'v' => [1, 2, 3]]);
same('lazy groupByDynamic', $ts->lazy()->groupByDynamic('t', every: '1d')->agg(col('v')->sum())->collect()->getColumn('v')->toList(), [3, 3]);
same('groupByDynamic sum() skips the index', $ts->groupByDynamic('t', every: '1d')->sum()->columns(), ['t', 'v']);
same('lazy rolling', $ts->lazy()->rolling('t', period: '1d')->agg(col('v')->sum())->collect()->getColumn('v')->toList(), [1, 3, 5]);
same('lazy joinAsof', $ts->lazy()->joinAsof($ts->select('t', col('v')->alias('w'))->tail(1), on: 't', strategy: 'forward')->collect()->getColumn('w')->toList(), [3, 3, 3]);
same('lazy describe', $ts->lazy()->describe()->getColumn('statistic')->toList()[0], 'count');
throws('bad duration', fn () => $ts->groupByDynamic('t', every: 'soon'), Phpolars\PolarsError::class);
throws('asof needs keys', fn () => $ts->joinAsof($ts), ValueError::class, 'needs on');
throws('ewm needs one parameter', fn () => col('v')->ewmMean(com: 1, span: 2), ValueError::class, 'exactly one');
throws('gatherEvery zero', fn () => $ts->gatherEvery(0), ValueError::class, 'at least 1');
same('exclude', $ts->select(Phpolars\all()->exclude('t'))->columns(), ['v']);

// SQL
$ctx = new Phpolars\SQLContext(['w' => $w]);
$ctx->register('t', $ts->lazy());
same('SQLContext tables', $ctx->tables(), ['t', 'w']);
same('execute is lazy by default', $ctx->execute('SELECT * FROM w') instanceof Phpolars\LazyFrame, true);
same('execute eager', $ctx->execute('SELECT SUM(v) AS s FROM t', eager: true)->item(), 6);
$ctx->unregister('t');
same('unregister', $ctx->tables(), ['w']);
throws('unknown table', fn () => $ctx->execute('SELECT * FROM t', eager: true), Phpolars\PolarsError::class);
same('LazyFrame sql', $w->lazy()->sql('SELECT COUNT(*) AS n FROM self')->collect()->item(), 4);
same('eager SQLContext', (new Phpolars\SQLContext(['w' => $w], eager: true))->execute('SELECT id FROM w LIMIT 1') instanceof DataFrame, true);

// glimpse, extend, rechunk
$g = $w->glimpse();
same('glimpse header', explode("\n", $g)[0], 'Rows: 4');
same('glimpse line', explode("\n", $g)[3], '$ k  <String> "a", "b", "a", "b"');
same('extend', $w->extend($w)->height(), 8);
same('rechunk keeps data', $w->extend($w)->rechunk()->equals($w->vstack($w)), true);
throws('getColumnIndex unknown', fn () => $w->getColumnIndex('nope'), Phpolars\Exception\ColumnNotFoundError::class);

// collect_all, from_*, LazyFrame schema, Series extras
[$a, $b] = Phpolars\collect_all([$w->lazy()->head(1), $w->lazy()->select('k')]);
same('collect_all', [$a->height(), $b->columns()], [1, ['k']]);
same('from_dict', Phpolars\from_dict(['a' => [1]])->columns(), ['a']);
same('from_dicts', Phpolars\from_dicts([['a' => 1], ['a' => 2]])->height(), 2);
same('LazyFrame schema', $w->lazy()->select('id')->schema(), ['id' => 'Int64']);
same('LazyFrame width', $w->lazy()->width(), 3);
$sv = new Series('s', [1, 2, 3]);
same('isSorted', [$sv->isSorted(), $sv->isSorted(descending: true)], [true, false]);
same('argTrue', (new Series('b', [false, true, true]))->argTrue()->toList(), [1, 2]);
same('zipWith', $sv->zipWith(new Series('m', [true, false, true]), new Series('o', [9, 9, 9]))->toList(), [1, 9, 3]);
same('Series clear', $sv->clear(2)->toList(), [null, null]);
same('Series extend', $sv->extend($sv)->len(), 6);

// rowsByKey, profile, Series helpers
$byKey = $w->rowsByKey('k', named: true);
same('rowsByKey groups', array_map('count', $byKey), ['a' => 2, 'b' => 2]);
same('rowsByKey unique', $w->rowsByKey('k', unique: true)['a'], [2, 30]);
[$res, $timings] = $w->lazy()->filter(col('id')->gt(1))->profile();
same('profile result', $res->height(), 2);
same('profile timings', in_array('node', $timings->columns(), true), true);
same('newFromIndex', $sv->newFromIndex(-1, 2)->toList(), [3, 3]);
same('shape', $sv->shape(), [3]);
same('UInt64 beyond int64 stays per value', (new Series('u', [1, 2], 'Int64'))->cast('UInt64')->extend((new Series('u', [-1], 'Int64'))->reinterpret(signed: false))->toList(), [1, 2, 18446744073709551615.0]);
