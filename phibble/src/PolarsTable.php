<?php

declare(strict_types=1);

namespace Phpolars\Phibble;

use Closure;
use Generator;
use InvalidArgumentException;
use Phibble\Column\FactorColumn;
use Phibble\Contracts\Table;
use Phibble\Exception\ColumnNotFound;
use Phibble\Expr\Col;
use Phibble\Expr\Desc;
use Phibble\Expr\Expr as PhibbleExpr;
use Phibble\Frame;
use Phibble\Row;
use Phibble\Type;
use Phpolars\DataFrame;
use Phpolars\Expr;
use Phpolars\LazyFrame;
use Phpolars\Series;

use function Phpolars\{all, col, lit};

/**
 * A Phibble\Contracts\Table backed by a Polars DataFrame.
 *
 * Verbs that take column names or Phibble\Expr run in Polars. Verbs given a
 * PHP closure cannot run on Polars threads, so they materialise a
 * Phibble\Frame, run there, and convert the result back.
 *
 * Factor columns are held as strings in Polars; their levels travel with
 * the table so that collect() and group ordering keep them.
 */
final class PolarsTable implements Table
{
    /**
     * @param list<string> $groups
     * @param array<string, array{levels: list<string>, ordered: bool}> $factors
     */
    private function __construct(
        private readonly DataFrame $df,
        private readonly array $groups = [],
        private readonly bool $rowwise = false,
        private readonly array $factors = [],
        private readonly bool $dropGroups = true,
    ) {}

    /** Wrap a Polars frame; a LazyFrame is collected. */
    public static function fromPolars(DataFrame|LazyFrame $df): self
    {
        return new self($df instanceof LazyFrame ? $df->collect() : $df);
    }

    /** Copy any phibble table into Polars, keeping types, groups and factor levels. */
    public static function fromTable(Table $table): self
    {
        if ($table instanceof self) {
            return $table;
        }
        $frame = $table->collect();
        $schema = [];
        $factors = [];
        foreach ($frame->types() as $name => $type) {
            $dtype = match ($type) {
                Type::Int => 'Int64',
                Type::Float => 'Float64',
                Type::Bool => 'Boolean',
                Type::String, Type::Factor => 'String',
                Type::Date => 'Date',
                Type::DateTime => 'Datetime(us)',
                Type::List => null,
            };
            if ($dtype !== null) {
                $schema[$name] = $dtype;
            }
            if ($type === Type::Factor) {
                $storage = $frame->storage($name);
                $factors[$name] = ['levels' => $storage->levels(), 'ordered' => $storage instanceof FactorColumn && $storage->isOrdered()];
            }
        }
        $df = DataFrame::fromDict($frame->toColumns(), schema: $schema);

        return new self($df, $frame->groups(), $frame->isRowwise(), $factors, $frame->dropsEmptyGroups());
    }

    /** The underlying Polars frame. */
    public function polars(): DataFrame
    {
        return $this->df;
    }

    private function with(DataFrame $df, ?array $groups = null, ?bool $rowwise = null, ?array $factors = null, ?bool $drop = null): static
    {
        $factors ??= $this->factors;
        $columns = array_flip($df->columns());
        $groups ??= $this->groups;

        return new self(
            $df,
            $groups,
            $rowwise ?? $this->rowwise,
            array_intersect_key($factors, $columns),
            $groups === [] ? true : ($drop ?? $this->dropGroups),
        );
    }

    /**
     * With groupBy(drop: false), a factor key has a group for every level,
     * even empty ones. Polars only sees observed groups, so those verbs run
     * on a Frame.
     */
    private function keepsEmptyGroups(): bool
    {
        return ! $this->dropGroups && array_any($this->groups, fn (string $g) => isset($this->factors[$g]));
    }

    /** How many verbs fell back to an in-memory Frame in this process. */
    public static int $fallbacks = 0;

    /** Run a verb on an in-memory Frame and bring the result back. */
    private function viaFrame(Closure $verb): static
    {
        self::$fallbacks++;

        return self::fromTable($verb($this->collect()));
    }

    private function check(string ...$names): void
    {
        $columns = $this->df->columns();
        foreach ($names as $name) {
            if (! in_array($name, $columns, true)) {
                throw ColumnNotFound::named($name, $columns);
            }
        }
    }

