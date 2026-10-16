<?php
require __DIR__ . '/harness.php';

use Phpolars\{DataFrame, Series};
use Phpolars\Exception\{ComputeError, InvalidOperationError};
use function Phpolars\{col, lit, when, element, concat_str, coalesce, sum_horizontal, int_range, struct_};

$df = new DataFrame(['g' => ['a', 'b', 'a', 'c', 'b'], 'x' => [1, 2, 3, null, 5], 'y' => [2.5, null, 1.5, 4.0, 0.5]]);
$one = fn ($e) => $df->select($e)->toSeries()->toList();

// Arithmetic and comparison
same('add', $one(col('x')->add(1)), [2, 3, 4, null, 6]);
same('truediv is float', $one(col('x')->truediv(2)), [0.5, 1.0, 1.5, null, 2.5]);
same('floordiv', $one(col('x')->floordiv(2)), [0, 1, 1, null, 2]);
same('mod', $one(col('x')->mod(2)), [1, 0, 1, null, 1]);
same('pow', $one(col('x')->pow(2)), [1, 4, 9, null, 25]);
same('neg', $one(col('x')->neg()), [-1, -2, -3, null, -5]);
same('column op column', $one(col('x')->mul(col('y'))), [2.5, null, 4.5, null, 2.5]);
same('string literal compare', $one(col('g')->eq('a')), [true, false, true, false, false]);
same('eqMissing', $one(col('x')->eqMissing(null)), [false, false, false, true, false]);
same('isBetween', $one(col('x')->isBetween(2, 3)), [false, true, true, null, false]);
same('isIn list', $one(col('g')->isIn(['a', 'c'])), [true, false, true, true, false]);
same('and or not', $one(col('x')->gt(1)->and(col('y')->lt(2))->or(col('g')->eq('c'))->not()), [true, null, false, false, false]);

// Nulls
same('isNull', $one(col('x')->isNull()), [false, false, false, true, false]);
same('fillNull', $one(col('x')->fillNull(0)), [1, 2, 3, 0, 5]);
same('fillNull mean', $one(col('x')->fillNull(strategy: 'mean')), [1, 2, 3, 2, 5]);
same('forwardFill', $one(col('y')->forwardFill()), [2.5, 2.5, 1.5, 4.0, 0.5]);
same('dropNulls', $one(col('x')->dropNulls()), [1, 2, 3, 5]);
same('nullCount', $one(col('x')->nullCount()), [1]);
same('interpolate', $one(col('y')->interpolate()), [2.5, 2.0, 1.5, 4.0, 0.5]);

// Aggregations
$agg = $df->select(
    col('x')->sum()->alias('sum'), col('x')->mean()->alias('mean'), col('x')->median()->alias('median'),
    col('x')->min()->alias('min'), col('x')->max()->alias('max'), col('x')->std()->alias('std'),
    col('x')->var()->alias('var'), col('x')->count()->alias('count'), col('x')->len()->alias('len'),
    col('g')->nUnique()->alias('nu'), col('x')->first()->alias('first'), col('x')->last()->alias('last'),
    col('x')->quantile(0.5, 'linear')->alias('q'), col('x')->product()->alias('prod'),
)->toDicts()[0];
same('aggregates', array_map(fn ($v) => is_float($v) ? round($v, 6) : $v, $agg), [
    'sum' => 11, 'mean' => 2.75, 'median' => 2.5, 'min' => 1, 'max' => 5, 'std' => 1.707825,
    'var' => 2.916667, 'count' => 4, 'len' => 5, 'nu' => 3, 'first' => 1, 'last' => 5, 'q' => 2.5, 'prod' => 30,
]);
same('argMax', $one(col('y')->argMax()), [3]);

