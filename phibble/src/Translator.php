<?php

declare(strict_types=1);

namespace Phpolars\Phibble;

use DateTimeInterface;
use InvalidArgumentException;
use Phibble\Exception\ColumnNotFound;
use Phibble\Expr\Call;
use Phibble\Expr\Col;
use Phibble\Expr\Desc;
use Phibble\Expr\Expr as PhibbleExpr;
use Phibble\Expr\Functions;
use Phibble\Expr\Lit;
use Phpolars\Expr;

use function Phpolars\{coalesce, col, concat_str, int_range, len, lit, max_horizontal, min_horizontal, struct_, when, all_horizontal};

/**
 * Translates Phibble\Expr trees into Polars expressions with dplyr's
 * semantics: NA propagates, aggregates default to na_rm = FALSE, and inside
 * a grouped mutate or filter, aggregates and window functions run per group.
 *
 * @internal
 */
final class Translator
{
    /**
     * Arguments that must hold no NA, checked before the verb runs: R's
     * quantile() raises an error on NA unless na.rm is TRUE.
     *
     * @var list<Expr>
     */
    public array $naChecks = [];

    /**
     * @param array<string, string> $schema Column name => Polars data type name.
     * @param list<string> $partition Group keys for mutate/filter; empty when ungrouped.
     */
    public function __construct(
        private readonly array $schema,
        private readonly array $partition = [],
    ) {}

    /** A row-level expression, as mutate() and filter() evaluate it. */
    public function row(PhibbleExpr $e): Expr
    {
        return $this->node($e, false);
    }

    /** An expression evaluated once per group, as summarise() needs. */
    public function aggregate(PhibbleExpr $e): Expr
    {
        return $this->node($e, true);
    }

    /** True when the expression yields one value per group. */
    public static function isScalar(PhibbleExpr $e): bool
    {
        return match (true) {
            $e instanceof Lit => true,
            $e instanceof Col, $e instanceof Desc => false,
            $e instanceof Call => match (Functions::kind($e->fn)) {
                'agg' => true,
                'window' => false,
                default => array_all($e->args, self::isScalar(...)),
            },
        };
    }

    /**
     * @param bool $inGroup True inside summarise(), or inside the arguments of an
     *     aggregate or window that will itself run per group.
     */
    private function node(PhibbleExpr $e, bool $inGroup): Expr
    {
        return match (true) {
            $e instanceof Col => $this->col($e->name),
            $e instanceof Lit => self::lit($e),
            $e instanceof Desc => throw new InvalidArgumentException('desc() is only allowed in arrange(), ranking functions and order_by.'),
            $e instanceof Call => $this->call($e, $inGroup),
        };
    }

    private function col(string $name): Expr
    {
        if (! array_key_exists($name, $this->schema)) {
            throw ColumnNotFound::named($name, array_keys($this->schema));
        }

        return col($name);
    }

    private static function lit(Lit $l): Expr
    {
        $v = $l->value;
        if ($v instanceof DateTimeInterface) {
            return $l->dtype === 'date' ? lit($v)->cast('Date') : lit($v);
        }

        return $v === null ? lit(null)->cast('Boolean') : lit($v);
    }