    private function translator(bool $grouped = true): Translator
    {
        return new Translator($this->df->schema(), $grouped ? $this->groups : []);
    }

    /** Sort keys for group order: factor levels first, NA last, strings by byte. */
    private function groupOrder(array $keys): array
    {
        return array_map(function (string $k) {
            if (! isset($this->factors[$k])) {
                return col($k);
            }

            return col($k)->replaceStrict(array_flip($this->factors[$k]['levels']), default: lit(null), returnDtype: 'Int64');
        }, $keys);
    }

    /** Raise R's error when a quantile() argument holds NA and na_rm is FALSE. */
    private function guard(Translator $translator): void
    {
        foreach ($translator->naChecks as $arg) {
            if ($this->df->select($arg->isNull()->any()->alias('na'))->item() === true) {
                throw new InvalidArgumentException("missing values and NaN's not allowed if 'na_rm' is FALSE");
            }
        }
    }

    // Shape and values

    public function columns(): array
    {
        return $this->df->columns();
    }

    public function types(): array
    {
        $types = [];
        foreach ($this->df->schema() as $name => $dtype) {
            $types[$name] = isset($this->factors[$name]) ? Type::Factor : match (true) {
                str_starts_with($dtype, 'Int'), str_starts_with($dtype, 'UInt') => Type::Int,
                str_starts_with($dtype, 'Float') => Type::Float,
                $dtype === 'Boolean', $dtype === 'Null' => Type::Bool,
                $dtype === 'Date' => Type::Date,
                str_starts_with($dtype, 'Datetime') => Type::DateTime,
                str_starts_with($dtype, 'List'), str_starts_with($dtype, 'Array'), str_starts_with($dtype, 'Struct') => Type::List,
                $dtype === 'Categorical' => Type::Factor,
                default => Type::String,
            };
        }

        return $types;
    }

    public function nrow(): int
    {
        return $this->df->height();
    }

    public function ncol(): int
    {
        return $this->df->width();
    }

    public function count(): int
    {
        return $this->df->height();
    }

    public function column(string $name): array
    {
        $this->check($name);

        return $this->df->getColumn($name)->toList();
    }

    public function getIterator(): Generator
    {
        foreach ($this->df->toDicts() as $i => $row) {
            yield $i => new Row($row);
        }
    }

    public function toRows(): array
    {
        return $this->df->toDicts();
    }

    public function toColumns(): array
    {
        return $this->df->toDict(asSeries: false);
    }

    public function collect(): Frame
    {
        $columns = $this->df->toDict(asSeries: false);
        $types = [];
        foreach ($this->types() as $name => $type) {
            if ($type === Type::Factor && isset($this->factors[$name])) {
                $levels = $this->factors[$name]['levels'];
                $extra = array_diff(array_unique(array_filter($columns[$name], fn ($v) => $v !== null)), $levels);
                $columns[$name] = FactorColumn::fromValues($columns[$name], [...$levels, ...array_values($extra)], $this->factors[$name]['ordered']);
            } elseif ($type !== Type::List) {
                $types[$name] = $type;
            }
        }
        $frame = Frame::fromColumns($columns, $types);
        if ($this->rowwise) {
            return $frame->rowwise($this->groups);
        }

        return $this->groups === [] ? $frame : $frame->groupBy($this->groups, $this->dropGroups);
    }

    // Verbs

    public function select(array $names): static
    {
        $names = array_values(array_unique($names));
        $this->check(...$names);
        // Grouping columns always stay, as in dplyr.
        $names = [...array_values(array_diff($this->groups, $names)), ...$names];

        return $this->with($this->df->select($names));
    }

    public function filter(Closure|PhibbleExpr $predicate): static
    {
        if ($predicate instanceof Closure || $this->rowwise || $this->keepsEmptyGroups()) {
            return $this->viaFrame(fn (Frame $f) => $f->filter($predicate));
        }

        $translator = $this->translator();
        $predicate = $translator->row($predicate);
        $this->guard($translator);

        return $this->with($this->df->filter($predicate));
    }

    public function filterGroups(Closure $predicate): static
    {
        return $this->viaFrame(fn (Frame $f) => $f->filterGroups($predicate));
    }