// Windows and order-dependent
same('over', $one(col('x')->sum()->over('g')), [4, 7, 4, 0, 7]);
same('over orderBy', $one(col('x')->cumSum()->over('g', orderBy: 'y')), [4, 2, 3, null, 7]);
same('shift', $one(col('x')->shift()), [null, 1, 2, 3, null]);
same('shift fill', $one(col('x')->shift(-1, fillValue: 0)), [2, 3, null, 5, 0]);
same('diff', $one(col('x')->diff()), [null, 1, 1, null, null]);
same('cumSum', $one(col('x')->cumSum()), [1, 3, 6, null, 11]);
same('cumMax', $one(col('y')->cumMax()), [2.5, null, 2.5, 4.0, 4.0]);
same('rank', $one(col('y')->rank()), [3.0, null, 2.0, 4.0, 1.0]);
same('rank dense desc', $one(col('g')->rank('dense', descending: true)), [3, 2, 3, 1, 2]);
same('rollingSum', $one(col('y')->fillNull(0)->rollingSum(2)), [null, 2.5, 1.5, 5.5, 4.5]);

// Sorting and selection
same('sort', $one(col('x')->sort(descending: true, nullsLast: true)), [5, 3, 2, 1, null]);
same('sortBy', $one(col('g')->sortBy('y')), ['b', 'b', 'a', 'a', 'c']);
same('head', $one(col('x')->head(2)), [1, 2]);
same('slice', $one(col('x')->slice(1, 2)), [2, 3]);
same('topK', $one(col('y')->topK(2)), [4.0, 2.5]);
same('filter', $one(col('x')->filter(col('g')->eq('b'))), [2, 5]);
same('gather', $one(col('g')->gather([0, 3])), ['a', 'c']);
same('unique ordered', $one(col('g')->unique(maintainOrder: true)), ['a', 'b', 'c']);
same('isDuplicated', $one(col('g')->isDuplicated()), [true, true, true, false, true]);
same('replace map', $one(col('g')->replace(['a' => 'A', 'b' => 'B'])), ['A', 'B', 'A', 'c', 'B']);
same('replaceStrict default', $one(col('g')->replaceStrict(['a' => 1], default: 0)), [1, 0, 1, 0, 0]);
same('clip', $one(col('x')->clip(2, 4)), [2, 2, 3, null, 4]);
same('cast', $one(col('x')->cast('Float32')), [1.0, 2.0, 3.0, null, 5.0]);
throws('strict cast fails', fn () => $one(col('g')->cast('Int64')), Phpolars\PolarsError::class, 'conversion from');
same('non-strict cast nulls', $one(col('g')->cast('Int64', strict: false)), [null, null, null, null, null]);
same('round', $one(col('y')->round()), [2.0, null, 2.0, 4.0, 0.0]);
same('abs sqrt', $one(lit(-4)->abs()->sqrt()), [2.0]);
same('implode', $one(col('x')->implode()), [[1, 2, 3, null, 5]]);

// when/then/otherwise
same('when chain', $one(when(col('x')->lt(2))->then(lit('low'))->when(col('x')->lt(4))->then(lit('mid'))->otherwise(lit('high'))),
    ['low', 'mid', 'mid', 'high', 'high']);
same('when bare then', $one(when(col('x')->gt(2))->then(col('x'))), [null, null, 3, null, 5]);
same('when string names column', $one(when(col('x')->gt(2))->then('g')->otherwise(lit('-'))), ['-', '-', 'a', '-', 'b']);

// Function helpers and name namespace
same('concat_str', $one(concat_str([col('g'), col('x')], separator: '-')), ['a-1', 'b-2', 'a-3', null, 'b-5']);
same('coalesce', $one(coalesce(col('x'), lit(-1))), [1, 2, 3, -1, 5]);
same('sum_horizontal', $one(sum_horizontal('x', 'y')), [3.5, 2.0, 4.5, 4.0, 5.5]);
same('int_range', $df->select(int_range(3)->alias('i'))->toSeries()->toList(), [0, 1, 2]);
same('name suffix', $df->select(col('x', 'y')->sum()->name()->suffix('_sum'))->columns(), ['x_sum', 'y_sum']);
same('col list', $df->select(col(['g', 'x']))->columns(), ['g', 'x']);
same('exclude', $df->select(Phpolars\exclude('g'))->columns(), ['x', 'y']);

