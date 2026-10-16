<?php
// Differential tests: run the cases from scripts/pyfixtures.py through phpolars and compare
// with the py-polars output in tests/fixtures/py/expected.json. Case names match the Python file.
// Every case runs; mismatches are listed at the end and the script exits non-zero.
require __DIR__ . '/harness.php';

use Phpolars\{DataFrame, Series};
use Phpolars\cs;
use function Phpolars\{col, lit, when, len, element, concat, concat_str, coalesce, concat_list, struct_};
use function Phpolars\{sum_horizontal, mean_horizontal, min_horizontal, max_horizontal};
use function Phpolars\{date_range, date_ranges, datetime_range, int_ranges, date, datetime, duration, from_epoch, arg_where, corr, cov, nth, repeat, linear_space, select, std, quantile, implode, approx_n_unique};

$fixture = json_decode(file_get_contents(__DIR__ . '/fixtures/py/expected.json'), true, flags: JSON_THROW_ON_ERROR);

// ---------------------------------------------------------------------------
// Inputs, built the same way as in scripts/pyfixtures.py.

$base = (new DataFrame([
    'id' => [1, 2, 3, 4, 5, 6],
    'g' => ['a', 'b', 'a', 'c', null, 'b'],
    'x' => [3, null, 1, 7, 2, 5],
    'y' => [1.5, 2.25, null, -0.5, 10.0, 3.0],
    'k' => [2, 1, 2, null, 3, 1],
    'b' => [true, false, null, true, false, true],
    's' => ['apple pie', 'Bänana', null, 'cherry-42', '  date  ', 'elder_7 berry'],
    'ds' => ['2024-01-15', '2024-02-29', null, '2023-12-31', '2024-07-04', '2025-03-09'],
    'ts' => ['2024-01-15 10:30:00', '2024-02-29 23:59:59', '2023-06-01 00:00:01', null, '2024-07-04 12:00:00', '2025-03-09 06:15:30'],
    'l' => [[1, 2, 3], [], null, [4, null], [5], [6, 7]],
]))
    ->withColumns(
        col('ds')->str()->toDate('%Y-%m-%d')->alias('d'),
        col('ts')->str()->toDatetime('%Y-%m-%d %H:%M:%S')->alias('t'),
    )
    ->select('id', 'g', 'x', 'y', 'k', 'b', 's', 'd', 't', 'l');

$right = new DataFrame(['g' => ['a', 'c', 'd', null], 'z' => [10, 30, 40, 99], 'x' => [100, 300, 400, 999]]);
$long = new DataFrame(['id' => [1, 1, 2, 2, 3, 1], 'k' => ['p', 'q', 'p', 'q', 'p', 'p'], 'v' => [10, 20, 30, null, 50, 5]]);
$nums = new DataFrame(['n' => ['1', '20', '-3', 'x', null, '007']]);
$rnd = new DataFrame(['f' => [0.5, 1.5, 2.5, -0.5, -1.5, 1.005, 2.675, null]]);
$dt = fn (string $s) => new DateTimeImmutable($s);
$words = new DataFrame(['w' => ['the quick fox', 'Lazy DOG', null, 'a.b*c', 'café', '{"a": 1, "b": [1, 2]}']]);
$days = new DataFrame(['d' => [$dt('2024-01-05'), $dt('2024-02-29'), $dt('1999-12-31'), null]])->cast(['d' => 'Date']);
$arrs = new DataFrame(['a' => [[1, 5, 2], [4, null, 4], null]], schema: ['a' => 'Array(Int64, 3)']);
$cats = new DataFrame(['c' => ['apple', 'banana', 'apple', null]], schema: ['c' => 'Categorical']);
$wide = new DataFrame([
    'id' => [1], 'i8' => [1], 'u32' => [1], 'f32' => [1.0], 'price_f64' => [1.0], 'name' => ['a'], 'flag' => [true],
    'day' => [$dt('2024-01-01')], 'ts_us' => [$dt('2024-01-01')], 'ts_ms_utc' => [$dt('2024-01-01')],
    'dur' => [86_400_000_000], 'tm' => ['01:02:00'], 'cat' => ['x'], 'lst' => [[1, 2]], 'lst_s' => [['a']],
    'arr' => [[1, 2]], 'st' => [['a' => 1]], 'bin' => ['x'], 'dec' => [1],
], schema: [
    'i8' => 'Int8', 'u32' => 'UInt32', 'f32' => 'Float32', 'day' => 'Date', 'ts_ms_utc' => 'Datetime(ms, UTC)', 'dur' => 'Duration(us)',
    'cat' => 'Categorical', 'arr' => 'Array(Int64, 2)', 'bin' => 'Binary', 'dec' => 'Decimal(10, 2)',
])->withColumns(col('tm')->str()->toTime('%H:%M:%S'));
$tsf = new DataFrame([
    't' => array_map($dt, ['2024-01-01 00:00', '2024-01-01 01:00', '2024-01-01 03:00', '2024-01-02 00:00', '2024-01-02 05:00', '2024-01-04 00:00']),
    'g' => ['a', 'b', 'a', 'a', 'b', 'a'],
    'v' => [1, 2, 3, null, 5, 6],
    'f' => [1.0, NAN, 2.5, 4.0, null, 8.0],
]);
$quotes = new DataFrame([
    't' => array_map($dt, ['2024-01-01 00:30', '2024-01-01 02:00', '2024-01-02 04:00']),
    'g' => ['a', 'a', 'b'],
    'px' => [10.0, 11.0, 12.0],
]);

// ---------------------------------------------------------------------------
// Cases

$S = fn (...$e) => $base->select(...$e);
$G = fn ($keys) => $base->groupBy($keys, maintainOrder: true);
$L = $base->select('id', 'g', 'x');
$A = new DataFrame(['a' => [1, 2], 'b' => ['x', 'y']]);
$B = new DataFrame(['a' => [3], 'b' => ['z']]);
$C = new DataFrame(['b' => ['w'], 'c' => [1.5]]);
$C2 = new DataFrame(['b2' => ['v', 'w'], 'c' => [1.5, null]]);
$s = fn () => col('s')->str();
$t = fn () => col('t')->dt();
$d = fn () => col('d')->dt();
$li = fn () => col('l')->list();
$xs = $base->getColumn('x');
$ys = $base->getColumn('y');
$gs = $base->getColumn('g');
$bs = $base->getColumn('b');