    public function mutate(array $columns): static
    {
        if ($this->rowwise || $this->keepsEmptyGroups() || array_any($columns, fn ($v) => $v instanceof Closure)) {
            return $this->viaFrame(fn (Frame $f) => $f->mutate($columns));
        }
        $df = $this->df;
        $factors = $this->factors;
        foreach ($columns as $name => $value) {
            $name = (string) $name;
            $translator = new Translator($df->schema(), $this->groups);
            $expr = match (true) {
                $value instanceof PhibbleExpr => $translator->row($value),
                is_array($value) => $this->listColumn($value, $df->height()),
                $value === null => lit(null)->cast('Boolean'),
                default => lit($value),
            };
            (new self($df))->guard($translator);
            $df = $df->withColumns([$name => $expr]);
            unset($factors[$name]);
        }

        return $this->with($df, factors: $factors);
    }

    private function listColumn(array $values, int $rows): Expr
    {
        if (count($values) !== $rows) {
            throw new InvalidArgumentException('A column of values must have '.$rows.' entries, not '.count($values).'.');
        }

        return lit(new Series('', array_values($values)));
    }

    public function mutateGroups(array $columns): static
    {
        return $this->viaFrame(fn (Frame $f) => $f->mutateGroups($columns));
    }

    public function rename(array $map): static
    {
        $this->check(...array_values($map));
        $flipped = [];
        foreach ($map as $new => $old) {
            $flipped[$old] = (string) $new;
        }
        $groups = array_map(fn (string $g) => $flipped[$g] ?? $g, $this->groups);
        $factors = [];
        foreach ($this->factors as $name => $meta) {
            $factors[$flipped[$name] ?? $name] = $meta;
        }

        return $this->with($this->df->rename($flipped), $groups, factors: $factors);
    }

    public function arrange(array $keys): static
    {
        if ($keys === []) {
            return $this;
        }
        $by = [];
        $descending = [];
        $translator = $this->translator(false);
        foreach ($keys as $key) {
            [$expr, $desc] = match (true) {
                is_string($key) => [$key, false],
                is_array($key) => [$key[0], ($key[1] ?? 'asc') === 'desc'],
                $key instanceof Desc => [$key->expr, true],
                $key instanceof PhibbleExpr => [$key, false],
                default => throw new InvalidArgumentException('arrange() keys are names, [name, direction] pairs or expressions.'),
            };
            if (is_string($expr)) {
                $this->check($expr);
                $expr = new Col($expr);
            }
            $by[] = $expr instanceof Col && isset($this->factors[$expr->name])
                ? $this->groupOrder([$expr->name])[0]
                : $translator->row($expr);
            $descending[] = $desc;
        }

        return $this->with($this->df->sort($by, descending: $descending, nullsLast: true, maintainOrder: true));
    }

    public function groupBy(array $names, bool $drop = true): static
    {
        $this->check(...$names);

        return $this->with($this->df, array_values($names), false, drop: $drop);
    }

    public function dropsEmptyGroups(): bool
    {
        return $this->dropGroups;
    }

    public function ungroup(): static
    {
        return $this->with($this->df, [], false);
    }

    public function groups(): array
    {
        return $this->groups;
    }

    public function rowwise(array $idColumns = []): static
    {
        $this->check(...$idColumns);

        return $this->with($this->df, array_values($idColumns), true);
    }

    public function isRowwise(): bool
    {
        return $this->rowwise;
    }

    public function eachGroup(): iterable
    {
        if ($this->rowwise || $this->keepsEmptyGroups()) {
            foreach ($this->collect()->eachGroup() as $i => $group) {
                yield $i => ['key' => $group['key'], 'table' => self::fromTable($group['table'])];
            }

            return;
        }
        foreach ($this->groupRows() as $i => $group) {
            yield $i => ['key' => $group['key'], 'table' => $this->ungroup()->take($group['rows'])];
        }
    }

    public function groupRows(): array
    {
        if ($this->rowwise || $this->keepsEmptyGroups()) {
            return $this->collect()->groupRows();
        }
        if ($this->groups === []) {
            return [['key' => [], 'rows' => $this->nrow() === 0 ? [] : range(0, $this->nrow() - 1)]];
        }
        $index = $this->df->withRowIndex('__row')
            ->groupBy($this->groups, maintainOrder: true)
            ->agg(col('__row'))
            ->sort($this->groupOrder($this->groups), nullsLast: true, maintainOrder: true);
        $out = [];
        foreach ($index->toDicts() as $group) {
            $rows = $group['__row'];
            unset($group['__row']);
            $out[] = ['key' => $group, 'rows' => $rows];
        }

        return $out;
    }