// str namespace
$s = new DataFrame(['s' => ['  Hello ', 'world', null, 'Foo-bar']]);
$str = fn ($e) => $s->select($e)->toSeries()->toList();
same('strip', $str(col('s')->str()->stripChars()), ['Hello', 'world', null, 'Foo-bar']);
same('upper', $str(col('s')->str()->toUppercase()), ['  HELLO ', 'WORLD', null, 'FOO-BAR']);
same('lenChars', $str(col('s')->str()->lenChars()), [8, 5, null, 7]);
same('contains regex', $str(col('s')->str()->contains('^F')), [false, false, null, true]);
same('contains literal', $str(col('s')->str()->contains('-', literal: true)), [false, false, null, true]);
same('startsWith', $str(col('s')->str()->startsWith('w')), [false, true, null, false]);
same('replace first', $str(col('s')->str()->replace('o', '0')), ['  Hell0 ', 'w0rld', null, 'F0o-bar']);
same('replaceAll', $str(col('s')->str()->replaceAll('o', '0')), ['  Hell0 ', 'w0rld', null, 'F00-bar']);
same('slice', $str(col('s')->str()->slice(0, 3)), ['  H', 'wor', null, 'Foo']);
same('split', $str(col('s')->str()->split('-')), [['  Hello '], ['world'], null, ['Foo', 'bar']]);
same('padStart', $str(col('s')->str()->stripChars()->str()->padStart(6, '*')), ['*Hello', '*world', null, 'Foo-bar']);
same('extract', $str(col('s')->str()->extract('(\w+)-(\w+)', 2)), [null, null, null, 'bar']);
same('countMatches', $str(col('s')->str()->countMatches('o')), [1, 1, null, 2]);
same('toInteger', (new DataFrame(['n' => ['1', '20', 'ff']]))->select(col('n')->str()->toInteger(base: 16))->toSeries()->toList(), [1, 32, 255]);
same('str join', $str(col('s')->str()->join('|')), ['  Hello |world|Foo-bar']);

// dt namespace and dates
$d = (new DataFrame(['t' => ['2024-01-15 10:30:00', '2024-02-29 23:59:59', null]]))
    ->withColumns(col('t')->str()->toDatetime('%Y-%m-%d %H:%M:%S'));
same('datetime dtype', $d->dtypes(), ['Datetime(us)']);
$dt = fn ($e) => $d->select($e)->toSeries()->toList();
same('year month day', $d->select(col('t')->dt()->year()->alias('y'), col('t')->dt()->month()->alias('m'), col('t')->dt()->day()->alias('d'))->row(1),
    [2024, 2, 29]);
same('weekday', $dt(col('t')->dt()->weekday()), [1, 4, null]);
same('hour', $dt(col('t')->dt()->hour()), [10, 23, null]);
same('strftime', $dt(col('t')->dt()->strftime('%d/%m/%Y')), ['15/01/2024', '29/02/2024', null]);
same('truncate month', $dt(col('t')->dt()->truncate('1mo')->dt()->day()), [1, 1, null]);
same('epoch s', $dt(col('t')->dt()->epoch('s')), [1705314600, 1709251199, null]);
$values = $d->getColumn('t')->toList();
same('datetime to PHP', [$values[0] instanceof DateTimeImmutable, $values[0]->format('Y-m-d H:i:s'), $values[2]], [true, '2024-01-15 10:30:00', null]);
same('date to PHP', $d->select(col('t')->dt()->date())->item(0, 't')->format('Y-m-d'), '2024-01-15');
$fromPhp = new DataFrame(['when' => [new DateTimeImmutable('2024-03-01 12:00:00'), null]]);
same('PHP DateTime in', [$fromPhp->dtypes(), $fromPhp->select(col('when')->dt()->hour())->item(0, 'when')], [['Datetime(us)'], 12]);
same('tz convert', $d->select(col('t')->dt()->replaceTimeZone('UTC')->dt()->convertTimeZone('America/New_York')->dt()->hour())->item(0, 't'), 5);
same('date literal filter', $d->filter(col('t')->gt(new DateTimeImmutable('2024-02-01')))->height(), 1);