$cases = [
    // select / withColumns: arithmetic and comparison
    'select_arith' => fn () => $S(
        'id',
        col('x')->add(1)->alias('add'),
        col('x')->sub(col('id'))->alias('sub'),
        col('x')->mul(col('y'))->alias('mul'),
        col('x')->truediv(2)->alias('truediv'),
        col('x')->floordiv(2)->alias('floordiv'),
        col('x')->mod(3)->alias('mod'),
        col('x')->pow(2)->alias('pow'),
        col('y')->neg()->alias('neg'),
    ),
    'select_float_specials' => fn () => $S(
        col('y')->truediv(0)->alias('div0'),
        col('x')->truediv(0)->alias('idiv0'),
        col('x')->floordiv(0)->alias('ifloordiv0'),
        col('y')->mul(0)->truediv(0)->alias('nan'),
    ),
    'select_int_div_neg' => fn () => (new DataFrame(['a' => [7, -7, 7, -7], 'b' => [2, 2, -2, -2]]))->select(
        col('a')->floordiv(col('b'))->alias('fd'), col('a')->mod(col('b'))->alias('m'), col('a')->truediv(col('b'))->alias('td'),
    ),
    'select_literals' => fn () => $base->head(2)->select(
        lit(1)->alias('i'), lit(2.5)->alias('f'), lit('k')->alias('s'), lit(true)->alias('t'), lit(null)->alias('n'),
    ),
    'select_literal_broadcast' => fn () => $S('id', lit(0)->alias('z'), col('x')->add(lit(0.5))->alias('xf')),
    'int32_plus_literal' => fn () => $S(
        col('x')->cast('Int32')->add(1)->alias('a'),
        col('x')->cast('Int32')->add(col('id'))->alias('b'),
        col('x')->cast('UInt8')->mul(2)->alias('c'),
        col('y')->cast('Float32')->mul(2)->alias('d'),
    ),
    'with_columns_compare' => fn () => $base->select('id', 'x', 'y', 'g')->withColumns(
        col('x')->gt(2)->alias('gt'),
        col('x')->ge(3)->alias('ge'),
        col('y')->lt(2)->alias('lt'),
        col('y')->le(2.25)->alias('le'),
        col('g')->eq('a')->alias('eq'),
        col('g')->ne('a')->alias('ne'),
        col('x')->eqMissing(null)->alias('eqm'),
        col('g')->neMissing('a')->alias('nem'),
        col('x')->gt(col('y'))->alias('cross'),
    ),
    'with_columns_overwrite' => fn () => $base->select('id', 'x', 'g')->withColumns(col('x')->mul(10)->alias('x'), col('g')->alias('g2')),
    'bool_logic' => fn () => $S(
        'b',
        col('b')->and(col('x')->gt(2))->alias('and'),
        col('b')->or(col('x')->gt(2))->alias('or'),
        col('b')->not()->alias('not'),
        col('b')->xor(col('x')->gt(2))->alias('xor'),
    ),
    'horizontal_fns' => fn () => $S(
        sum_horizontal('x', 'y')->alias('sum'),
        mean_horizontal('x', 'y')->alias('mean'),
        min_horizontal('x', 'y')->alias('min'),
        max_horizontal('x', 'k')->alias('max'),
        coalesce('x', 'k', lit(-1))->alias('coal'),
        concat_str(['g', 'x'], separator: '-')->alias('cs'),
        concat_str(['g', 'x'], separator: '-', ignoreNulls: true)->alias('csi'),
    ),

    // filter
    'filter_gt' => fn () => $base->select('id', 'x')->filter(col('x')->gt(2)),
    'filter_multi' => fn () => $base->select('id', 'g', 'x')->filter(col('x')->gt(1), col('g')->eq('b')),
    'filter_or_null' => fn () => $base->select('id', 'g', 'y')->filter(col('g')->isNull()->or(col('y')->lt(0))),
    'filter_bool_col' => fn () => $base->select('id', 'b')->filter(col('b')),
    // PHP has no date-only value; a DateTimeImmutable literal cast to Date is the equivalent of py's date literal.
    'filter_date' => fn () => $base->select('id', 'd')->filter(col('d')->gt(lit($dt('2024-01-31'))->cast('Date'))),
    'filter_datetime' => fn () => $base->select('id', 't')->filter(col('t')->ge($dt('2024-02-01'))),

    // sort
    'sort_single' => fn () => $base->select('id', 'x')->sort('x'),
    'sort_desc_nulls_last' => fn () => $base->select('id', 'y')->sort('y', descending: true, nullsLast: true),
    'sort_multi' => fn () => $base->select('id', 'g', 'x')->sort(['g', 'x'], descending: [false, true]),
    'sort_multi_nulls_last' => fn () => $base->select('id', 'g', 'k')->sort(['g', 'k'], nullsLast: true),
    'sort_multi_mixed_nulls_last' => fn () => $base->select('id', 'g', 'k')->sort(['g', 'k'], descending: [true, false], nullsLast: [true, false]),
    'sort_expr' => fn () => $base->select('id', 's')->sort(col('s')->str()->lenChars(), descending: true),
    'sort_date_desc' => fn () => $base->select('id', 'd')->sort('d', descending: true),
    'sort_string' => fn () => $base->select('id', 's')->sort('s'),
    'sort_bool' => fn () => $base->select('id', 'b')->sort(['b', 'id']),

    // groupBy
    'gb_sum_mean' => fn () => $G('g')->agg(col('x')->sum()->alias('sx'), col('y')->sum()->alias('sy'), col('x')->mean()->alias('mx'), col('y')->mean()->alias('my')),
    'gb_median_min_max' => fn () => $G('g')->agg(
        col('x')->median()->alias('med'), col('x')->min()->alias('min'), col('x')->max()->alias('max'),
        col('s')->min()->alias('smin'), col('d')->max()->alias('dmax'), col('t')->min()->alias('tmin'),
    ),
    'gb_std_var' => fn () => $G('g')->agg(col('y')->std()->alias('std'), col('y')->var()->alias('var'), col('x')->std(ddof: 0)->alias('std0')),
    'gb_count_len' => fn () => $G('g')->agg(col('x')->count()->alias('count'), len()->alias('len'), col('x')->nullCount()->alias('nulls')),
    'gb_n_unique_first_last' => fn () => $G('b')->agg(col('g')->nUnique()->alias('nu'), col('x')->first()->alias('first'), col('x')->last()->alias('last')),
    'gb_quantile' => fn () => $G('b')->agg(
        col('id')->quantile(0.4, 'nearest')->alias('nearest'),
        col('id')->quantile(0.4, 'linear')->alias('linear'),
        col('id')->quantile(0.4, 'lower')->alias('lower'),
        col('id')->quantile(0.4, 'higher')->alias('higher'),
        col('id')->quantile(0.4, 'midpoint')->alias('midpoint'),
    ),
    'gb_multi_key' => fn () => $G(['g', 'b'])->agg(col('id')->sum()->alias('ids')),
    'gb_expr_key' => fn () => $G(col('x')->gt(2)->alias('big'))->agg(col('id')->alias('ids'), col('y')->max()),
    'gb_implicit_list' => fn () => $G('g')->agg('x', col('s')->sort()->first()->alias('s1')),
    'gb_filter_sortby' => fn () => $G('g')->agg(
        col('x')->filter(col('y')->gt(1))->sum()->alias('fs'),
        col('id')->sortBy('y', descending: true)->first()->alias('top'),
    ),
    'gb_len_method' => fn () => $G('g')->len(),
    'gb_sum_method' => fn () => $base->select('g', 'x', 'y')->groupBy('g', maintainOrder: true)->sum(),
    'gb_mean_method' => fn () => $base->select('b', 'x', 'y')->groupBy('b', maintainOrder: true)->mean(),
    'gb_head' => fn () => $base->select('g', 'id')->groupBy('g', maintainOrder: true)->head(1),

    // joins
    'join_inner' => fn () => $L->join($right, on: 'g', suffix: '_r')->sort('id'),
    'join_left' => fn () => $L->join($right->select('g', 'z'), on: 'g', how: 'left', maintainOrder: 'left'),
    'join_full' => fn () => $L->join($right->select('g', 'z'), on: 'g', how: 'full')->sort(['id', 'z']),
    'join_full_coalesce' => fn () => $L->join($right->select('g', 'z'), on: 'g', how: 'full', coalesce: true)->sort(['id', 'z']),
    'join_semi' => fn () => $L->join($right, on: 'g', how: 'semi', maintainOrder: 'left'),
    'join_anti' => fn () => $L->join($right, on: 'g', how: 'anti', maintainOrder: 'left'),
    'join_nulls_equal' => fn () => $L->join($right, on: 'g', how: 'left', nullsEqual: true, maintainOrder: 'left'),
    'join_cross' => fn () => $L->head(2)->join($right->select('z')->head(3), how: 'cross', maintainOrder: 'left_right'),
    'join_suffix' => fn () => $L->join($right, on: 'g', suffix: '_r', maintainOrder: 'left'),
    'join_left_on_right_on' => fn () => $L->join($right->rename(['g' => 'key']), leftOn: 'g', rightOn: 'key', how: 'left', maintainOrder: 'left'),
    'join_multi_key' => fn () => $base->select('id', 'g', 'b')->join(
        new DataFrame(['g' => ['a', 'b', 'c'], 'b' => [true, true, true], 'w' => [1.5, 2.5, 3.5]]), on: ['g', 'b'], how: 'left', maintainOrder: 'left',
    ),

    // unique
    'unique_subset_first' => fn () => $base->select('id', 'g')->unique(subset: 'g', keep: 'first', maintainOrder: true),
    'unique_keep_last' => fn () => $base->select('id', 'g')->unique(subset: 'g', keep: 'last', maintainOrder: true),
    'unique_keep_none' => fn () => $base->select('id', 'g')->unique(subset: 'g', keep: 'none', maintainOrder: true),
    'unique_rows' => fn () => $long->select('id', 'k')->unique(maintainOrder: true),

    // nulls
    'fill_null_value_frame' => fn () => $base->select('g', 'x', 'y', 'b')->fillNull(0),
    'fill_null_string' => fn () => $S(col('g')->fillNull('?'), col('s')->fillNull(lit('none'))),
    'fill_null_strategies' => fn () => $S(
        col('x')->fillNull(strategy: 'forward')->alias('fwd'),
        col('y')->fillNull(strategy: 'backward')->alias('bwd'),
        col('x')->fillNull(strategy: 'min')->alias('min'),
        col('x')->fillNull(strategy: 'max')->alias('max'),
        col('x')->fillNull(strategy: 'mean')->alias('mean'),
        col('y')->fillNull(strategy: 'mean')->alias('ymean'),
        col('x')->fillNull(strategy: 'zero')->alias('zero'),
        col('x')->fillNull(strategy: 'one')->alias('one'),
    ),
    'fill_null_limit' => fn () => (new DataFrame(['a' => [1, null, null, null, 5, null]]))->select(
        col('a')->fillNull(strategy: 'forward', limit: 1)->alias('f1'),
        col('a')->fillNull(strategy: 'backward', limit: 2)->alias('b2'),
    ),
    'fill_null_frame_forward' => fn () => $base->select('g', 'x', 'd')->fillNull(strategy: 'forward'),
    'drop_nulls_all' => fn () => $base->select('id', 'g', 'x', 'y')->dropNulls(),
    'drop_nulls_subset' => fn () => $base->select('id', 'g', 'x', 'y')->dropNulls(['x', 'g']),
    'null_count' => fn () => $base->nullCount(),
    'interpolate' => fn () => (new DataFrame(['a' => [1.0, null, null, 4.0, null], 'i' => [1, null, 3, null, 5]]))->select(
        col('a')->interpolate(), col('i')->interpolate(),
    ),

    // reshape
    'unpivot' => fn () => $base->select('id', 'x', 'k')->unpivot(index: 'id'),
    'unpivot_mixed_names' => fn () => $base->select('id', 'x', 'y')->unpivot(on: ['x', 'y'], index: 'id', variableName: 'var', valueName: 'val'),
    'pivot_first' => fn () => $long->pivot('k', index: 'id', values: 'v', aggregateFunction: 'first'),
    'pivot_sum' => fn () => $long->pivot('k', index: 'id', values: 'v', aggregateFunction: 'sum'),
    'pivot_mean' => fn () => $long->pivot('k', index: 'id', values: 'v', aggregateFunction: 'mean'),
    'pivot_len' => fn () => $long->pivot('k', index: 'id', values: 'v', aggregateFunction: 'len'),
    'explode_single' => fn () => $base->select('id', 'l')->explode('l'),
    'explode_two' => fn () => (new DataFrame(['a' => [[1, 2], [3]], 'b' => [['x', 'y'], ['z']], 'c' => [0, 1]]))->explode('a', 'b'),
    'transpose' => fn () => $base->select('x', 'k')->head(3)->transpose(includeHeader: true),
    'with_row_index' => fn () => $base->select('g')->withRowIndex('i', offset: 10),

    // concat
    'concat_vertical' => fn () => concat([$A, $B]),
    'concat_horizontal' => fn () => concat([$A, $C2], how: 'horizontal'),
    'concat_diagonal' => fn () => concat([$A, $C], how: 'diagonal'),
    'concat_vertical_relaxed' => fn () => concat([$A, new DataFrame(['a' => [1.5], 'b' => ['q']])], how: 'vertical_relaxed'),
    'concat_diagonal_relaxed' => fn () => concat([$A, new DataFrame(['a' => [0.5], 'c' => [true]])], how: 'diagonal_relaxed'),

    // describe
    'describe_numeric' => fn () => $base->select('x', 'y')->describe(),
    'describe_mixed' => fn () => $base->select('x', 'g', 'b', 'd')->describe(),
    'describe_percentiles' => fn () => $base->select('y')->describe(percentiles: [0.1, 0.9]),

    // frame aggregates
    'frame_sum' => fn () => $base->select('g', 'x', 'y', 'b')->sum(),
    'frame_mean' => fn () => $base->select('x', 'y', 'b')->mean(),
    'frame_min_max' => fn () => concat([$base->select('g', 'x', 'y', 'd')->min(), $base->select('g', 'x', 'y', 'd')->max()]),
    'frame_std_var' => fn () => concat([$base->select('x', 'y')->std(), $base->select('x', 'y')->var(ddof: 0)]),
    'frame_median' => fn () => $base->select('x', 'y')->median(),
    'frame_n_unique' => fn () => $S(col('g', 'x', 'b')->nUnique()),

    // windows
    'over_sum' => fn () => $S('id', col('x')->sum()->over('g')->alias('sx')),
    'over_multi' => fn () => $S('id', col('y')->mean()->over(['g', 'b'])->alias('m'), len()->over('g')->alias('n')),
    'over_order_by' => fn () => $S('id', col('x')->cumSum()->over('g', orderBy: 'id')->alias('cs')),
    'over_order_by_desc' => fn () => $S('id', col('id')->first()->over('g', orderBy: 'y', descending: true, nullsLast: true)->alias('f')),
    'over_rank' => fn () => $S('id', col('y')->rank('dense')->over('b')->alias('r')),
    'cum_fns' => fn () => $S(
        col('x')->cumSum()->alias('sum'),
        col('x')->cumProd()->alias('prod'),
        col('y')->cumMin()->alias('min'),
        col('y')->cumMax()->alias('max'),
        col('x')->cumCount()->alias('count'),
        col('x')->cumSum(reverse: true)->alias('rsum'),
    ),
    'shift' => fn () => $S(
        col('x')->shift()->alias('s1'),
        col('x')->shift(-2)->alias('sm2'),
        col('g')->shift(1, fillValue: '-')->alias('sf'),
        col('d')->shift(1)->alias('sd'),
    ),
    'diff' => fn () => $S(col('x')->diff()->alias('d1'), col('y')->diff(2)->alias('d2'), col('t')->diff()->alias('dt')),
    'pct_change' => fn () => $S(col('y')->pctChange()->alias('p')),
    'rank_methods' => fn () => $S(
        'k',
        col('k')->rank('average')->alias('avg'),
        col('k')->rank('min')->alias('min'),
        col('k')->rank('max')->alias('max'),
        col('k')->rank('dense')->alias('dense'),
        col('k')->rank('ordinal')->alias('ord'),
        col('k')->rank('dense', descending: true)->alias('dd'),
    ),
    'rolling' => fn () => $S(
        col('x')->rollingMean(2)->alias('m2'),
        col('x')->rollingMean(3, minPeriods: 1)->alias('m3'),
        col('y')->rollingMean(3, center: true)->alias('mc'),
        col('id')->rollingSum(3)->alias('s3'),
        col('id')->rollingMin(2)->alias('min2'),
        col('id')->rollingMax(2)->alias('max2'),
        col('id')->rollingStd(3)->alias('std3'),
    ),

    // when / then / otherwise
    'when_basic' => fn () => $S('id', when(col('x')->gt(2))->then(lit('big'))->otherwise(lit('small'))->alias('w')),
    'when_chain' => fn () => $S('id', when(col('x')->lt(2))->then(lit(1))->when(col('x')->lt(5))->then(lit(2))->otherwise(lit(3))->alias('w')),
    'when_no_otherwise' => fn () => $S('id', when(col('g')->eq('a'))->then(col('y'))->alias('w')),
    'when_multi_cond' => fn () => $S('id', when(col('x')->gt(1), col('b'))->then(col('x'))->otherwise(col('k'))->alias('w')),
    'when_supertype' => fn () => $S('id', when(col('b'))->then(col('x'))->otherwise(col('y'))->alias('w')),

    // str namespace
    'str_contains' => fn () => $S(
        $s()->contains('a')->alias('re'),
        $s()->contains('-', literal: true)->alias('lit'),
        $s()->contains('\d+')->alias('digits'),
        $s()->contains('(?i)BANANA')->alias('ci'),
        $s()->contains('.', literal: true)->alias('dot'),
    ),
    'str_replace' => fn () => $S(
        $s()->replace('a', 'A')->alias('r1'),
        $s()->replaceAll('a', 'A')->alias('ra'),
        $s()->replace('(\w+)-(\d+)', '$2-$1')->alias('grp'),
        $s()->replaceAll('.', '!', literal: true)->alias('lit'),
        $s()->replaceAll('\s+', '_')->alias('ws'),
    ),
    'str_slice' => fn () => $S($s()->slice(1, 3)->alias('a'), $s()->slice(-3)->alias('b'), $s()->head(2)->alias('h'), $s()->tail(2)->alias('t')),
    'str_split' => fn () => $S($s()->split(' ')->alias('sp'), $s()->split('-', inclusive: true)->alias('inc')),
    'str_strip' => fn () => $S($s()->stripChars()->alias('a'), $s()->stripChars('ae ')->alias('b'), $s()->stripCharsStart()->alias('c'), $s()->stripCharsEnd('y')->alias('d')),
    'str_case' => fn () => $S($s()->toUppercase()->alias('u'), $s()->toLowercase()->alias('l')),
    'str_len' => fn () => $S($s()->lenChars()->alias('chars'), $s()->lenBytes()->alias('bytes')),
    'str_pad' => fn () => $S($s()->padStart(10, '*')->alias('ps'), $s()->padEnd(10)->alias('pe'), col('x')->cast('String')->str()->zfill(3)->alias('z')),
    'str_extract' => fn () => $S(
        $s()->extract('(\d+)', 1)->alias('num'),
        $s()->extract('^(\w)(\w)', 2)->alias('second'),
        $s()->extractAll('[aeiou]')->alias('vowels'),
    ),
    'str_count_matches' => fn () => $S($s()->countMatches('e')->alias('e'), $s()->countMatches('.', literal: true)->alias('dots'), $s()->countMatches('\w+')->alias('words')),
    'str_starts_ends' => fn () => $S($s()->startsWith('a')->alias('sw'), $s()->endsWith('y')->alias('ew')),
    'str_to_integer' => fn () => $nums->select(col('n')->str()->toInteger(strict: false)->alias('i')),
    'str_to_integer_base' => fn () => (new DataFrame(['h' => ['ff', '10', '7f']]))->select(col('h')->str()->toInteger(base: 16)),
    'str_find_reverse' => fn () => $S($s()->find('e')->alias('f'), $s()->reverse()->alias('r')),
    'str_join_agg' => fn () => $S($s()->join('|')->alias('j'), $s()->join('|', ignoreNulls: false)->alias('jn')),
    'str_to_date_datetime' => fn () => (new DataFrame(['a' => ['2024-03-05', '2021-12-31'], 'b' => ['05/03/2024 13:14', '31/12/2021 00:00']]))->select(
        col('a')->str()->toDate(), col('b')->str()->toDatetime('%d/%m/%Y %H:%M'),
    ),

    // dt namespace
    'dt_parts' => fn () => $S(
        $t()->year()->alias('y'), $t()->month()->alias('m'), $t()->day()->alias('d'), $t()->weekday()->alias('wd'),
        $t()->hour()->alias('h'), $t()->minute()->alias('mi'), $t()->second()->alias('s'),
        $t()->ordinalDay()->alias('od'), $t()->quarter()->alias('q'), $t()->week()->alias('w'),
    ),
    'date_parts' => fn () => $S($d()->year()->alias('y'), $d()->month()->alias('m'), $d()->day()->alias('dd'), $d()->weekday()->alias('wd'), $d()->isLeapYear()->alias('leap')),
    'dt_strftime' => fn () => $S($t()->strftime('%Y/%m/%d %H:%M')->alias('a'), $d()->strftime('%d %b %Y')->alias('b'), $t()->toString('%H-%M-%S')->alias('c')),
    'dt_truncate' => fn () => $S($t()->truncate('1mo')->alias('mo'), $t()->truncate('1w')->alias('w'), $t()->truncate('1d')->alias('d'), $t()->truncate('1h')->alias('h'), $d()->truncate('1mo')->alias('dmo')),
    'dt_round' => fn () => $S($t()->round('1h')->alias('h')),
    'dt_epoch' => fn () => $S($t()->epoch('s')->alias('s'), $t()->epoch('ms')->alias('ms'), $t()->epoch()->alias('us'), $d()->epoch('d')->alias('days'), $t()->timestamp('ms')->alias('ts')),
    'dt_date_time' => fn () => $S($t()->date()->alias('date'), $t()->time()->alias('time'), $d()->monthStart()->alias('ms'), $d()->monthEnd()->alias('me')),
    'dt_offset_by' => fn () => $S($d()->offsetBy('1mo')->alias('a'), $t()->offsetBy('-2d')->alias('b'), $d()->offsetBy('1y')->alias('c')),
    'dt_duration' => fn () => $S(
        col('t')->sub(col('t')->min())->alias('dur'),
        col('t')->sub(col('t')->min())->dt()->totalHours()->alias('hrs'),
        col('d')->sub(col('d')->min())->dt()->totalDays()->alias('days'),
    ),
    'dt_cast_time_unit' => fn () => $S($t()->castTimeUnit('ms')->alias('ms')),
    'dt_time_zone' => fn () => $S($t()->replaceTimeZone('UTC')->dt()->convertTimeZone('Asia/Tokyo')->dt()->hour()->alias('h'), $t()->replaceTimeZone('UTC')->alias('utc')),

    // list namespace
    'list_aggs' => fn () => $S($li()->len()->alias('len'), $li()->sum()->alias('sum'), $li()->mean()->alias('mean'), $li()->min()->alias('min'), $li()->max()->alias('max')),
    'list_get' => fn () => $S($li()->get(0, nullOnOob: true)->alias('g0'), $li()->get(-1, nullOnOob: true)->alias('gl'), $li()->first()->alias('f'), $li()->last()->alias('la')),
    'list_contains' => fn () => $S($li()->contains(2)->alias('c2'), $li()->contains(null)->alias('cn')),
    'list_join' => fn () => $S(col('s')->str()->split(' ')->list()->join('+')->alias('j')),
    'list_eval' => fn () => $S($li()->eval(element()->mul(10))->alias('x10'), $li()->eval(element()->rank())->alias('rank')),
    'list_sort_unique' => fn () => $S($li()->sort(descending: true)->alias('sd'), $li()->reverse()->alias('rv'), $li()->slice(1, 2)->alias('sl'), $li()->head(1)->alias('h')),
    'list_unique' => fn () => (new DataFrame(['a' => [[3, 1, 3, 2], [null, null]]]))->select(col('a')->list()->unique(maintainOrder: true), col('a')->list()->nUnique()->alias('nu')),
    'list_concat_list' => fn () => $S(concat_list('x', 'k')->alias('cl')),

    // cast
    'cast_numeric' => fn () => $S(
        col('x')->cast('Float64')->alias('xf'),
        col('y')->cast('Int64')->alias('yi'),
        col('y')->cast('Int32')->alias('y32'),
        col('b')->cast('Int8')->alias('b8'),
        col('x')->cast('Boolean')->alias('xb'),
    ),
    'cast_to_string' => fn () => $S(
        col('x')->cast('String')->alias('xs'),
        col('y')->cast('String')->alias('ys'),
        col('b')->cast('String')->alias('bs'),
        col('d')->cast('String')->alias('ds'),
        col('t')->cast('String')->alias('ts'),
    ),
    'cast_temporal' => fn () => $S(col('d')->cast('Datetime')->alias('dt'), col('t')->cast('Date')->alias('date'), col('d')->cast('Int32')->alias('di')),
    'cast_strict_fails' => fn () => $S(col('s')->cast('Int64')),
    'cast_non_strict' => fn () => $S(col('s')->cast('Int64', strict: false)->alias('si'), col('x')->mul(50)->cast('UInt8', strict: false)->alias('u8')),
    'cast_overflow_strict_fails' => fn () => $S(col('x')->mul(50)->cast('UInt8')),
    'cast_string_to_float' => fn () => (new DataFrame(['a' => ['1.5', '-2', '1e3', 'x', null]]))->select(col('a')->cast('Float64', strict: false)),
    'cast_frame_map' => fn () => $base->select('x', 'y', 'k')->cast(['x' => 'Float32', 'k' => 'String']),

    // isIn / isBetween / clip / round
    'is_in' => fn () => $S(
        col('x')->isIn([1, 2, 3])->alias('x'),
        col('g')->isIn(['a', 'c'])->alias('g'),
        col('g')->isIn(['a', null], nullsEqual: true)->alias('gn'),
        col('y')->isIn([1.5, 3.0])->alias('y'),
    ),
    'is_between' => fn () => $S(
        col('x')->isBetween(2, 5)->alias('both'),
        col('x')->isBetween(2, 5, closed: 'left')->alias('left'),
        col('x')->isBetween(2, 5, closed: 'right')->alias('right'),
        col('x')->isBetween(2, 5, closed: 'none')->alias('none'),
        col('y')->isBetween(0, 2.25)->alias('f'),
    ),
    'is_between_dates' => fn () => $S(col('t')->isBetween($dt('2024-01-01'), $dt('2024-12-31'))->alias('t')),
    'clip' => fn () => $S(
        col('x')->clip(2, 5)->alias('both'),
        col('y')->clip(lowerBound: 0)->alias('lo'),
        col('y')->clip(upperBound: 2)->alias('hi'),
        col('x')->clip(1.5, 4.5)->alias('fb'),
    ),
    'round' => fn () => $rnd->select(
        col('f')->round(0)->alias('r0'),
        col('f')->round(1)->alias('r1'),
        col('f')->round(2)->alias('r2'),
        col('f')->round(0, mode: 'half_away_from_zero')->alias('away'),
        col('f')->floor()->alias('floor'),
        col('f')->ceil()->alias('ceil'),
    ),
    'math' => fn () => $S(col('y')->abs()->alias('abs'), col('x')->sqrt()->alias('sqrt'), col('x')->log()->alias('ln'), col('x')->log(2)->alias('log2'), col('x')->exp()->alias('exp'), col('y')->sign()->alias('sign')),

    // valueCounts
    'value_counts_expr' => fn () => $S(col('g')->valueCounts(sort: true))->unnest('g')->sort(['count', 'g'], descending: [true, false], nullsLast: true),
    'value_counts_series' => fn () => $base->getColumn('k')->valueCounts(sort: true)->sort(['count', 'k'], descending: [true, false], nullsLast: true),
    'value_counts_normalize' => fn () => $bs->valueCounts(normalize: true, name: 'share')->sort('b'),

    // Series
    'series_int_aggs' => fn () => [
        'sum' => $xs->sum(), 'mean' => $xs->mean(), 'median' => $xs->median(), 'min' => $xs->min(), 'max' => $xs->max(),
        'std' => $xs->std(), 'var' => $xs->var(), 'product' => $xs->product(), 'count' => $xs->count(), 'len' => $xs->len(),
        'null_count' => $xs->nullCount(), 'n_unique' => $xs->nUnique(), 'first' => $xs->first(), 'last' => $xs->last(),
        'arg_min' => $xs->argMin(), 'arg_max' => $xs->argMax(),
    ],
    'series_float_aggs' => fn () => [
        'sum' => $ys->sum(), 'mean' => $ys->mean(), 'median' => $ys->median(), 'min' => $ys->min(), 'max' => $ys->max(),
        'std' => $ys->std(), 'std0' => $ys->std(ddof: 0), 'var' => $ys->var(), 'product' => $ys->product(),
        'q25' => $ys->quantile(0.25), 'q25_linear' => $ys->quantile(0.25, 'linear'), 'q75_nearest' => $ys->quantile(0.75, 'nearest'),
        'first' => $ys->first(), 'last' => $ys->last(),
    ],
    'series_quantile_int_arg' => fn () => ['q0' => $ys->quantile(0), 'q1' => $ys->quantile(1)],
    'series_str_aggs' => fn () => ['min' => $gs->min(), 'max' => $gs->max(), 'n_unique' => $gs->nUnique(), 'count' => $gs->count(), 'first' => $gs->first(), 'last' => $gs->last()],
    'series_bool_aggs' => fn () => ['any' => $bs->any(), 'all' => $bs->all(), 'all_nulls' => $bs->all(ignoreNulls: false), 'sum' => $bs->sum(), 'mean' => $bs->mean()],
    'series_ops' => fn () => new DataFrame([
        'mul' => $xs->mul(2), 'cs' => $xs->cumSum(), 'diff' => $xs->diff(), 'shift' => $xs->shift(1),
        'rank' => $xs->rank(), 'clip' => $xs->clip(2, 5), 'isin' => $xs->isIn([1, 7]), 'round' => $ys->round(1),
    ]),
    'series_sort' => fn () => $xs->sort(descending: true, nullsLast: true),
    'series_unique' => fn () => $gs->unique(maintainOrder: true),
    'series_empty_aggs' => fn () => (fn (Series $e) => ['sum' => $e->sum(), 'mean' => $e->mean(), 'min' => $e->min(), 'max' => $e->max(), 'std' => $e->std()])(new Series('e', [], 'Int64')),

    // misc frame ops
    'head_tail_slice' => fn () => concat([$base->select('id')->head(2), $base->select('id')->tail(1), $base->select('id')->slice(-3, 2)]),
    'rename_drop_reverse' => fn () => $base->select('id', 'g', 'x')->rename(['g' => 'group'])->drop('x')->reverse(),
    'struct_pack_field' => fn () => $S(struct_('g', 'x')->alias('st'))->withColumns(col('st')->struct()->field('x')->alias('sx')),
    'unique_counts_mode' => fn () => $S(col('k')->dropNulls()->mode()->sort()->alias('mode')),
    'top_bottom_k' => fn () => $S(col('y')->topK(2)->alias('top'), col('y')->bottomK(2)->alias('bottom')),

    // asof joins, time windows, and rolling
    'join_asof_backward' => fn () => $tsf->joinAsof($quotes, on: 't'),
    'join_asof_forward_by' => fn () => $tsf->joinAsof($quotes, on: 't', by: 'g', strategy: 'forward', checkSortedness: false),
    'join_asof_nearest_tolerance' => fn () => $tsf->joinAsof($quotes, on: 't', strategy: 'nearest', tolerance: '2h'),
    'join_asof_int_tolerance' => fn () => (new DataFrame(['k' => [1, 5, 10]]))->joinAsof(
        new DataFrame(['k' => [2, 4, 9], 'r' => ['a', 'b', 'c']]), on: 'k', tolerance: 1, allowExactMatches: false),
    'group_by_dynamic_day' => fn () => $tsf->groupByDynamic('t', every: '1d')->agg(col('v')->sum(), len()),
    'group_by_dynamic_by_bounds' => fn () => $tsf->groupByDynamic('t', every: '12h', period: '1d', closed: 'both', groupBy: 'g', includeBoundaries: true)->agg(col('v')->max()),
    'group_by_dynamic_label_offset' => fn () => $tsf->groupByDynamic('t', every: '1d', offset: '6h', label: 'right', startBy: 'datapoint')->agg(col('v')->first()),
    'rolling_frame' => fn () => $tsf->rolling('t', period: '1d')->agg(col('v')->sum()->alias('s'), col('v')->count()->alias('n')),
    'rolling_frame_by' => fn () => $tsf->rolling('t', period: '2h', offset: '-1h', closed: 'both', groupBy: 'g')->agg(col('v')->sum()),
    'rolling_by_exprs' => fn () => $tsf->select(
        't',
        col('v')->rollingMeanBy('t', '1d')->alias('mean'),
        col('v')->fillNull(0)->rollingSumBy('t', '3h', closed: 'both')->alias('sum'),
        col('v')->rollingMaxBy('t', '2d', minPeriods: 2)->alias('max'),
        col('v')->rollingMinBy('t', '1d', closed: 'left')->alias('min'),
        col('v')->rollingMedianBy('t', '1d')->alias('median'),
        col('v')->rollingStdBy('t', '1d')->alias('std'),
        col('v')->rollingVarBy('t', '1d', ddof: 0)->alias('var'),
        col('v')->rollingQuantileBy('t', '1d', quantile: 0.5)->alias('q'),
    ),
    'rolling_quantile_skew' => fn () => $S(
        col('y')->rollingQuantile(0.5, windowSize: 2)->alias('q'),
        col('x')->rollingQuantile(0.25, interpolation: 'linear', windowSize: 3, minPeriods: 1)->alias('ql'),
        col('id')->rollingSkew(3)->alias('skew'),
        col('id')->cast('Float64')->pow(2)->rollingKurtosis(4)->alias('kurt'),
    ),
    'ewm' => fn () => $S(
        col('y')->ewmMean(alpha: 0.5)->alias('mean'),
        col('y')->ewmMean(span: 3, adjust: false)->alias('mean_span'),
        col('y')->ewmStd(com: 1)->alias('std'),
        col('x')->ewmVar(halfLife: 2, ignoreNulls: true, bias: true)->alias('var'),
    ),
    'ewm_mean_by' => fn () => $tsf->select(col('v')->ewmMeanBy('t', halfLife: '1d')),
    'ewm_no_alpha' => fn () => $S(col('y')->ewmMean()),

    // binning, runs, and searching
    'cut_qcut' => fn () => $S(
        'y',
        col('y')->cut([0, 2])->alias('cut'),
        col('y')->cut([0, 2], labels: ['lo', 'mid', 'hi'], leftClosed: true)->alias('cutl'),
        col('y')->qcut([0.5])->alias('q'),
        col('y')->qcut(2, labels: ['a', 'b'])->alias('q2'),
    )->withColumns(Phpolars\all()->exclude('y')->cast('String')),
    'cut_include_breaks' => fn () => $S(col('y')->cut([0, 2], includeBreaks: true)->alias('c'))->unnest('c')->withColumns(col('category')->cast('String')),
    'rle' => fn () => (new DataFrame(['a' => [1, 1, 2, 2, 2, null, null, 1]]))->select(col('a')->rle()->alias('r'))->unnest('r'),
    'rle_id' => fn () => (new DataFrame(['a' => [1, 1, 2, 2, 2, null, null, 1]]))->select(col('a')->rleId()),
    'search_sorted' => fn () => (new DataFrame(['a' => [1, 2, 2, 4, 7]]))->select(
        col('a')->searchSorted(2)->alias('any'), col('a')->searchSorted(2, 'left')->alias('left'),
        col('a')->searchSorted(2, 'right')->alias('right'), col('a')->searchSorted(9)->alias('end'),
    ),
    'search_sorted_many' => fn () => (new DataFrame(['a' => [1, 2, 2, 4, 7]]))->select(col('a')->searchSorted([0, 3, 7], 'left')),
    'peaks_entropy' => fn () => (new DataFrame(['a' => [1, 3, 2, 2, 5, 4]]))->select(
        col('a')->peakMax()->alias('max'), col('a')->peakMin()->alias('min'),
        col('a')->entropy()->alias('e'), col('a')->entropy(base: 2, normalize: false)->alias('e2'),
    ),
    'hist_bins' => fn () => (new DataFrame(['a' => [1.0, 2.5, 3.0, 4.5, 5.0, 2.0]]))->select(col('a')->hist(bins: [0, 2, 4, 6])),
    'hist_count_breaks' => fn () => (new DataFrame(['a' => [1.0, 2.5, 3.0, 4.5, 5.0, 2.0]]))->select(
        col('a')->hist(binCount: 2, includeBreakpoint: true, includeCategory: true)->alias('h'))->unnest('h')->withColumns(col('category')->cast('String')),
    'interpolate_by' => fn () => (new DataFrame(['a' => [1.0, null, 3.0, null, null, 6.0, null], 'b' => [1, 2, 5, 6, 7, 10, 11]]))->select(col('a')->interpolateBy('b')),
    'is_close_index_of' => fn () => (new DataFrame(['a' => [1.0, 1.0000000001, 2.0, NAN], 'k' => [3, 1, 3, 2]]))->select(
        col('a')->isClose(1.0)->alias('close'),
        col('a')->isClose(1.5, absTol: 0.6)->alias('abs'),
        col('a')->isClose(NAN, nansEqual: true)->alias('nan'),
        col('k')->indexOf(3)->alias('idx'),
        col('k')->indexOf(9)->alias('missing'),
    ),
    'arg_unique_bounds' => fn () => (new DataFrame(['k' => [3, 1, 3, 2, 1]]))->select(
        col('k')->argUnique()->alias('au')->implode(), col('k')->upperBound()->alias('ub'), col('k')->cast('Int8')->lowerBound()->alias('lb'),
    ),
    'trig_extra' => fn () => $rnd->select(
        col('f')->degrees()->alias('deg'), col('f')->radians()->alias('rad'), col('f')->sinh()->alias('sinh'),
        col('f')->cosh()->alias('cosh'), col('f')->tanh()->alias('tanh'), col('f')->arcsinh()->alias('asinh'),
        col('f')->abs()->add(1)->arccosh()->alias('acosh'), col('f')->truediv(10)->arctanh()->alias('atanh'), col('f')->cot()->alias('cot'),
    ),

    // frame verbs
    'frame_shift' => fn () => $base->select('id', 'x')->shift(2),
    'frame_shift_fill' => fn () => $base->select('id', 'x')->shift(-1, fillValue: 0),
    'frame_interpolate' => fn () => $base->select('x', 'y')->interpolate(),
    'frame_quantile_product' => fn () => concat([
        $base->select('x', 'y', 'k')->quantile(0.5, 'linear'),
        $base->select('x', 'y', 'k')->cast('Float64')->product(),
    ]),
    'frame_top_bottom_k' => fn () => concat([
        $base->select('id', 'y')->topK(2, by: 'y'),
        $base->select('id', 'k')->bottomK(3, by: ['k', 'id']),
        $base->select('id', 'x')->topK(2, by: 'x', reverse: true),
    ], how: 'diagonal'),
    'frame_gather_every' => fn () => $base->select('id', 'g')->gatherEvery(2, offset: 1),
    'frame_drop_nans' => fn () => $tsf->dropNans(),
    'frame_drop_nans_subset' => fn () => $tsf->withColumns(col('f')->alias('f2'))->dropNans(subset: 'f2'),
    'frame_is_duplicated' => fn () => new DataFrame([
        'dup' => $long->select('id', 'k')->isDuplicated(), 'uniq' => $long->select('id', 'k')->isUnique(),
    ]),
    'frame_upsample' => fn () => (new DataFrame([
        't' => [$dt('2024-01-01'), $dt('2024-01-04'), $dt('2024-01-02'), $dt('2024-01-03')],
        'g' => ['a', 'a', 'b', 'b'], 'v' => [1, 4, 2, 3],
    ]))->cast(['t' => 'Date'])->sort('t')->upsample('t', every: '1d', groupBy: 'g', maintainOrder: true),
    'frame_corr' => fn () => $base->select('id', 'x', 'y')->dropNulls()->corr(),
    'frame_insert_to_struct' => fn () => $base->select('id', 'g')->head(3)->insertColumn(1, new Series('n', [7, 8, 9]))->toStruct('st')->toFrame()->unnest('st'),
    'frame_approx_n_unique' => fn () => $base->select('g', 'k')->approxNUnique(),

    // functions
    'fn_date_range' => fn () => select(date_range($dt('2024-01-01'), $dt('2024-01-10'), '3d')->alias('d')),
    'fn_date_range_closed' => fn () => select(date_range($dt('2024-01-30'), $dt('2024-04-30'), '1mo', closed: 'left')->alias('d')),
    'fn_datetime_range' => fn () => select(datetime_range($dt('2024-01-01'), $dt('2024-01-02'), '6h')->alias('t')),
    'fn_date_ranges' => fn () => $base->select('d')->dropNulls()->head(2)->select(date_ranges(col('d'), col('d')->dt()->offsetBy('2d'))->alias('r')),
    'fn_int_ranges' => fn () => $base->select('k')->dropNulls()->select(int_ranges(col('k'))->alias('r'), int_ranges(1, col('k')->add(2), 2)->alias('r2')),
    'fn_date_datetime' => fn () => $base->select('k')->dropNulls()->select(
        date(2024, col('k'), 15)->alias('d'), datetime(2024, 1, col('k'), 12, col('k'), 30)->alias('t')),
    'fn_duration' => fn () => $base->select('k')->dropNulls()->select(
        duration(days: col('k'), hours: 1)->alias('d'), date(2024, 1, 1)->add(duration(weeks: col('k')))->alias('shifted')),
    'fn_from_epoch' => fn () => (new DataFrame(['s' => [0, 1_700_000_000], 'd' => [0, 19_700]]))->select(
        from_epoch('s')->alias('s'), from_epoch('d', timeUnit: 'd')->alias('d'), from_epoch(col('s')->mul(1000), timeUnit: 'ms')->alias('ms')),
    'fn_arg_where' => fn () => $S(arg_where(col('x')->gt(2))),
    'fn_corr_cov' => fn () => $S(
        corr('id', 'y')->alias('pearson'), corr('id', 'y', method: 'spearman')->alias('spearman'),
        cov('id', 'x')->alias('cov'), cov('id', 'x', ddof: 0)->alias('cov0')),
    'fn_nth' => fn () => $S(nth(0, 2)),
    'fn_repeat_linear_space' => fn () => select(repeat('a', 3)->alias('r'), linear_space(0, 1, 3)->alias('ls')),
    'fn_select' => fn () => select(lit(1)->alias('one'), lit(2)->mul(3)->alias('six')),
    'fn_stat_shorthands' => fn () => $S(
        std('y')->alias('std'), Phpolars\var_('y', ddof: 0)->alias('var'), quantile('y', 0.5)->alias('q'),
        implode('g')->alias('all'), approx_n_unique('g')->alias('au')),

    // namespaces: str, dt, list, cat, arr, and bin additions
    'str_many' => fn () => $words->select(
        col('w')->str()->containsAny(['fox', 'dog'])->alias('any'),
        col('w')->str()->containsAny(['fox', 'dog'], asciiCaseInsensitive: true)->alias('any_ci'),
        col('w')->str()->replaceMany(['quick', 'DOG'], 'X')->alias('rep'),
        col('w')->str()->replaceMany(['the' => 'a', 'fox' => 'cat'])->alias('rep_map'),
        col('w')->str()->extractMany(['o', 'a'])->alias('ext'),
        col('w')->str()->findMany(['o', 'a'])->alias('find'),
    ),
    'str_extract_groups' => fn () => (new DataFrame(['u' => ['id=7&n=ab', 'id=12', null]]))->select(
        col('u')->str()->extractGroups('id=(?<id>\d+)(?:&n=(\w+))?')->alias('g'))->unnest('g'),
    'str_split_exact' => fn () => (new DataFrame(['s' => ['a,b,c', 'd', null]]))->select(
        col('s')->str()->splitExact(',', 1)->struct()->renameFields(['e_field_0', 'e_field_1'])->alias('e'),
        col('s')->str()->splitExact(',', 1, inclusive: true)->struct()->renameFields(['i_field_0', 'i_field_1'])->alias('i'))->unnest('e', 'i'),
    'str_encode_decode' => fn () => (new DataFrame(['s' => ['hi', '', null]]))->select(
        col('s')->str()->encode('hex')->alias('hex'), col('s')->str()->encode('base64')->alias('b64'),
        col('s')->str()->encode('hex')->str()->decode('hex')->cast('String')->alias('back'),
    ),
    'str_to_decimal' => fn () => (new DataFrame(['s' => ['1.25', '-3.5', null]]))->select(col('s')->str()->toDecimal(2)->cast('Float64')),
    'str_normalize_escape' => fn () => $words->select(
        col('w')->str()->normalize('NFD')->str()->lenChars()->alias('nfd'), col('w')->str()->escapeRegex()->alias('esc')),
    'str_json' => fn () => (new DataFrame(['j' => ['{"a": 1, "b": "x"}', '{"a": null}', null]]))->select(
        col('j')->str()->jsonDecode('Struct(a: Int64, b: String)')->alias('d'),
        col('j')->str()->jsonPathMatch('$.b')->alias('jb'),
    )->unnest('d'),
    'dt_parts_extra' => fn () => $days->select(
        col('d')->dt()->millennium()->alias('mil'), col('d')->dt()->century()->alias('cen'), col('d')->dt()->daysInMonth()->alias('dim')),
    'dt_business' => fn () => $days->select(
        col('d')->dt()->addBusinessDays(1)->alias('next'),
        col('d')->dt()->addBusinessDays(-2, roll: 'forward')->alias('back'),
        col('d')->dt()->addBusinessDays(1, holidays: [$dt('2024-01-08')])->alias('hol'),
        col('d')->dt()->isBusinessDay()->alias('is'),
        col('d')->dt()->addBusinessDays(1, weekMask: [true, true, true, true, true, true, false])->alias('six'),
    ),
    'dt_combine_replace' => fn () => $days->select(
        col('d')->dt()->combine('12:30:00')->alias('c'),
        col('d')->dt()->replace(day: 1)->alias('first'),
        col('d')->cast('Datetime')->dt()->replace(hour: 6, minute: 15)->alias('t'),
    ),
    'dt_time_unit' => fn () => $S(
        col('t')->dt()->withTimeUnit('ms')->cast('Int64')->alias('ms'),
        col('t')->sub(col('t')->min())->dt()->totalNanoseconds()->alias('ns'),
    ),
    'list_more' => fn () => $S(
        col('l')->list()->any()->alias('any'),
        col('l')->list()->eval(element()->gt(2))->list()->all()->alias('all'),
        col('l')->list()->filter(element()->gt(2))->alias('filt'),
        col('l')->list()->gatherEvery(2)->alias('ge'),
        col('l')->list()->agg(element()->sum())->alias('agg'),
    ),
    'list_to_array' => fn () => (new DataFrame(['l' => [[1, 2], [3, 4]]]))->select(
        col('l')->list()->toArray(2)->alias('a'))->select(col('a')->arr()->sum()->alias('sum'), col('a')->arr()->max()->alias('max'), col('a')->arr()->toList()->alias('back')),
    'arr_aggs' => fn () => $arrs->select(
        col('a')->arr()->min()->alias('min'), col('a')->arr()->max()->alias('max'), col('a')->arr()->sum()->alias('sum'),
        col('a')->arr()->mean()->alias('mean'), col('a')->arr()->median()->alias('median'), col('a')->arr()->std()->alias('std'),
        col('a')->arr()->nUnique()->alias('nu'), col('a')->arr()->argMax()->alias('am'), col('a')->arr()->len()->alias('len'),
    ),
    'arr_ops' => fn () => $arrs->select(
        col('a')->arr()->sort(descending: true)->alias('sort'), col('a')->arr()->reverse()->alias('rev'),
        col('a')->arr()->get(1)->alias('get'), col('a')->arr()->first()->alias('first'), col('a')->arr()->last()->alias('last'),
        col('a')->arr()->contains(4)->alias('has4'), col('a')->arr()->countMatches(4)->alias('n4'),
        col('a')->arr()->shift(1)->alias('shift'), col('a')->arr()->head(2)->alias('head'), col('a')->arr()->tail(2)->alias('tail'),
        col('a')->arr()->unique(maintainOrder: true)->alias('uniq'),
        col('a')->cast('Array(String, 3)')->arr()->join('-')->alias('join'),
        col('a')->arr()->eval(element()->mul(2))->alias('eval'),
    ),
    'arr_explode_struct' => fn () => $arrs->head(2)->select(col('a')->arr()->toStruct()->alias('s'))->unnest('s'),
    'cat_ns' => fn () => $cats->select(
        col('c')->cat()->lenBytes()->alias('lb'), col('c')->cat()->lenChars()->alias('lc'),
        col('c')->cat()->startsWith('ba')->alias('sw'), col('c')->cat()->endsWith('le')->alias('ew'),
        col('c')->cat()->slice(1, 3)->alias('sl'),
    ),
    'cat_categories' => fn () => $cats->select(col('c')->cat()->getCategories()->sort()),
    'bin_ns' => fn () => (new DataFrame(['s' => ['hello', '', null]]))->select(col('s')->cast('Binary')->alias('b'))->select(
        col('b')->bin()->contains(lit('ell')->cast('Binary'))->alias('has'), col('b')->bin()->startsWith(lit('he')->cast('Binary'))->alias('sw'),
        col('b')->bin()->endsWith(lit('lo')->cast('Binary'))->alias('ew'), col('b')->bin()->size()->alias('size'), col('b')->bin()->size('kb')->alias('kb'),
        col('b')->bin()->encode('hex')->alias('hex'), col('b')->bin()->encode('base64')->alias('b64'),
        col('b')->bin()->encode('hex')->str()->decode('hex')->bin()->slice(1, 2)->cast('String')->alias('slice'),
    ),

    // Series methods that mirror Expr
    'series_mirrors_expr' => fn () => new DataFrame([
        'ewm' => $ys->ewmMean(alpha: 0.5), 'rq' => $ys->rollingQuantile(0.5, windowSize: 2),
        'rle' => $xs->rleId(), 'deg' => $ys->degrees(), 'cut' => $ys->cut([0, 2])->cast('String'),
    ]),
    'series_scalar_reductions' => fn () => [
        'skew' => $ys->skew(), 'kurtosis' => $ys->kurtosis(), 'entropy' => $xs->dropNulls()->entropy(),
        'nan_max' => $ys->nanMax(), 'approx' => $gs->approxNUnique(), 'index_of' => $xs->indexOf(7),
        'search' => $xs->dropNulls()->sort()->searchSorted(4),
    ],

    // selectors
    'selectors' => fn () => array_map(fn ($sel) => \implode(',', $wide->select($sel)->columns()), [
        'numeric' => cs\numeric(), 'integer' => cs\integer(), 'signed' => cs\signed_integer(), 'unsigned' => cs\unsigned_integer(),
        'float' => cs\float(), 'decimal' => cs\decimal(), 'temporal' => cs\temporal(), 'categorical' => cs\categorical(),
        'nested' => cs\nested(), 'struct' => cs\struct_(), 'string' => cs\string(), 'boolean' => cs\boolean(), 'binary' => cs\binary(),
        'date' => cs\date(), 'time' => cs\time(), 'all' => cs\all(), 'first' => cs\first(), 'last' => cs\last(),
        'list' => cs\list_(), 'list_str' => cs\list_(cs\string()), 'array' => cs\array_(), 'array_w2' => cs\array_(width: 2),
        'datetime' => cs\datetime(), 'datetime_ms' => cs\datetime('ms'), 'datetime_utc' => cs\datetime(timeZone: 'UTC'),
        'datetime_naive' => cs\datetime(timeZone: null), 'duration' => cs\duration(),
        'by_dtype' => cs\by_dtype('Int64', 'String'), 'by_name' => cs\by_name(['name', 'id']),
        'by_index' => cs\by_index([0, -1]), 'starts_with' => cs\starts_with('ts_', 'lst'), 'ends_with' => cs\ends_with('_utc'),
        'contains' => cs\contains('f'), 'matches' => cs\matches('^[a-c]'), 'exclude' => cs\exclude('id', cs\numeric()),
        'union' => cs\string()->or(cs\boolean()), 'intersect' => cs\numeric()->and(cs\starts_with('i')),
        'difference' => cs\numeric()->sub(cs\by_name('id')), 'complement' => cs\temporal()->not()->and(cs\nested()->not()),
        'xor' => cs\integer()->xor(cs\starts_with('i', 'f')),
    ]),
    'selectors_in_ops' => fn () => $S(cs\numeric()->sub(cs\by_name('id')))->withColumns(cs\float()->mul(2)->name()->suffix('_x2')),
];