    private function call(Call $c, bool $inGroup): Expr
    {
        $kind = Functions::kind($c->fn);
        if ($kind === 'agg' || $kind === 'window') {
            $out = $kind === 'agg' ? $this->agg($c) : $this->window($c);
            if ($inGroup) {
                return $out;
            }
            $order = $c->option('order_by');
            if ($kind === 'window' && $order !== null && in_array($c->fn, ['lag', 'lead'], true)) {
                [$by, $desc] = $this->orderKey($order);

                return $out->over($this->partition ?: lit(1), orderBy: $by, descending: $desc, nullsLast: true);
            }

            return $this->partition === [] ? $out : $out->over($this->partition);
        }
        $a = fn (int $i) => $this->node($c->args[$i], $inGroup);
        $args = fn () => array_map(fn (PhibbleExpr $x) => $this->node($x, $inGroup), $c->args);

        return match ($c->fn) {
            '+' => $a(0)->add($a(1)),
            '-' => $a(0)->sub($a(1)),
            '*' => $a(0)->mul($a(1)),
            '/' => $a(0)->truediv($a(1)),
            '%%' => $a(0)->mod($a(1)),
            '^' => $a(0)->cast('Float64')->pow($a(1)),
            'neg' => $a(0)->neg(),
            '==' => $a(0)->eq($a(1)),
            '!=' => $a(0)->ne($a(1)),
            '<' => $a(0)->lt($a(1)),
            '<=' => $a(0)->le($a(1)),
            '>' => $a(0)->gt($a(1)),
            '>=' => $a(0)->ge($a(1)),
            '&' => $a(0)->and($a(1)),
            '|' => $a(0)->or($a(1)),
            '!' => $a(0)->not(),
            'is_na' => $this->type($c->args[0]) === 'float' ? $a(0)->isNull()->or($a(0)->isNan()) : $a(0)->isNull(),
            'if_else' => when($a(0)->isNull())->then($this->optional($c, 'missing', $inGroup) ?? lit(null))
                ->when($a(0))->then($a(1))->otherwise($a(2)),
            'case_when' => $this->caseWhen($c, $inGroup),
            'coalesce' => coalesce(...$args()),
            'in' => $a(0)->isIn($this->setValues($c->args[1]), nullsEqual: true),
            'between' => $a(0)->isBetween($a(1), $a(2)),
            'na_if' => when($a(0)->eq($a(1)))->then(lit(null))->otherwise($a(0)),
            'near' => $a(0)->sub($a(1))->abs()->lt($this->optional($c, 'tol', $inGroup) ?? lit(sqrt(PHP_FLOAT_EPSILON))),
            'abs' => $a(0)->abs(),
            'sqrt' => $a(0)->cast('Float64')->sqrt(),
            'exp' => $a(0)->cast('Float64')->exp(),
            'log' => $a(0)->cast('Float64')->log((float) ($this->literal($c, 'base') ?? M_E)),
            'log10' => $a(0)->cast('Float64')->log10(),
            'round' => $this->round($c, $a(0)),
            'floor' => $a(0)->cast('Float64')->floor(),
            'ceiling' => $a(0)->cast('Float64')->ceil(),
            'as_int' => $this->type($c->args[0]) === 'string'
                ? $a(0)->str()->stripChars()->cast('Float64', strict: false)->cast('Int64', strict: false)
                : $a(0)->cast('Int64', strict: false),
            'as_float' => $a(0)->cast('Float64', strict: false),
            'as_string' => $this->asString($c->args[0], $a(0)),
            'as_bool' => $this->type($c->args[0]) === 'string'
                ? $a(0)->replaceStrict(['TRUE' => true, 'true' => true, 'T' => true, 'True' => true, 'FALSE' => false, 'false' => false, 'F' => false, 'False' => false], default: lit(null)->cast('Boolean'), returnDtype: 'Boolean')
                : $a(0)->cast('Boolean', strict: false),
            'str_lower' => $a(0)->str()->toLowercase(),
            'str_upper' => $a(0)->str()->toUppercase(),
            'str_length' => $a(0)->str()->lenChars()->cast('Int64'),
            'str_trim' => match ($this->literal($c, 'side') ?? 'both') {
                'left' => $a(0)->str()->stripCharsStart(),
                'right' => $a(0)->str()->stripCharsEnd(),
                default => $a(0)->str()->stripChars(),
            },
            'str_sub' => $this->strSub($a(0), $a(1), isset($c->args[2]) ? $a(2) : lit(-1)),
            'str_detect' => $this->negate($c, $a(0)->str()->contains($a(1), literal: true)),
            'str_starts' => $this->negate($c, $a(0)->str()->startsWith($a(1))),
            'str_ends' => $this->negate($c, $a(0)->str()->endsWith($a(1))),
            'str_replace' => $a(0)->str()->replace($a(1), $a(2), literal: true, n: 1),
            'str_replace_all' => $a(0)->str()->replaceAll($a(1), $a(2), literal: true),
            'paste0' => $this->paste($c->args, '', $inGroup),
            'paste' => $this->paste($c->args, (string) ($this->literal($c, 'sep') ?? ' '), $inGroup),
            default => throw new InvalidArgumentException("Unknown function '{$c->fn}'."),
        };
    }

    // Aggregates: arguments are evaluated within the group.