// list and struct
$l = new DataFrame(['v' => [[1, 2, 3], [], null, [4, null]]]);
$lst = fn ($e) => $l->select($e)->toSeries()->toList();
same('list dtype', $l->dtypes(), ['List(Int64)']);
same('list len', $lst(col('v')->list()->len()), [3, 0, null, 2]);
same('list sum', $lst(col('v')->list()->sum()), [6, 0, null, 4]);
same('list get', $lst(col('v')->list()->get(0, nullOnOob: true)), [1, null, null, 4]);
same('list contains', $lst(col('v')->list()->contains(2)), [true, false, null, false]);
same('list eval', $lst(col('v')->list()->eval(element()->mul(10))), [[10, 20, 30], [], null, [40, null]]);
same('list join', (new DataFrame(['w' => [['a', 'b'], ['c']]]))->select(col('w')->list()->join('+'))->toSeries()->toList(), ['a+b', 'c']);
$st = new DataFrame(['p' => [['x' => 1, 'y' => 'a'], ['x' => 2, 'y' => null], null]]);
same('struct dtype', $st->dtypes(), ['Struct(x: Int64, y: String)']);
same('struct to PHP', $st->getColumn('p')->toList(), [['x' => 1, 'y' => 'a'], ['x' => 2, 'y' => null], null]);
same('struct field', $st->select(col('p')->struct()->field('x'))->toSeries()->toList(), [1, 2, null]);
same('unnest', $st->unnest('p')->columns(), ['x', 'y']);
same('struct_ pack', $df->select(struct_('g', 'x')->alias('s'))->head(1)->toSeries()->toList(), [['g' => 'a', 'x' => 1]]);

// Series
$ser = new Series('n', [3, 1, null, 2]);
same('series basics', [$ser->name(), $ser->len(), $ser->count(), $ser->nullCount(), $ser->dtype()], ['n', 4, 3, 1, 'Int64']);
same('series aggs', [$ser->sum(), $ser->min(), $ser->max(), $ser->mean()], [6, 1, 3, 2.0]);
same('series get', [$ser->get(0), $ser->get(-1)], [3, 2]);
throws('series get oob', fn () => $ser->get(10), Phpolars\Exception\OutOfBoundsError::class);
same('series sort', $ser->sort()->toList(), [null, 1, 2, 3]);
same('series arithmetic', $ser->mul(2)->add(new Series('m', [1, 1, 1, 1]))->toList(), [7, 3, null, 5]);
same('series compare', $ser->gt(1)->toList(), [true, false, null, true]);
same('series filter', $ser->filter($ser->gt(1))->toList(), [3, 2]);
same('series dtype arg', (new Series('f', [1, 2], 'Float64'))->toList(), [1.0, 2.0]);
same('series str', (new Series('s', ['ab', 'c']))->str()->lenChars()->toList(), [2, 1]);
same('series valueCounts', (new Series('s', ['a', 'b', 'a']))->valueCounts(sort: true)->toDicts(), [['s' => 'a', 'count' => 2], ['s' => 'b', 'count' => 1]]);
same('series toFrame', $ser->toFrame()->columns(), ['n']);
same('series in frame', (new DataFrame(['n' => $ser]))->height(), 4);
same('series unique', $ser->unique(maintainOrder: true)->toList(), [3, 1, null, 2]);
same('series cast', $ser->cast('String')->toList(), ['3', '1', null, '2']);
same('series isIn', $ser->isIn([1, 2])->toList(), [false, true, null, true]);
same('series item', (new Series('o', [42]))->item(), 42);
same('series equals', $ser->equals(new Series('n', [3, 1, null, 2])), true);

// Series forwards Expr methods it does not define
$fs = new Series('f', [1.0, 3.0, 2.0]);
same('series forwards to Expr', $fs->cumSum()->toList(), [1.0, 4.0, 6.0]);
same('forwarded reduction is scalar', is_float($fs->skew()), true);
same('forwarded keeps name', $fs->sinh()->name(), 'f');
throws('unknown Series method', fn () => $fs->nope(), Error::class, 'Series::nope()');
same('cat namespace on Series', (new Series('c', ['x', 'y', 'x'], 'Categorical'))->cat()->lenChars()->toList(), [1, 1, 1]);
same('Array dtype parses', (new Series('a', [[1, 2]], 'Array(Int64, 2)'))->dtype(), 'Array(Int64, 2)');
same('Struct dtype parses', (new Series('s', [['a' => 1]], 'Struct(a: Int64)'))->dtype(), 'Struct(a: Int64)');