// ---------------------------------------------------------------------------
// Comparison

/** Normalise a phpolars value to the JSON shape scripts/pyfixtures.py writes. */
function normalise(mixed $v, string $dtype): mixed {
    if ($v instanceof DateTimeInterface) {
        if ($dtype === 'Date' || (str_contains($dtype, 'Date') && !str_contains($dtype, 'Datetime'))) {
            return $v->format('Y-m-d');
        }
        return $v->format('Y-m-d\TH:i:s.u') . (preg_match('/^Datetime\(\w+, .+\)$/', $dtype) ? $v->format('O') : '');
    }
    if (is_float($v) && !is_finite($v)) {
        return ['$float' => is_nan($v) ? 'NaN' : ($v > 0 ? 'INF' : '-INF')];
    }
    if (is_array($v)) {
        $inner = preg_match('/^(?:List|Array)\((.*)\)$/', $dtype, $m) ? $m[1] : $dtype;
        return array_map(fn ($x) => normalise($x, $inner), $v);
    }
    return $v;
}

/** Strict equality, except floats compare with a 1e-9 relative tolerance. */
function matches(mixed $expected, mixed $actual): bool {
    if (is_float($expected) && is_float($actual)) {
        return $expected === $actual || abs($expected - $actual) <= 1e-9 * max(abs($expected), abs($actual));
    }
    if (is_array($expected) && is_array($actual)) {
        if (count($expected) !== count($actual) || array_keys($expected) !== array_keys($actual)) {
            return false;
        }
        foreach ($expected as $k => $e) {
            if (!matches($e, $actual[$k])) {
                return false;
            }
        }
        return true;
    }
    return $expected === $actual;
}