    private function agg(Call $c): Expr
    {
        $fn = $c->fn;
        if ($fn === 'n') {
            return len()->cast('Int64');
        }
        $naRm = (bool) ($this->literal($c, 'na_rm') ?? false);
        if ($fn === 'n_distinct') {
            $args = array_map(fn (PhibbleExpr $x) => $this->node($x, true), $c->args);
            $x = count($args) === 1 ? $args[0] : struct_(...$args);
            if ($naRm) {
                $x = $x->filter(all_horizontal(...array_map(fn (Expr $e) => $e->isNotNull(), $args)));
            }

            return $x->nUnique()->cast('Int64');
        }
        $x = $this->node($c->args[0], true);
        if (in_array($fn, ['first', 'last', 'nth'], true)) {
            return $this->nth($c, $x, $naRm);
        }
        $type = $this->type($c->args[0]);
        $value = match ($fn) {
            'sum' => $type === 'bool' ? $x->cast('Int64')->sum() : $x->sum(),
            'mean' => $x->cast('Float64')->mean(),
            'median' => $x->cast('Float64')->median(),
            'min' => $x->min(),
            'max' => $x->max(),
            'sd' => $x->cast('Float64')->std(),
            'var' => $x->cast('Float64')->var(),
            'quantile' => $x->cast('Float64')->quantile((float) $this->scalarArg($c->args[1]), 'linear'),
            'iqr' => $x->cast('Float64')->quantile(0.75, 'linear')->sub($x->cast('Float64')->quantile(0.25, 'linear')),
            'any' => $x->any(ignoreNulls: $naRm),
            'all' => $x->all(ignoreNulls: $naRm),
        };
        if (! $naRm && in_array($fn, ['quantile', 'iqr'], true)) {
            $this->naChecks[] = $x;
        }
        if ($naRm || in_array($fn, ['any', 'all'], true)) {
            return $value;
        }

        // dplyr's na.rm = FALSE: any NA makes the result NA.
        return when($x->nullCount()->gt(0))->then(lit(null))->otherwise($value);
    }

    private function nth(Call $c, Expr $x, bool $naRm): Expr
    {
        $order = $c->option('order_by');
        if ($order !== null) {
            [$by, $desc] = $this->orderKey($order, true);
            $x = $x->sortBy($by, descending: $desc, nullsLast: true);
        }
        if ($naRm) {
            $x = $x->dropNulls();
        }
        $n = match ($c->fn) {
            'first' => 1,
            'last' => -1,
            'nth' => (int) $this->scalarArg($c->args[1]),
        };
        $default = $this->optional($c, 'default', true);
        if ($n === 0) {
            return $default ?? lit(null);
        }
        $value = $x->get($n > 0 ? $n - 1 : $n, nullOnOob: true);

        return $default === null ? $value : when($x->len()->ge(abs($n)))->then($value)->otherwise($default);
    }

    // Window functions: one value per row, computed over the group in row order.

    private function window(Call $c): Expr
    {
        $fn = $c->fn;
        $arg = fn (int $i) => $this->node($c->args[$i], true);
        if ($fn === 'lag' || $fn === 'lead') {
            $n = (int) ($this->literal($c, 'n') ?? 1);
            $default = $this->optional($c, 'default', true);

            return $arg(0)->shift($fn === 'lag' ? $n : -$n, fillValue: $default);
        }
        if (in_array($fn, Functions::RANKING, true)) {
            return $this->rank($c);
        }
        if ($fn === 'consecutive_id') {
            $args = array_map(fn (PhibbleExpr $x) => $this->node($x, true), $c->args);
            $key = count($args) === 1 ? $args[0] : struct_(...$args);
            $changed = int_range(len())->eq(0)->or($key->neMissing($key->shift()));

            return $changed->cast('Int64')->cumSum();
        }
        $x = $arg(0);
        $type = $this->type($c->args[0]);
        $seenNull = $x->isNull()->cast('Int64')->cumSum()->gt(0);

        return match ($fn) {
            'cumsum' => when($seenNull)->then(lit(null))->otherwise(($type === 'bool' ? $x->cast('Int64') : $x)->cumSum()),
            'cumprod' => when($seenNull)->then(lit(null))->otherwise(($type === 'bool' ? $x->cast('Int64') : $x)->cumProd()),
            'cummin' => when($seenNull)->then(lit(null))->otherwise($x->cumMin()),
            'cummax' => when($seenNull)->then(lit(null))->otherwise($x->cumMax()),
            'cummean' => when($seenNull)->then(lit(null))->otherwise($x->cast('Float64')->cumSum()->truediv(int_range(len())->add(1))),
            'cumall' => when($x->fillNull(true)->not()->cast('Int64')->cumSum()->gt(0))->then(lit(false))
                ->when($seenNull)->then(lit(null))->otherwise(lit(true)),
            'cumany' => when($x->fillNull(false)->cast('Int64')->cumSum()->gt(0))->then(lit(true))
                ->when($seenNull)->then(lit(null))->otherwise(lit(false)),
        };
    }