    /** @param ?string $groups dplyr's .groups: drop_last (default), drop, keep or rowwise. */
    public function summarise(array $aggregates, ?string $groups = null): static
    {
        if ($this->rowwise || $this->keepsEmptyGroups() || array_any($aggregates, fn ($v) => $v instanceof Closure)) {
            return $this->viaFrame(fn (Frame $f) => $f->summarise($aggregates, $groups));
        }
        if (! in_array($groups, [null, 'drop_last', 'drop', 'keep', 'rowwise'], true)) {
            throw new InvalidArgumentException("summarise() .groups must be drop_last, drop, keep or rowwise, not '{$groups}'.");
        }
        $translator = $this->translator(false);
        $named = [];
        foreach ($aggregates as $name => $expr) {
            if (! $expr instanceof PhibbleExpr) {
                $expr = new \Phibble\Expr\Lit($expr);
            }
            if (! Translator::isScalar($expr)) {
                $size = $this->groups === [] ? $this->nrow() : $this->firstGroupSize();
                throw new \LengthException(
                    "summarise() '{$name}' must be size 1, not {$size}. Use an aggregate such as mean() or n(), or reframe() for several rows per group."
                );
            }
            $named[(string) $name] = $translator->aggregate($expr);
        }
        $this->guard($translator);
        if ($this->groups === []) {
            $out = $this->with($this->df->select($named), []);

            return $groups === 'rowwise' ? $out->rowwise() : $out;
        }
        $out = $this->df->groupBy($this->groups, maintainOrder: true)->agg($named)
            ->sort($this->groupOrder($this->groups), nullsLast: true, maintainOrder: true);

        return match ($groups ?? 'drop_last') {
            'drop_last' => $this->with($out, array_slice($this->groups, 0, -1)),
            'drop' => $this->with($out, []),
            'keep' => $this->with($out, $this->groups),
            'rowwise' => $this->with($out, $this->groups, true),
        };
    }

    private function firstGroupSize(): int
    {
        $sizes = $this->df->groupBy($this->groups, maintainOrder: true)->len()
            ->sort($this->groupOrder($this->groups), nullsLast: true, maintainOrder: true);

        return $sizes->height() === 0 ? 0 : (int) $sizes->item(0, 'len');
    }

    public function distinct(array $names = []): static
    {
        $names = $names === [] ? $this->df->columns() : $names;
        $this->check(...$names);
        $out = $this->df->unique(subset: $names, keep: 'first', maintainOrder: true)->select($names);

        return $this->with($out, array_values(array_intersect($this->groups, $names)));
    }

    public function take(array $indices): static
    {
        $rows = $this->nrow();
        foreach ($indices as $i) {
            if ($i !== null && ($i < 0 || $i >= $rows)) {
                throw new InvalidArgumentException("Row index {$i} is out of range for a table of {$rows} rows.");
            }
        }
        if ($indices === []) {
            return $this->with($this->df->head(0));
        }
        $rows = DataFrame::fromDict(['__row' => array_values($indices)], schema: ['__row' => 'UInt32']);
        $out = $rows->join($this->df->withRowIndex('__row'), on: '__row', how: 'left', maintainOrder: 'left')->drop('__row');

        return $this->with($out);
    }

    public function join(
        Table $other,
        array $by,
        string $type = 'inner',
        array $suffix = ['.x', '.y'],
        ?bool $keep = null,
        bool $naMatches = true,
        string $multiple = 'all',
        ?string $relationship = null,
        string $unmatched = 'drop',
    ): static {
        $y = self::fromTable($other);
        [$df, $factors] = Joiner::run($this, $y, $by, $type, $suffix, $keep, $naMatches, $multiple, $relationship, $unmatched);

        return $this->with($df, array_values(array_intersect($this->groups, $df->columns())), false, $factors);
    }

    public function slice(int $offset, ?int $length = null): static
    {
        return $this->with($this->df->slice($offset, $length));
    }

    public function head(int $n = 6): static
    {
        return $this->with($this->df->head($n));
    }

    /** @internal Factor metadata for Joiner. */
    public function factorLevels(): array
    {
        return $this->factors;
    }
}