function shape_of(mixed $out): array {
    if ($out instanceof Series) {
        $out = $out->toFrame();
    }
    if ($out instanceof DataFrame) {
        $dtypes = $out->dtypes();
        $rows = array_map(fn ($row) => array_map(fn ($v, $t) => normalise($v, $t), $row, $dtypes), $out->rows());
        return ['columns' => $out->columns(), 'dtypes' => $dtypes, 'rows' => $rows];
    }
    if (is_array($out)) {
        $types = ['integer' => 'int', 'double' => 'float', 'boolean' => 'bool', 'string' => 'string', 'NULL' => 'null'];
        return [
            'columns' => array_keys($out),
            'dtypes' => array_map(fn ($v) => $types[gettype($v)] ?? get_debug_type($v), array_values($out)),
            'rows' => [array_map(fn ($v) => normalise($v, ''), array_values($out))],
            'scalar' => true,
        ];
    }
    return ['unexpected' => get_debug_type($out)];
}

$enc = fn ($v) => json_encode($v, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES | JSON_PRESERVE_ZERO_FRACTION | JSON_PARTIAL_OUTPUT_ON_ERROR);
$failures = [];
foreach ($fixture['cases'] as $name => $expected) {
    if (!isset($cases[$name])) {
        $failures[$name] = "no PHP case defined";
        continue;
    }
    try {
        $actual = shape_of($cases[$name]());
    } catch (Throwable $e) {
        $actual = ['error' => (new ReflectionClass($e))->getShortName(), 'message' => strtok($e->getMessage(), "\n")];
    }
    if (isset($expected['error']) || isset($actual['error'])) {
        $ok = ($expected['error'] ?? null) === ($actual['error'] ?? null);
    } else {
        $ok = $expected['columns'] === $actual['columns'] && $expected['dtypes'] === $actual['dtypes'] && matches($expected['rows'], $actual['rows']);
    }
    if ($ok) {
        $GLOBALS['passed'] = ($GLOBALS['passed'] ?? 0) + 1;
        continue;
    }
    $msg = '';
    foreach (['error', 'message', 'columns', 'dtypes'] as $key) {
        if (($expected[$key] ?? null) !== ($actual[$key] ?? null)) {
            $msg .= sprintf("  %-8s py:  %s\n  %-8s php: %s\n", $key, $enc($expected[$key] ?? null), '', $enc($actual[$key] ?? null));
        }
    }
    if (!isset($expected['error']) && !isset($actual['error'])) {
        foreach ($expected['rows'] as $i => $row) {
            if (!matches($row, $actual['rows'][$i] ?? null)) {
                $msg .= sprintf("  row %-4d py:  %s\n  %-8s php: %s\n", $i, $enc($row), '', $enc($actual['rows'][$i] ?? null));
            }
        }
        for ($i = count($expected['rows']); $i < count($actual['rows']); $i++) {
            $msg .= sprintf("  row %-4d py:  (none)\n  %-8s php: %s\n", $i, '', $enc($actual['rows'][$i]));
        }
    }
    $failures[$name] = rtrim($msg);
}
foreach (array_diff_key($cases, $fixture['cases']) as $name => $_) {
    $failures[$name] = 'no py fixture; rerun scripts/pyfixtures.py';
}

foreach ($failures as $name => $msg) {
    fwrite(STDERR, "MISMATCH $name\n$msg\n");
}
printf("pyfixtures.php: %d of %d cases match py-polars %s\n", $GLOBALS['passed'] ?? 0, count($fixture['cases']), $fixture['polars_version']);
if ($failures) {
    fwrite(STDERR, count($failures) . " mismatched: " . implode(', ', array_keys($failures)) . "\n");
    exit(1);
}