    private function rank(Call $c): Expr
    {
        $fn = $c->fn;
        if ($c->args === []) {
            $rowNumber = int_range(len())->add(1);
            $x = null;
            $desc = false;
        } else {
            $first = $c->args[0];
            $desc = $first instanceof Desc;
            $x = $this->node($desc ? $first->expr : $first, true);
            $rowNumber = $x->rank('ordinal', descending: $desc)->cast('Int64');
        }
        if ($fn === 'row_number') {
            return $rowNumber;
        }
        $m = $x === null ? len()->cast('Int64') : $x->count()->cast('Int64');
        if ($fn === 'ntile') {
            $bins = lit((int) floor((float) $this->literal($c, 'n')));
            $larger = $m->mod($bins);
            $largerSize = $m->add($bins)->sub(1)->floordiv($bins);
            $smallerSize = $m->floordiv($bins);
            $threshold = $largerSize->mul($larger);

            return when($rowNumber->le($threshold))
                ->then($rowNumber->add($largerSize)->sub(1)->floordiv($largerSize))
                ->otherwise($rowNumber->sub($threshold)->add($smallerSize)->sub(1)->floordiv(max_horizontal(lit(1), $smallerSize))->add($larger));
        }
        if ($x === null) {
            throw new InvalidArgumentException("{$fn}() needs a column to rank.");
        }

        return match ($fn) {
            'min_rank' => $x->rank('min', descending: $desc)->cast('Int64'),
            'dense_rank' => $x->rank('dense', descending: $desc)->cast('Int64'),
            'percent_rank' => $x->rank('min', descending: $desc)->cast('Float64')->sub(1)->truediv($m->sub(1)),
            'cume_dist' => $x->rank('max', descending: $desc)->cast('Float64')->truediv($m),
        };
    }

    // Helpers.

    private function caseWhen(Call $c, bool $inGroup): Expr
    {
        $chain = null;
        for ($k = 0; $k < count($c->args); $k += 2) {
            $cond = $this->node($c->args[$k], $inGroup);
            $value = $this->node($c->args[$k + 1], $inGroup);
            $chain = $chain === null ? when($cond)->then($value) : $chain->when($cond)->then($value);
        }

        return $chain->otherwise($this->optional($c, 'default', $inGroup) ?? lit(null));
    }

    /** R's round(): ties to even at the stored double; negative digits round to tens. */
    private function round(Call $c, Expr $x): Expr
    {
        $digits = (int) ($this->literal($c, 'digits') ?? 0);
        if ($this->type($c->args[0]) === 'int' && $digits >= 0) {
            return $x;
        }
        // Of the two candidates at this many digits, take the one nearer the
        // stored double; an exact tie goes to the even digit.
        $x = $x->cast('Float64');
        $scaled = $digits >= 0 ? $x->mul(10 ** $digits) : $x->truediv(10 ** -$digits);
        $unscale = fn (Expr $e) => $digits >= 0 ? $e->truediv(10 ** $digits) : $e->mul(10 ** -$digits);
        $lo = $unscale($scaled->floor());
        $hi = $unscale($scaled->ceil());
        $dLo = $x->sub($lo);
        $dHi = $hi->sub($x);

        return when($x->isNan()->or($x->isInfinite()))->then($x)
            ->when($dLo->lt($dHi))->then($lo)
            ->when($dLo->gt($dHi))->then($hi)
            ->when($scaled->floor()->mod(2)->eq(0))->then($lo)
            ->otherwise($hi);
    }

    /** str_sub(): 1-based, inclusive end, negative positions count from the end. */
    private function strSub(Expr $s, Expr $start, Expr $end): Expr
    {
        $n = $s->str()->lenChars()->cast('Int64');
        $from = when($start->lt(0))->then($n->add($start)->add(1))->otherwise($start);
        $from = max_horizontal(lit(1), $from);
        $to = when($end->lt(0))->then($n->add($end)->add(1))->otherwise(min_horizontal($end, $n));
        $length = max_horizontal(lit(0), $to->sub($from)->add(1));

        return $s->str()->slice($from->sub(1), $length);
    }

    private function negate(Call $c, Expr $e): Expr
    {
        return ($this->literal($c, 'negate') ?? false) ? $e->not() : $e;
    }

    /** R's as.character(): 15 significant digits for doubles, TRUE/FALSE for logicals. */
    private function asString(PhibbleExpr $node, Expr $x): Expr
    {
        return match ($this->type($node)) {
            'float' => $x->roundSigFigs(15)->cast('String')->str()->replace('\.0$', ''),
            'bool' => when($x)->then(lit('TRUE'))->when($x->not())->then(lit('FALSE'))->otherwise(lit(null)),
            'date' => $x->dt()->strftime('%Y-%m-%d'),
            'datetime' => $x->dt()->strftime('%Y-%m-%d %H:%M:%S'),
            default => $x->cast('String', strict: false),
        };
    }

    /** @param list<PhibbleExpr> $args */
    private function paste(array $args, string $sep, bool $inGroup): Expr
    {
        $parts = array_map(fn (PhibbleExpr $a) => $this->asString($a, $this->node($a, $inGroup))->fillNull('NA'), $args);

        return concat_str($parts, separator: $sep);
    }

    /** @return array{0: Expr, 1: bool} An order_by key and whether it is descending. */
    private function orderKey(PhibbleExpr $order, bool $inGroup = true): array
    {
        $desc = $order instanceof Desc;

        return [$this->node($desc ? $order->expr : $order, $inGroup), $desc];
    }

    /** The values of a list literal, for in(). */
    private function setValues(PhibbleExpr $set): array|Expr
    {
        if ($set instanceof Lit) {
            return is_array($set->value) ? $set->value : [$set->value];
        }

        return $this->node($set, false)->implode();
    }

    private function optional(Call $c, string $name, bool $inGroup): ?Expr
    {
        $node = $c->option($name);

        return $node === null ? null : $this->node($node, $inGroup);
    }

    /** A named option that must be a literal, such as na_rm or digits. */
    private function literal(Call $c, string $name): mixed
    {
        $node = $c->option($name);
        if ($node === null) {
            return null;
        }

        return $this->scalarArg($node);
    }

    private function scalarArg(PhibbleExpr $node): mixed
    {
        if ($node instanceof Lit) {
            return $node->value;
        }
        if ($node instanceof Call && $node->fn === 'neg' && $node->args[0] instanceof Lit) {
            return -$node->args[0]->value;
        }

        throw new InvalidArgumentException('This argument must be a literal value for the Polars backend.');
    }

    /** The R-level type of an expression, as far as it can be told without data. */
    public function type(PhibbleExpr $e): string
    {
        if ($e instanceof Col) {
            $dtype = $this->schema[$e->name] ?? 'String';

            return match (true) {
                str_starts_with($dtype, 'Float') => 'float',
                str_starts_with($dtype, 'Int'), str_starts_with($dtype, 'UInt') => 'int',
                $dtype === 'Boolean', $dtype === 'Null' => 'bool',
                $dtype === 'Date' => 'date',
                str_starts_with($dtype, 'Datetime') => 'datetime',
                default => 'string',
            };
        }
        if ($e instanceof Lit) {
            return match (true) {
                is_float($e->value) => 'float',
                is_int($e->value) => 'int',
                is_bool($e->value) || $e->value === null => 'bool',
                $e->value instanceof DateTimeInterface => $e->dtype === 'date' ? 'date' : 'datetime',
                default => 'string',
            };
        }
        if ($e instanceof Desc) {
            return $this->type($e->expr);
        }
        $args = $e->args;
        $first = fn () => $args === [] ? 'int' : $this->type($args[0]);
        $numeric = function () use ($args): string {
            foreach ($args as $a) {
                if ($this->type($a) === 'float') {
                    return 'float';
                }
            }

            return 'int';
        };

        return match ($e->fn) {
            '+', '-', '*', '%%' => $numeric(),
            '/', '^', 'sqrt', 'exp', 'log', 'log10', 'floor', 'ceiling', 'mean', 'median', 'sd', 'var',
            'quantile', 'iqr', 'percent_rank', 'cume_dist', 'cummean', 'as_float' => 'float',
            'neg', 'abs', 'round', 'min', 'max', 'first', 'last', 'nth', 'lag', 'lead', 'cummin', 'cummax', 'coalesce', 'na_if' => $first(),
            '==', '!=', '<', '<=', '>', '>=', '&', '|', '!', 'is_na', 'in', 'between', 'near', 'str_detect', 'str_starts',
            'str_ends', 'any', 'all', 'cumall', 'cumany', 'as_bool' => 'bool',
            'if_else' => $this->type($args[1]),
            'case_when' => $this->type($args[1]),
            'n', 'n_distinct', 'row_number', 'min_rank', 'dense_rank', 'ntile', 'consecutive_id', 'str_length', 'as_int' => 'int',
            'sum', 'cumsum', 'cumprod' => $first() === 'float' ? 'float' : 'int',
            default => 'string',
        };
    }
}
